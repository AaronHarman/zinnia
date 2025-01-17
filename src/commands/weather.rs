use crate::commands::{Command, CommandResult};
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
    // for sending HTTP requests
    fn request(url : String) -> Result<String, &'static str> {
        let response = reqwest::blocking::get(url).or(Err("I was unable to connect to the weather service."))?;
        if response.status().is_success() {
            let text = response.text().or(Err("I had a problem understanding the weather service."))?;
            return Ok(text);
        } else {
            return Err("I didn't get a response from the weather service.");
        }
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
        let response = WeatherCommand::request(format!("wttr.in/{}?format=j1", place));
        let stringy : String = match response {
            Ok(s) => {s},
            Err(e) => {
                speak.send(SpeakMessage::Say(format!("{} Please try again later.", e))).unwrap();
                return CommandResult::Done;
            }
        };
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


