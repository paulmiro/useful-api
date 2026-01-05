use axum::debug_handler;
use axum::{Router, routing::get};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

lazy_static! {
    static ref CACHE: Mutex<Option<SatoshiPriceCache>> = Mutex::new(None);
}

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

#[tokio::main]
async fn main() {
    // build our application with two routes
    let app = Router::new()
        .route("/", get(hello_handler))
        .route("/mensatoshi", get(mensatoshi_handler));

    // run it with hyper on localhost:3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:19190")
        .await
        .unwrap();

    println!("Server running on http://0.0.0.0:19190");

    axum::serve(listener, app).await.unwrap();
}

#[debug_handler]
async fn hello_handler() -> String {
    "Hello, World!".to_string()
}

#[debug_handler]
async fn mensatoshi_handler() -> String {
    let mut cache = CACHE.lock().await;

    let satoshi_per_eur = match *cache {
        Some(price_cache) if price_cache.time.elapsed() < Duration::from_secs(10) => {
            price_cache.price
        }
        _ => {
            let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=eur";
            let response = match reqwest::get(url).await {
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
    return format!(
        "Der Mensa-Eintopf kostet aktuell {} Satoshi.",
        mensasatoshi.round()
    );
}
