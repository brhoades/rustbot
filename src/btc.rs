use std::boxed::Box;
use events::CommandEvent;
use actions::Action;
use irc::client::prelude::*;
use futures::sync::mpsc::{UnboundedSender};
use util::get_url_json;
use serde_json;

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

pub fn btc_price(event: &CommandEvent, tx: &UnboundedSender<Action>) -> bool {
    let supported_coins = ["btc", "eth", "xrp", "bch", "xlm", "ltc", "iota", "dash", "etc", "usdt"];

    if supported_coins.contains(&event.name.as_str()) {
        match get_url_json("https://api.coinmarketcap.com/v1/ticker/".to_owned()) {
            Ok(raw_data) => {
                // FIXME: Can't find out how to convert from a value so... convert back and again
                let raw_data =  &serde_json::to_string(&raw_data).unwrap().to_owned();
                let data: Vec<CryptoCoin> = serde_json::from_str(raw_data).unwrap();

                let response = get_response_msg(data, event.name.as_str().to_uppercase());
                let local_event = event.clone();
                tx.unbounded_send(Action {
                    action: Box::new(move |server: &IrcClient| {
                        server.send_privmsg(local_event.channel.as_str(), response.as_str()).unwrap();
                    }),
                    from: "cryptocoin".to_owned()
                }).unwrap();
            },
            Err(_) => println!("Error communicating with coinmarketcap.")
        }
        true
    } else {
        false
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CryptoCoin {
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
