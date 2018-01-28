use std::boxed::Box;
use irc::client::prelude::*;
use events::CommandEvent;
use actions::Action;
use futures::sync::mpsc::{UnboundedSender};

pub fn command(event: &CommandEvent, tx: &UnboundedSender<Action>) -> bool {
    let supported_events = ["join", "part"];
    let name = event.name.as_str();
    let channel = event.args[0].clone();

    if supported_events.contains(&name) {
        if name == "join" && event.args.len() == 1 {
            let local_event = event.clone();
            tx.unbounded_send(Action {
                action: Box::new(move |server: &IrcClient| {
                    match server.send_join(channel.as_str()) {
                        Ok(()) => server.send_privmsg(local_event.channel.as_str(), format!("Joined channel {}", channel).as_str()).unwrap(),
                        Err(err) => server.send_privmsg(local_event.channel.as_str(), format!("Failed to join channel {}: {}", channel, err).as_str()).unwrap()
                    };
                }),
                from: "IRC Control".to_owned()
            }).unwrap();
        }
        true
    } else {
        false
    }
}
