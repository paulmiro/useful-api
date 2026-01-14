#[macro_use]
extern crate rocket;

use rocket::tokio::sync::RwLock;

mod endpoints;

use endpoints::{
    congressbeer::congressbeer,
    hello::hello,
    mensatoshi::{SatoshiPriceCache, mensatoshi},
    shark::{SharkCache, shark},
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
        .manage(RwLock::new(None::<SatoshiPriceCache>))
        .manage(RwLock::new(None::<SharkCache>))
        .mount("/", routes![hello, mensatoshi, congressbeer, shark])
}
