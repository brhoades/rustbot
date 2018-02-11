use actions::Action;
use std::fmt;

pub type CommandError = String;

pub trait CommandHandler {
    fn get_name(&self) -> &'static str;
    fn handles_event(&self, &CommandEvent) -> bool;
    fn method(&self, &CommandEvent) -> Result<Action,CommandError>;
}

// Identified command that had bee digested
#[derive(Clone)]
pub struct CommandEvent {
    pub message: String,
    pub name: String,
    pub channel: String,
    pub args: Vec<String>
}

impl fmt::Debug for CommandEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {} {:?}", self.channel, self.name, self.args)
    }
}
