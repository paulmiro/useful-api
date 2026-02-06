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

    let config = rocket::Config::figment()
        .merge(("port", port))
        .merge(("address", "0.0.0.0"));

    rocket::custom(config)
        .manage(RwLock::new(None::<common::bitcoin::BitcoinPriceCache>))
        .manage(RwLock::new(None::<endpoints::shark::SharkCache>))
        .mount(
            "/",
            openapi_get_routes![
                endpoints::hello::hello,
                endpoints::mensatoshi::mensatoshi,
                endpoints::congressbeer::congressbeer,
                endpoints::shark::shark,
                endpoints::mensabeer::mensabeer,
                endpoints::teapot::teapot
            ],
        )
        //.mount("/", routes![openapi_json])
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
}
