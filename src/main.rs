use axum::{Router, routing::get};
use serde::Deserialize;

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
    axum::serve(listener, app).await.unwrap();
}

async fn hello_handler() -> String {
    "Hello, World!".to_string()
}

async fn mensatoshi_handler() -> String {
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=eur";
    match reqwest::get(url).await {
        Ok(response) => match response.json::<CoinGeckoResponse>().await {
            Ok(data) => {
                let eur_per_btc = data.bitcoin.eur;
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
