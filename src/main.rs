#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_okapi;

use rocket::tokio::sync::RwLock;
use rocket_okapi::swagger_ui::*;

mod common;
mod endpoints;

#[launch]
fn rocket() -> _ {
    let port = std::env::var("USEFUL_API_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let address = std::env::var("USEFUL_API_ADDRESS")
        .ok()
        .and_then(|h| h.parse().ok())
        .unwrap_or("0.0.0.0".to_owned());

    let config = rocket::Config::figment()
        .merge(("port", port))
        .merge(("address", address));

    rocket::custom(config)
        .manage(RwLock::new(None::<common::bitcoin::BitcoinPriceCache>))
        .manage(RwLock::new(None::<endpoints::shark::SharkCache>))
        .manage(RwLock::new(None::<endpoints::alditowels::AldiTowelCache>))
        .mount(
            "/",
            openapi_get_routes![
                endpoints::hello::hello,
                endpoints::alditowels::alditowels,
                endpoints::mensatoshi::mensatoshi,
                endpoints::congressbeer::congressbeer,
                endpoints::shark::shark,
                endpoints::mensabeer::mensabeer,
                endpoints::teapot::teapot
            ],
        )
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
}
