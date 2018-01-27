use std::fmt;

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
