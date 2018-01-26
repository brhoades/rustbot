extern crate irc;

use std::default::Default;
use irc::client::prelude::*;

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
    server.for_each_incoming(|message| {
        // Do message processing.
        println!("{:?}", message);
    }).unwrap();
}
