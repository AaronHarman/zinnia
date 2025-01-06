use crate::commands::{Command, CommandResult};
use std::sync::mpsc::{Sender};
use crate::SpeakMessage;

pub struct TestCommand {}
impl Command for TestCommand {
    fn name(&self) -> String {
        return String::from("Test Command");
    }
    fn desc(&self) -> String {
        return String::from("This command is purely to test if commands work. It doesn't do anything productive.");
    }
    fn help(&self) -> String {
        return String::from("Simply say a phrase containing the words \"Test Command\" and you will get a response.");
    }
    fn uses_internet(&self) -> bool {
        return false;
    }
    fn recognize(&self, text : String) -> bool {
        return text.contains("test command");
    }
    fn effect(&mut self, text : String, speak : Sender<SpeakMessage>) -> CommandResult {
        speak.send(SpeakMessage::Say(String::from("Test Command recognized. What you said was: ") + &text)).unwrap();
        return CommandResult::Done;
    }
}


