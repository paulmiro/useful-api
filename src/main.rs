#[macro_use]
extern crate rocket;

use rocket::tokio::sync::RwLock;

mod common;
mod endpoints;

use common::bitcoin::BitcoinPriceCache;
use endpoints::{
    congressbeer::congressbeer,
    hello::hello,
    mensabeer::mensabeer,
    mensatoshi::mensatoshi,
    shark::{SharkCache, shark},
    teapot::teapot,
    random::random,
};

// Central list of route paths used by the API.
// This can be consumed by other endpoints (e.g. /random) to avoid
// duplicating the list of valid routes.
pub fn all_routes() -> &'static [&'static str] {
    &[
        "/",            // hello
        "/mensabeer",
        "/mensatoshi",
        "/congressbeer",
        "/shark",
        "/teapot",
        "/random",
    ]
}

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
        .manage(RwLock::new(None::<BitcoinPriceCache>))
        .manage(RwLock::new(None::<SharkCache>))
        .mount(
            "/",
            routes![hello, mensatoshi, congressbeer, shark, mensabeer, teapot, random],
        )
}
