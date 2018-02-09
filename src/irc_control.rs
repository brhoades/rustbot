use std::boxed::Box;
use irc::client::prelude::*;
use actions::Action;
use command::{CommandHandler,CommandError,CommandEvent};

pub struct IRCControlCommand;

impl CommandHandler for IRCControlCommand {
    fn get_name(&self) -> &'static str {
        "IRC Control Commands"
    }

    fn method(&self, event: &CommandEvent) -> Result<Action,CommandError> {
        let supported_events = ["join", "part"];
        let name = event.name.as_str();
        let channel = event.args[0].clone();

        if supported_events.contains(&name) {
            let local_event = event.clone();
            Ok(Action {
                action: Box::new(move |server: &IrcClient| {
                    match server.send_join(channel.as_str()) {
                        Ok(()) => server.send_privmsg(local_event.channel.as_str(), format!("Joined channel {}", channel).as_str()).unwrap(),
                        Err(err) => server.send_privmsg(local_event.channel.as_str(), format!("Failed to join channel {}: {}", channel, err).as_str()).unwrap()
                    };
                }),
                from: "IRC Control".to_owned()
            })
        } else {
            Err("Error joining channel".to_owned())
        }
    }
    fn handles_event(&self, event: &CommandEvent) -> bool {
        event.name == "join" && event.args.len() == 1
    }
}
