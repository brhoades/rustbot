extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate serde_json;
extern crate hyper_tls;

use std::io;
use self::futures::{Future,Stream};
use self::futures::future;
use self::hyper::{Client,Chunk,Error};
use self::serde_json::Value;
use self::hyper_tls::HttpsConnector;
use events::CommandEvent;
use irc::client::prelude::*;


fn get_url<F>(url: String, cb: F)
    where F: Fn(Value) {
    let url = url.parse::<hyper::Uri>().unwrap();

    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &core.handle()).unwrap())
        .build(&core.handle());

    let work = client.get(url).and_then(|res| {
        println!("Response: {}", res.status());

        res.body().concat2().and_then(move |body| {
            let v: Value = serde_json::from_slice(&body).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    e
                )
            })?;
            println!("Value: {:?}", v);
            cb(v);

            Ok(())
        })
    });

    match core.run(work) {
        Err(e) => println!("Error fetching API: {:?}", e),
        Ok(_) => ()
    }
}

pub fn btc_price(command: CommandEvent, server: &IrcServer) {
    if command.name == "!btc" {
        get_url("https://blockchain.info/ticker".to_owned(), |v: Value| {
            server.send_privmsg(command.channel.as_str(), format!("{}", v["USD"]["15m"]).as_str());
        });
    }
}
