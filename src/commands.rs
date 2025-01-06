use std::sync::mpsc::{Sender};
use crate::SpeakMessage;

// all the command modules
mod test;
use test::TestCommand;
mod help;
use help::HelpCommand;

// the result of the execution of a command, indicates whether it needs more input
pub enum CommandResult {
    Done,
    Continue(fn(String, Sender<SpeakMessage>)->CommandResult),
}
impl CommandResult {
    // converts to DispatchResult, for sending to things that don't need the function'
    pub fn to_dispatch(&self) -> DispatchResult {
        match self {
            CommandResult::Done => {return DispatchResult::Done},
            CommandResult::Continue(_) => {return DispatchResult::Continue},
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
    next_func : Option<fn(String, Sender<SpeakMessage>)->CommandResult>,
    speak : Sender<SpeakMessage>,
}
impl CommandDirector {
    // this should populate the commands list with all available commands, in order of priority
    pub fn new(speak : Sender<SpeakMessage>) -> CommandDirector {
        let mut cd = CommandDirector {
            commands : Vec::new(),
            next_func : None,
            speak,
        };
        // add commands here
        cd.commands.push(Box::new(TestCommand{})); // I should probably make a ::new() for this
        cd.commands.insert(0, Box::new(HelpCommand::new(&cd.commands)));
        return cd;
    }

    // takes in text, then determines which command it matches and executes it
    pub fn dispatch_command(&mut self, text : String) -> DispatchResult {
        // if there's a leftover function from last time
        if self.next_func.is_some() {
            let func = self.next_func.unwrap();
            let result = func(text, self.speak.clone());
            match result {
                CommandResult::Done => {
                    self.next_func = None;
                    return DispatchResult::Done
                },
                CommandResult::Continue(c) => {
                    self.next_func = Some(c);
                    return DispatchResult::Continue;
                }
            }
        }
        // if there's not a leftover function from last time
        for command in &mut self.commands {
            if command.recognize(text.clone()) {
                let result = command.effect(text, self.speak.clone());
                match result {
                    CommandResult::Done => {return DispatchResult::Done},
                    CommandResult::Continue(c) => {
                        self.next_func = Some(c);
                        return DispatchResult::Continue;
                    }
                }
            }
        }
        self.speak.send(SpeakMessage::Say(String::from("I'm not sure what you're asking for. Please try again."))).unwrap();
        return DispatchResult::Done;
    }
}
