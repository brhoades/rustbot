extern crate hyper;
extern crate serde_json;
extern crate hyper_tls;

use std::io;
use std::fmt::Debug;
use std::boxed::Box;
use events::CommandEvent;
use actions::Action;
use irc::client::prelude::*;
use futures::{Future,Stream};
use self::hyper::{Client};
use self::hyper::client::{HttpConnector};
use self::hyper_tls::HttpsConnector;
use tokio_core::{reactor};
use chan::{Sender};

fn get_client<F,G>(cb: F)
    where F: Fn(Client<HttpsConnector<HttpConnector>>) -> G,
          G: Future,
          <G as Future>::Error: Debug {

    let mut core = reactor::Core::new().unwrap();
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

pub fn btc_price(event: CommandEvent, tx: Sender<Action>) {
    let supported_coins = ["btc", "eth", "xrp", "bch", "xlm", "ltc", "iota", "dash", "etc", "usdt"];

    if supported_coins.contains(&event.name.as_str()) {
        get_url_json("https://api.coinmarketcap.com/v1/ticker/".to_owned(), |v: Vec<CryptoCoin>| {
            let symbol = event.name.as_str().to_uppercase();
            let response = get_response_msg(v, symbol);
            let event_inner = event.clone();
            tx.send(Action {
                action: Box::new(move |server: IrcClient| {
                    server.send_privmsg(event_inner.channel.as_str(), response.clone().as_str()).unwrap();
                }),
                from: "cryptocoin".to_owned()
            });
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
