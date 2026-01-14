use crate::endpoints::ApiError;
use rocket::State;
use rocket::tokio::sync::RwLock;
use serde::Deserialize;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct BitcoinPriceCache {
    pub price: f64,
    pub time: Instant,
}

pub type Cache = RwLock<Option<BitcoinPriceCache>>;

#[derive(Deserialize)]
struct CoinGeckoResponse {
    bitcoin: Bitcoin,
}

#[derive(Deserialize)]
struct Bitcoin {
    eur: f64,
}

async fn fetch_price() -> Result<f64, ApiError> {
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=eur";
    let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .build()
        .map_err(|_| ApiError {
            message: "Error creating HTTP client".to_string(),
        })?;

    let response = client.get(url).send().await.map_err(|_| ApiError {
        message: "Error fetching data from CoinGecko".to_string(),
    })?;

    let data = response
        .json::<CoinGeckoResponse>()
        .await
        .map_err(|_| ApiError {
            message: "Error deserializing CoinGecko response. Probably rate limited.".to_string(),
        })?;

    let eur_per_btc = data.bitcoin.eur;
    Ok(eur_per_btc)
}

pub async fn get_price(cache_state: &State<Cache>) -> Result<f64, ApiError> {
    // First, try to get a read lock.
    {
        let cache = cache_state.read().await;
        if let Some(price_cache) = *cache {
            if price_cache.time.elapsed() < Duration::from_secs(10) {
                return Ok(price_cache.price);
            }
        }
    }

    // If the cache is stale or empty, get a write lock to update it.
    let mut cache = cache_state.write().await;

    // We need to check again in case another request has already updated the cache
    // while we were waiting for the write lock.
    if let Some(price_cache) = *cache {
        if price_cache.time.elapsed() < Duration::from_secs(10) {
            return Ok(price_cache.price);
        }
    }

    let price = fetch_price().await?;

    *cache = Some(BitcoinPriceCache {
        price,
        time: Instant::now(),
    });

    Ok(price)
}
