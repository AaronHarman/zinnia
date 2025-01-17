use crate::commands::{Command, CommandResult};
use std::process::Command as ExtCommand;
use std::sync::mpsc::{Sender};
use crate::SpeakMessage;
use reqwest;
use json;

pub struct WeatherCommand {
    default_loc : String,    
}
impl WeatherCommand {
    pub fn new(default_loc : String) -> WeatherCommand {
        return WeatherCommand{default_loc};
    }
}
impl Command for WeatherCommand {
    fn name(&self) -> String {
        return String::from("Weather");
    }
    fn desc(&self) -> String {
        return String::from("This command can give you weather information about a given location.");
    }
    fn help(&self) -> String {
        return String::from("Mention the weather and a location to recieve weather data about that location. Make sure to precede the location with the word \"in\".");
    }
    fn uses_internet(&self) -> bool {
        return true;
    }
    fn recognize(&self, text : String) -> bool {
        return text.contains("weather");
    }
    fn effect(&mut self, text : String, speak : Sender<SpeakMessage>) -> CommandResult {
        let mut place = self.default_loc.clone();
        let words = text.split_whitespace().collect::<Vec<_>>();
        if words.contains(&"in") {
            let mut iter = words.split_inclusive(|s| *s == "in");
            place = iter.nth(1).unwrap().join("+");
        }
        let response = reqwest::blocking::get(format!("wttr.in/{}?format=j1", place));
        //TODO ALL OF THIS LMAO
        let result = ExtCommand::new("curl")
            .arg(format!("wttr.in/{}?format=j1", place))
            .output()
            .expect("Failed to execute curl")
            .stdout;
        let stringy = String::from_utf8(result).unwrap();
        let parsed = json::parse(&stringy).expect("Couldn't parse JSON response");
        println!("{:?}", parsed["current_condition"].pretty(2));
        let weather = parsed["current_condition"][0]["weatherDesc"][0]["value"].dump();
        let temp = parsed["current_condition"][0]["temp_F"].dump();
        let feels = parsed["current_condition"][0]["FeelsLikeF"].dump();
        speak.send(SpeakMessage::Say(format!("The weather in {} is {}. It is {} degrees and feels like {} degrees.",
                                             place, weather, temp, feels))).unwrap();
        return CommandResult::Done;
    }
}


