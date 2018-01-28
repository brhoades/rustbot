extern crate hyper;
extern crate hyper_tls;

use futures::{Future,Stream};
use self::hyper::{Client,Error};
use self::hyper_tls::HttpsConnector;
use tokio_core::{reactor};
use cached::TimedCache;
use serde_json;

cached!{ SLOW_FN: TimedCache = TimedCache::with_lifespan_and_capacity(180,10); >>
         fn get_url_json(url: String) -> Result<serde_json::Value,String> = {
             let url = url.parse::<hyper::Uri>().unwrap();
             let mut core = reactor::Core::new().unwrap();
             let client = Client::configure()
                 .connector(HttpsConnector::new(4, &core.handle()).unwrap())
                 .build(&core.handle());

             let work = client.get(url).and_then(|res| {
                 println!("Response: {}", res.status());

                 res.body().concat2().and_then(|body| {
                     let raw_data: Result<serde_json::Value, _> = serde_json::from_slice(&body);
                     println!("{:?}", raw_data);

                     match raw_data {
                         Ok(data) => Ok(data),
                         Err(e) => {
                             println!("Error getting API {:?}", e);
                             Err(Error::Status)
                         }
                     }
                 })
             });

             core.run(work).map_err(|_| "Error processing JSON".to_owned())
         }
}
