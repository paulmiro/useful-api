use rocket::State;
use rocket::tokio::sync::Mutex;
use serde::Deserialize;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct SatoshiPriceCache {
    pub price: f64,
    pub time: Instant,
}

pub type Cache = Mutex<Option<SatoshiPriceCache>>;

#[derive(Deserialize)]
struct CoinGeckoResponse {
    bitcoin: Bitcoin,
}

#[derive(Deserialize)]
struct Bitcoin {
    eur: f64,
}

#[get("/mensatoshi")]
pub async fn mensatoshi(cache_state: &State<Cache>) -> String {
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
