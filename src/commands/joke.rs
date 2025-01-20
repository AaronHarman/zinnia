use crate::commands::{Command, CommandResult};
use std::sync::mpsc::{Sender};
use crate::SpeakMessage;
use reqwest;

pub struct JokeCommand {}
impl JokeCommand {
    // for sending HTTP requests
    fn request(url : String) -> Result<String, &'static str> {
        let client = reqwest::blocking::Client::new();
        let response = client.get(url)
            .header(reqwest::header::ACCEPT, "text/plain")
            .send().or(Err("I had a problem connecting to the joke service"))?;
        if response.status().is_success() {
            let text = response.text().or(Err("I had a problem understanding the joke service."))?;
            return Ok(text);
        } else {
            return Err("I didn't get a response from the joke service.");
        }
    }
}
impl Command for JokeCommand {
    fn name(&self) -> String {
        return String::from("Joke");
    }
    fn desc(&self) -> String {
        return String::from("This command will tell you a joke.");
    }
    fn help(&self) -> String {
        return String::from("Ask for a joke and you will recieve one.");
    }
    fn uses_internet(&self) -> bool {
        return true;
    }
    fn recognize(&self, text : String) -> bool {
        return text.contains("joke");
    }
    fn effect(&mut self, _text : String, speak : Sender<SpeakMessage>) -> CommandResult {
        let response = JokeCommand::request(String::from("https://icanhazdadjoke.com"));
        let stringy : String = match response {
            Ok(s) => {s},
            Err(e) => {
                speak.send(SpeakMessage::Say(format!("{} Please try again later.", e))).unwrap();
                return CommandResult::Done;
            }
        };
        println!("{}", stringy);
        speak.send(SpeakMessage::Say(stringy)).unwrap();
        return CommandResult::Done;
    }
}

