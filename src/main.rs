use axum::{Router, routing::get};
use axum::debug_handler;
use serde::Deserialize;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};
use lazy_static::lazy_static;

lazy_static! {
    static ref CACHE: Mutex<(Option<f64>, Option<Instant>)> = Mutex::new((None, None));
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
    let (cached_price, last_fetch) = *cache;

    if let (Some(price), Some(fetch_time)) = (cached_price, last_fetch) {
        if fetch_time.elapsed() < Duration::from_secs(300) {
            let satoshi_per_btc = 100_000_000.0;
            let satoshi_per_eur = satoshi_per_btc / price;
            let mensa_price_eur = 1.20;
            let mensasatoshi = mensa_price_eur * satoshi_per_eur;
            return mensasatoshi.round().to_string();
        }
    }

    let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=eur";
    match reqwest::get(url).await {
        Ok(response) => match response.json::<CoinGeckoResponse>().await {
            Ok(data) => {
                let eur_per_btc = data.bitcoin.eur;
                *cache = (Some(eur_per_btc), Some(Instant::now()));
                let satoshi_per_btc = 100_000_000.0;
                let satoshi_per_eur = satoshi_per_btc / eur_per_btc;
                let mensa_price_eur = 1.20;
                let mensasatoshi = mensa_price_eur * satoshi_per_eur;
                mensasatoshi.round().to_string()
            }
            Err(_) => "Error deserializing response".to_string(),
        },
        Err(_) => "Error fetching data from CoinGecko".to_string(),
    }
}
