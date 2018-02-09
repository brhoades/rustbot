use std::boxed::Box;
use actions::Action;
use irc::client::prelude::*;
use util::get_url_json;
use serde_json;
use command::{CommandHandler,CommandError,CommandEvent};

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



pub struct CryptoCoinCommand;

impl CommandHandler for CryptoCoinCommand {
    fn get_name(&self) -> &'static str {
        "Crypto Coin Lookup"
    }

    fn method(&self, event: &CommandEvent) -> Result<Action,CommandError> {
       match get_url_json("https://api.coinmarketcap.com/v1/ticker/".to_owned()) {
            Ok(raw_data) => {
                // FIXME: Can't find out how to convert from a value so... convert back and again
                let raw_data =  &serde_json::to_string(&raw_data).unwrap().to_owned();
                let data: Vec<CryptoCoin> = serde_json::from_str(raw_data).unwrap();

                let response = get_response_msg(data, event.name.as_str().to_uppercase());
                let local_event = event.clone();
                Ok(Action {
                    action: Box::new(move |server: &IrcClient| {
                        server.send_privmsg(local_event.channel.as_str(), response.as_str()).unwrap();
                    }),
                    from: "cryptocoin".to_owned()
                })
            },
            Err(_) => Err("Error communicating with coinmarketcap.".to_owned())
        }
    }

    fn handles_event(&self, event: &CommandEvent) -> bool{
        // TODO: Cache
        let supported_commands = vec!["btc", "eth", "xrp", "bch", "xlm", "ltc", "iota", "dash", "etc", "usdt"];
        supported_commands.contains(&event.name.as_str())
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
