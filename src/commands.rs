use std::sync::mpsc::{Sender};
use crate::SpeakMessage;

// all the command modules
mod test;
use test::TestCommand;
mod help;
use help::HelpCommand;
mod weather;
use weather::WeatherCommand;

// the result of the execution of a command, indicates whether it needs more input
// it used to be different from DispatchResult, but has since been changed to be the same.
// it's being kept separate in case that changes again
pub enum CommandResult {
    Done,
    Continue,
}
impl CommandResult {
    // converts to DispatchResult, for sending to things that don't need the function'
    pub fn to_dispatch(&self) -> DispatchResult {
        match self {
            CommandResult::Done => {return DispatchResult::Done},
            CommandResult::Continue => {return DispatchResult::Continue},
        }
    }
}

// the result of calling dispatch_command
pub enum DispatchResult {
    Done,
    Continue,
}

// Trait to define all the functionality of a Zinnia command
pub trait Command {
    fn name(&self) -> String;
    fn desc(&self) -> String;
    fn help(&self) -> String;
    fn uses_internet(&self) -> bool;
    fn recognize(&self, text : String) -> bool;
    fn effect(&mut self, text : String, speak : Sender<SpeakMessage>) -> CommandResult;
}

// the object responsible for running commands
pub struct CommandDirector {
    commands : Vec<Box<dyn Command>>,
    next_comm : Option<usize>,
    speak : Sender<SpeakMessage>,
}
impl CommandDirector {
    // this should populate the commands list with all available commands, in order of priority
    pub fn new(speak : Sender<SpeakMessage>) -> CommandDirector {
        let mut cd = CommandDirector {
            commands : Vec::new(),
            next_comm : None,
            speak,
        };
        // add commands here
        cd.commands.push(Box::new(TestCommand{})); // I should probably make a ::new() for this
        cd.commands.push(Box::new(WeatherCommand::new(String::from("Drums"))));
        cd.commands.insert(0, Box::new(HelpCommand::new(&cd.commands)));
        return cd;
    }

    // takes in text, then determines which command it matches and executes it
    pub fn dispatch_command(&mut self, text : String) -> DispatchResult {
        // if there's a leftover function from last time
        if self.next_comm.is_some() {
            let result = self.commands[self.next_comm.unwrap()].effect(text, self.speak.clone());
            match result {
                CommandResult::Done => {
                    self.next_comm = None;
                    return DispatchResult::Done
                },
                CommandResult::Continue => {
                    // next_comm stays the same
                    return DispatchResult::Continue;
                }
            }
        }
        // if there's not a leftover function from last time
        for (index, command) in &mut self.commands.iter_mut().enumerate() {
            if command.recognize(text.clone()) {
                let result = command.effect(text, self.speak.clone());
                match result {
                    CommandResult::Done => {return DispatchResult::Done},
                    CommandResult::Continue => {
                        self.next_comm = Some(index);
                        return DispatchResult::Continue;
                    }
                }
            }
        }
        self.speak.send(SpeakMessage::Say(String::from("I'm not sure what you're asking for. Please try again."))).unwrap();
        return DispatchResult::Done;
    }
}
