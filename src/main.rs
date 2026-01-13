#[macro_use]
extern crate rocket;

use rocket::State;
use rocket::tokio::sync::Mutex;
use serde::Deserialize;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
struct SatoshiPriceCache {
    price: f64,
    time: Instant,
}

#[derive(Deserialize)]
struct CoinGeckoResponse {
    bitcoin: Bitcoin,
}

#[derive(Deserialize)]
struct Bitcoin {
    eur: f64,
}

type Cache = Mutex<Option<SatoshiPriceCache>>;

#[get("/")]
fn hello() -> &'static str {
    "Hello, World!"
}

#[get("/mensatoshi")]
async fn mensatoshi(cache_state: &State<Cache>) -> String {
    let mut cache = cache_state.lock().await;

    let satoshi_per_eur = match *cache {
        Some(price_cache) if price_cache.time.elapsed() < Duration::from_secs(10) => {
            price_cache.price
        }
        _ => {
            let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=eur";
            let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
            let client = match reqwest::Client::builder().user_agent(user_agent).build() {
                Ok(client) => client,
                Err(_) => return "Error creating HTTP client".to_string(),
            };

            let response = match client.get(url).send().await {
                Ok(response) => response,
                Err(_) => return "Error fetching data from CoinGecko".to_string(),
            };
            match response.json::<CoinGeckoResponse>().await {
                Ok(data) => {
                    let eur_per_btc = data.bitcoin.eur;
                    let satoshi_per_btc: f64 = 100_000_000.0;
                    let satoshi_per_eur = satoshi_per_btc / eur_per_btc;
                    *cache = Some(SatoshiPriceCache {
                        price: satoshi_per_eur,
                        time: Instant::now(),
                    });
                    satoshi_per_eur
                }
                Err(_) => {
                    return "Error deserializing CoinGecko response. Probably rate limited."
                        .to_string();
                }
            }
        }
    };

    let mensa_price_eur = 1.20; // TOOO: fetch dynamically
    let mensasatoshi = mensa_price_eur * satoshi_per_eur;
    format!(
        "Der Mensa-Eintopf kostet aktuell {} Satoshi.",
        mensasatoshi.round()
    )
}

#[get("/congressbeer?<satoshi>")]
fn congressbeer(satoshi: f64) -> String {
    const CONGRESSBEER_SATOSHI: f64 = 69.0;
    let congressbeers = (satoshi / CONGRESSBEER_SATOSHI).floor() as i64;
    format!(
        "{} Satoshi entspricht {} Congressbeers.",
        satoshi,
        congressbeers
    )
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
        .manage(Mutex::new(None::<SatoshiPriceCache>))
        .mount("/", routes![hello, mensatoshi, congressbeer])
}