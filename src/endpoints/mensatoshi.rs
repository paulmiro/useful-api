use crate::common::bitcoin::{Cache, get_price};
use crate::common::constants::MENSA_EINTOPF_EUR;
use crate::endpoints::ApiResponse;
use rocket::State;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct MensaSatoshiData {
    satoshi: f64,
    message: String,
}

#[openapi(tag = "Conversion")]
#[get("/mensatoshi?<format>")]
pub async fn mensatoshi(
    cache_state: &State<Cache>,
    format: Option<String>,
) -> ApiResponse<MensaSatoshiData> {
    let eur_per_btc = match get_price(cache_state).await {
        Ok(price) => price,
        Err(e) => return ApiResponse::Error(e),
    };
    let satoshi_per_eur = 100_000_000.0 / eur_per_btc;

    let mensasatoshi = MENSA_EINTOPF_EUR * satoshi_per_eur;
    let rounded_satoshi = mensasatoshi.round();
    let message = format!(
        "Der Mensa-Eintopf kostet aktuell {} Satoshi.",
        rounded_satoshi
    );

    match format.as_deref() {
        Some("json") => ApiResponse::Json(MensaSatoshiData {
            satoshi: rounded_satoshi,
            message,
        }),
        _ => ApiResponse::Plain(message),
    }
}
