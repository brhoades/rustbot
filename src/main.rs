extern crate irc;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate cached;
#[macro_use] extern crate lazy_static;
extern crate futures;
extern crate tokio_core;
extern crate serde_json;
extern crate regex;

mod btc;
mod actions;
mod util;
mod irc_control;
mod command;

use std::default::Default;
use std::thread;
use irc::client::prelude::*;
use irc::client::PackedIrcClient;
use irc::proto::message::Message;
use actions::Action;
use futures::sync::mpsc::{UnboundedSender,UnboundedReceiver,unbounded};
use btc::{CryptoCoinCommand};
use irc_control::{IRCControlCommand};
use command::{CommandHandler,CommandEvent};

fn main() {
    let (command_tx, command_rx): (UnboundedSender<CommandEvent>, UnboundedReceiver<CommandEvent>) = unbounded();
    let (action_tx, action_rx): (UnboundedSender<Action>, UnboundedReceiver<Action>) = unbounded();
    // let server = IrcServer::new("config.toml").unwrap();

    let server = thread::spawn(move || {
        let cfg = Config {
            nickname: Some("rustbot".to_owned()),
            server: Some("irc.wobscale.website".to_owned()),
            channels: Some(vec!["##ircbottesting".to_owned()]),
            use_ssl: Some(true),
            port: Some(6697),
            ..Default::default()
        };

        let mut reactor = tokio_core::reactor::Core::new().unwrap();
        let future = IrcClient::new_future(reactor.handle(), &cfg).unwrap();
        let PackedIrcClient(client, future) = reactor.run(future).unwrap();
        client.identify().unwrap();

        let handle = reactor.handle();
        let client_stream = client.stream();

        // Receive events from command loop.
        handle.spawn(action_rx.for_each(move |mut action| {
            (action.action)(&client);
            Ok(())
        }));

        // Dispatch messages for command processing
        reactor.run(client_stream.for_each(move |message| {
            match handle_message(message) {
                Ok(event) => {
                    command_tx.unbounded_send(event).unwrap();
                },
                _ => ()
            }
            Ok(())
        }).join(future)).unwrap();
    });

    let cmd = thread::spawn(move || { command_loop(command_rx, action_tx) });
    server.join().unwrap();
    cmd.join().unwrap();
}

fn handle_message(message: Message) -> Result<CommandEvent,()> {
    println!("{:?}", message);
    match message.command {
        irc::client::prelude::Command::PRIVMSG(ref channel, ref msg) => {
            if msg.starts_with("!") {
                let command = process_command(channel, msg);
                println!("\t{:?}", command);
                return Ok(command);
            }
        }
        _ => ()
    };
    Err(())
}

fn command_loop(rx: UnboundedReceiver<CommandEvent>, tx: UnboundedSender<Action>) {
    let modules: Vec<Box<CommandHandler>> = vec![
        Box::new(CryptoCoinCommand {}),
        Box::new(IRCControlCommand {})
    ];

    rx.for_each(|event| {
        for module in &modules {
            if module.handles_event(&event) {
                match module.method(&event) {
                    Ok(action) => tx.unbounded_send(action).unwrap(),
                    Err(message) => println!("Error in {:?}: {:?}", module.get_name(), message)
                }
            }
        }
        Ok(())
    }).wait().unwrap();
}

fn process_command(channel: &String, message: &String) -> CommandEvent {
    let mut iter = message.split_whitespace();
    let command = iter.next().unwrap().to_owned().trim_left_matches('!').to_owned();

    CommandEvent {
        message: message.to_owned(),
        name: command.to_owned(),
        args: iter.map(|s: &str| {s.to_owned()}).collect(),
        channel: channel.to_owned()
    }
}

