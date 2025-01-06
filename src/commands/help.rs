use crate::commands::{Command, CommandResult};
use std::sync::mpsc::{Sender};
use crate::SpeakMessage;

pub struct HelpCommand {
    list : Vec<(String, String, bool)>,
}
impl HelpCommand {
    pub fn new(commands : &Vec<Box<dyn Command>>) -> HelpCommand {
       let mut list = Vec::new();
       for command in commands {
           list.push((command.name().make_ascii_lowercase(), command.help(), command.uses_internet()));
       }
       return HelpCommand{list};
    }
    // follow-up function for when you give it the name of a command
    pub fn effect2(&mut self, text : String, speak : Sender<SpeakMessage>) -> CommandResult {
        //TODO This whole thing lol
        return CommandResult::Done;
    }
}
impl Command for HelpCommand {
    fn name(&self) -> String {
        return String::from("Help Command");
    }
    fn desc(&self) -> String {
        return String::from("This command gives help information for any of the available commands.");
    }
    fn help(&self) -> String {
        return String::from("Say \"Help\" and then supply the name of a command when prompted.");
    }
    fn uses_internet(&self) -> bool {
        return false;
    }
    fn recognize(&self, text : String) -> bool {
        return text.contains("help");
    }
    fn effect(&mut self, text : String, speak : Sender<SpeakMessage>) -> CommandResult {
        speak.send(SpeakMessage::Say(String::from("Which command would you like help with?"))).unwrap();
        return CommandResult::Continue(self.effect2);
    }
}
