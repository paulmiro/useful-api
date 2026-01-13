#[macro_use]
extern crate rocket;

use rocket::tokio::sync::Mutex;

mod endpoints;

use endpoints::{
    congressbeer::congressbeer,
    hello::hello,
    mensatoshi::{SatoshiPriceCache, mensatoshi},
    shark::shark,
};

#[launch]
fn rocket() -> _ {
    let port = std::env::var("USEFUL_API_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let config = rocket::Config::figment()
        .merge(("port", port))
        .merge(("address", "0.0.0.0"));

    rocket::custom(config)
        .manage(Mutex::new(None::<SatoshiPriceCache>))
        .manage(reqwest::Client::new())
        .mount("/", routes![hello, mensatoshi, congressbeer, shark])
}
