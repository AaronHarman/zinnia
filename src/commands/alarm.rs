use crate::commands::{Command, CommandResult};
use std::sync::mpsc::{Sender};
use crate::SpeakMessage;
use text2num::{Language, replace_numbers_in_text};
use std::thread;
use std::time::Duration;

pub struct AlarmCommand {}
impl Command for AlarmCommand {
    fn name(&self) -> String {
        return String::from("Alarm");
    }
    fn desc(&self) -> String {
        return String::from("This command allows you to set an alarm for a specific time or after a duration.");
    }
    fn help(&self) -> String {
        return String::from("Use \"Alarm\" for a set time or \"Timer\" for a set duration.");
    }
    fn uses_internet(&self) -> bool {
        return false;
    }
    fn recognize(&self, text : String) -> bool {
        return text.contains("timer") || text.contains("alarm");
    }
    fn effect(&mut self, text : String, speak : Sender<SpeakMessage>) -> CommandResult {
        let en = Language::english();
        let numtext = replace_numbers_in_text(&text, &en, 0.0);
        let words : Vec<&str> = numtext.split_whitespace().collect();

        if text.contains("timer") {
            // timer stuff
            let mut millis = 0;
            for pair in words.windows(2) {
                if pair[1].contains("second") {
                    match pair[0].parse::<u64>() {
                        Ok(n) => millis += n * 1000,
                        Err(_) => {},
                    }
                } else if pair[1].contains("minute") {
                    match pair[0].parse::<u64>() {
                        Ok(n) => millis += n * 60000,
                        Err(_) => {},
                    }
                } else if pair[1].contains("hour") {
                    match pair[0].parse::<u64>() {
                        Ok(n) => millis += n * 3600000,
                        Err(_) => {},
                    }
                } 
            }
            // temporary message to make sure it's adding up times correctly
            speak.send(SpeakMessage::Say(String::from("Timer set."))).unwrap();
            // set a thread to wait for the allotted amount of time
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(millis));
                speak.send(SpeakMessage::Say(String::from("Your timer has run out."))).unwrap();
            });
        } else if text.contains("alarm") {
            // alarm stuff
        }
        //speak.send(SpeakMessage::Say(String::from("Test Command recognized. What you said was: ") + &text)).unwrap();
        return CommandResult::Done;
    }
}


