use std::fmt;
use irc::client::prelude::*;
use std::boxed::Box;

pub struct Action {
    pub from: String,
    pub action: Box<FnMut(&IrcClient) -> () + Send>
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Action from {}", self.from)
    }
}
