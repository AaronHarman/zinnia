use crate::commands::{Command, CommandResult};
use std::sync::mpsc::{Sender};
use crate::SpeakMessage;

enum State {
    AskForCommand,
    GiveHelp,
}

pub struct HelpCommand {
    list : Vec<(String, String, bool)>,
    state : State,
}
impl HelpCommand {
    pub fn new(commands : &Vec<Box<dyn Command>>) -> HelpCommand {
        let mut list = Vec::new();
        for command in commands {
            let mut lower = command.name();
            lower.make_ascii_lowercase();
            list.push((lower, command.help(), command.uses_internet()));
        }
        return HelpCommand{
            list,
            state : State::AskForCommand,
        };
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
        match self.state {
            State::AskForCommand => {
                speak.send(SpeakMessage::Say(String::from("Which command would you like help with?"))).unwrap();
                self.state = State::GiveHelp;
                return CommandResult::Continue;
            },
            State::GiveHelp => {
                // say the help text for whatever command they say
                for (name, help, net) in &self.list {
                    if text.contains(name) {
                        let does_it = if *net {String::from("does")} else {String::from("does not")};
                        speak.send(SpeakMessage::Say(format!("{} This command {} require the internet.", help, does_it))).unwrap();
                        return CommandResult::Done;
                    }
                }
                speak.send(SpeakMessage::Say(format!("I couldn't find a command named {}, please try again.", text))).unwrap();
                self.state = State::AskForCommand;
                return CommandResult::Done;
            }
        }

    }
}
