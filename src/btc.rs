extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate serde_json;
extern crate hyper_tls;

use std::io;
use self::futures::{Future,Stream};
use self::futures::{future,AndThen};
use self::hyper::{Client,Chunk,Error,Body,Response};
use self::hyper::client::{FutureResponse,HttpConnector};
use self::serde_json::Value;
use self::hyper_tls::HttpsConnector;
use events::CommandEvent;
use irc::client::prelude::*;
use std::fmt::Debug;

fn get_client<F,G>(cb: F)
    where F: Fn(Client<HttpsConnector<HttpConnector>>) -> G,
          G: Future,
          <G as Future>::Error: Debug {

    let mut core = tokio_core::reactor::Core::new().unwrap();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &core.handle()).unwrap())
        .build(&core.handle());

    let work = cb(client);

    match core.run(work) {
        Err(e) => println!("Error fetching API: {:?}", e),
        Ok(_) => ()
    }
}


fn get_url_json<F>(url: String, cb: F)
    where F: Fn(Vec<CryptoCoin>) {

    get_client(|client| {
        let url = url.parse::<hyper::Uri>().unwrap();
        client.get(url).and_then(|res| {
            println!("Response: {}", res.status());

            res.body().concat2().and_then(|body| {
                let v: Vec<CryptoCoin> = serde_json::from_slice(&body).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        e
                    )
                })?;
                cb(v);
                Ok(())
            })
        })
    });
}

pub fn btc_price(command: CommandEvent, server: &IrcServer) {
    let supported_coins = ["btc", "eth", "xrp", "bch", "xlm", "ltc", "iota", "dash", "etc", "usdt"];

    if supported_coins.contains(&command.name.as_str()) {
        get_url_json("https://api.coinmarketcap.com/v1/ticker/".to_owned(), |v: Vec<CryptoCoin>| {
            let symbol = command.name.as_str().to_uppercase();

            let mut iter = v
                .into_iter()
                .filter(|ref x| x.symbol == symbol);
            match iter.next() {
                Some(crypto) => server.send_privmsg(
                    command.channel.as_str(),
                    format!("${} | Change over last: Hour: {}% Day: {}% Week: {}%",
                            crypto.price_usd,
                            crypto.percent_change_1h,
                            crypto.percent_change_24h,
                            crypto.percent_change_7d
                ).as_str()),
                None => server.send_privmsg(command.channel.as_str(), "Unknown Coin")
            };
        });
    }
}

#[derive(Serialize, Deserialize)]
struct CryptoCoin {
    symbol: String,
    name: String,
    rank: String,
    price_usd: String,
    price_btc: String,
    percent_change_1h: String,
    percent_change_24h: String,
    percent_change_7d: String,
    // day_volume_usd: String,
    total_supply: Option<String>,
    max_supply: Option<String>,
    market_cap_usd: String,
    id: String,
    available_supply: String,
}
