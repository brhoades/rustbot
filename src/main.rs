extern crate irc;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate chan;
extern crate futures;
extern crate tokio_core;

mod btc;
mod events;
mod actions;

use std::default::Default;
use std::thread;
use irc::client::prelude::*;
use irc::client::PackedIrcClient;
use irc::proto::message::Message;
use events::CommandEvent;
use actions::Action;

fn main() {
    let (command_tx, command_rx): (chan::Sender<CommandEvent>, chan::Receiver<CommandEvent>) = chan::async();
    let (action_tx, action_rx): (chan::Sender<Action>, chan::Receiver<Action>) = chan::async();
    // let server = IrcServer::new("config.toml").unwrap();
    let tick = chan::tick_ms(100);

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
        let handle = reactor.handle();
        let future = IrcClient::new_future(reactor.handle(), &cfg).unwrap();
        let PackedIrcClient(client, future) = reactor.run(future).unwrap();
        client.identify().unwrap();

        handle.spawn_fn(move || {
            chan_select! {
                default => println!("Timeout"),
                action_rx.recv() -> action => println!("ACTION RECEIVED {:?}", action),
            }
            Ok(())
        });
        reactor.run(client.stream().for_each(move |message| {
            match handle_message(message) {
                Ok(event) => command_tx.send(event),
                Err(()) => ()
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
        Command::PRIVMSG(ref channel, ref msg) => {
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

fn command_loop(rx: chan::Receiver<CommandEvent>, tx: chan::Sender<Action>) {
    let command = rx.recv();

    match command {
        Some(event) => btc::btc_price(event.clone(), tx),
        None => ()
    }
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

