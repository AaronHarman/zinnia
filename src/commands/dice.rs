use crate::commands::{Command, CommandResult};
use std::sync::mpsc::{Sender};
use crate::SpeakMessage;
use text2num::{Language, text2digits};
use rand;

pub struct DiceCommand {}
impl Command for DiceCommand {
    fn name(&self) -> String {
        return String::from("Dice Command");
    }
    fn desc(&self) -> String {
        return String::from("This command rolls dice.");
    }
    fn help(&self) -> String {
        return String::from("Say \"Roll\" followed by a type and number of dice in the number D number format.");
    }
    fn uses_internet(&self) -> bool {
        return false;
    }
    fn recognize(&self, text : String) -> bool {
        return text.contains("roll ") || text.contains("role ");
    }
    fn effect(&mut self, text : String, speak : Sender<SpeakMessage>) -> CommandResult {
        // parse out the numbers given
        let (_, dice) = if text.contains("roll") {
            text.split_once("roll").unwrap()
        } else {
            text.split_once("role").unwrap()
        };
        if !dice.contains("d") {
            speak.send(SpeakMessage::Say(String::from("Make sure to say \"Roll\" followed by a type and number of dice in the number D number format."))).unwrap();
            return CommandResult::Done;
        }
        let (str_num, str_size) = dice.split_once("d").unwrap();
        println!("numbers: \"{}\", \"{}\"", str_num, str_size);
        // convert the numbers into u32
        let en = Language::english();
        let num : u32 = match text2digits(str_num, &en) {
            Ok(n) => n,
            Err(_) => {
                speak.send(SpeakMessage::Say(String::from("I couldn't make out the first number. Please try again."))).unwrap();
                return CommandResult::Done;
            },
        }.parse().unwrap();
        let size : u32 = match text2digits(str_size, &en) {
            Ok(n) => n,
            Err(_) => {
                speak.send(SpeakMessage::Say(String::from("I couldn't make out the second number. Please try again."))).unwrap();
                return CommandResult::Done;
            },
        }.parse().unwrap();
        // roll the dice and format the output
        let mut answer = String::from("I rolled: ");
        for i in 1..=num {
            let roll = (rand::random::<u32>() % size) + 1;
            if num == 1 {
                answer.push_str(&format!("{}.", roll));
            }
            else if i == num {
                answer.push_str(&format!("and {}.", roll));
            } else {
                answer.push_str(&format!("{}, ", roll));
            }
        }
        speak.send(SpeakMessage::Say(answer)).unwrap();
        return CommandResult::Done;
    }
}


