extern crate hyper;
extern crate hyper_tls;

use std::io;
use futures::{Future,Stream};
use self::hyper::{Client};
use self::hyper_tls::HttpsConnector;
use tokio_core::{reactor};
use cached::TimedCache;
use serde_json;

cached!{ SLOW_FN: TimedCache = TimedCache::with_lifespan_and_capacity(180,10); >>
         fn get_url_json(url: String) -> Result<String,()> = {
             let url = url.parse::<hyper::Uri>().unwrap();
             let mut core = reactor::Core::new().unwrap();
             let client = Client::configure()
                 .connector(HttpsConnector::new(4, &core.handle()).unwrap())
                 .build(&core.handle());

             let work = client.get(url).and_then(|res| {
                 res.body().concat2().and_then(|body| {
                     let raw_data = serde_json::from_slice(&body).map_err(|e| {
                         io::Error::new(
                             io::ErrorKind::Other,
                             e
                         )
                     }).unwrap();

                     Ok(raw_data)
                 })
             });

             match core.run(work) {
                 Ok(v) => Ok(v),
                 Err(_) => Err(())
             }
         }
}
