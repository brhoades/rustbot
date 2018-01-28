extern crate hyper;
extern crate serde_json;
extern crate hyper_tls;

use std::io;
use std::boxed::Box;
use events::CommandEvent;
use actions::Action;
use irc::client::prelude::*;
use futures::{Future,Stream};
use self::hyper::{Client};
use self::hyper_tls::HttpsConnector;
use tokio_core::{reactor};
use futures::sync::mpsc::{UnboundedSender};

fn get_url_json(url: String) -> hyper::Result<Vec<CryptoCoin>> {
    let url = url.parse::<hyper::Uri>().unwrap();
    let mut core = reactor::Core::new().unwrap();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &core.handle()).unwrap())
        .build(&core.handle());

    let work = client.get(url).and_then(|res| {
        res.body().concat2().and_then(|body| {
            let v: Vec<CryptoCoin> = serde_json::from_slice(&body).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    e
                )
            }).unwrap();
            Ok(v)
        })
    });

    core.run(work)
}

fn get_response_msg(data: Vec<CryptoCoin>, symbol: String) -> String {
    let mut iter = data
        .into_iter()
        .filter(|ref x| x.symbol == symbol);
    match iter.next() {
        Some(crypto) => 
            format!("${} | Change over last: Hour: {}% Day: {}% Week: {}%",
                    crypto.price_usd,
                    crypto.percent_change_1h,
                    crypto.percent_change_24h,
                    crypto.percent_change_7d
            ),
        None => "Unknown Coin".to_owned()
    }
}

pub fn btc_price(event: CommandEvent, tx: UnboundedSender<Action>) -> bool {
    let supported_coins = ["btc", "eth", "xrp", "bch", "xlm", "ltc", "iota", "dash", "etc", "usdt"];

    if supported_coins.contains(&event.name.as_str()) {
        match get_url_json("https://api.coinmarketcap.com/v1/ticker/".to_owned()) {
            Ok(data) => {
                let response = get_response_msg(data, event.name.as_str().to_uppercase());
                let event_inner = event.clone();
                tx.unbounded_send(Action {
                    action: Box::new(move |server: &IrcClient| {
                        server.send_privmsg(event_inner.channel.as_str(), response.clone().as_str()).unwrap();
                    }),
                    from: "cryptocoin".to_owned()
                }).unwrap();
            },
            Err(_) => println!("Error hitting coinmarketcap API.")
        }
        true
    } else {
        false
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
    day_volume_usd: Option<String>,
    total_supply: Option<String>,
    max_supply: Option<String>,
    market_cap_usd: String,
    id: String,
    available_supply: String,
}
