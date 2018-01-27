extern crate irc;
mod btc;
mod events;

use std::default::Default;
use irc::client::prelude::*;
use irc::proto::message::Message;
use events::CommandEvent;

fn main() {
    let cfg = Config {
        nickname: Some("rustbot".to_owned()),
        server: Some("irc.wobscale.website".to_owned()),
        channels: Some(vec!["##ircbottesting".to_owned()]),
        use_ssl: Some(true),
        port: Some(6697),
        ..Default::default()
    };
    // let server = IrcServer::new("config.toml").unwrap();
    let server = IrcServer::from_config(cfg).unwrap();
    println!("Connecting...");
    server.identify().unwrap();
    println!("Ready!");
    server.for_each_incoming(|msg| {
        handle_message(&server, msg);
    }).unwrap();
}

fn handle_message(server: &IrcServer, message: Message) {
    println!("{:?}", message);
    match message.command {
        Command::PRIVMSG(ref channel, ref msg) => {
            if msg.starts_with("!") {
                let command = process_command(channel, msg);
                btc::btc_price(command.clone(), &server);
                println!("\t{:?}", command);
            }
        }
        _ => ()
    }
}

fn process_command(channel: &String, message: &String) -> CommandEvent {
    let mut iter = message.split_whitespace();
    let command = iter.next().unwrap();

    CommandEvent {
        message: message.to_owned(),
        name: command.to_owned(),
        args: iter.map(|s: &str| {s.to_owned()}).collect(),
        channel: channel.to_owned()
    }
}

