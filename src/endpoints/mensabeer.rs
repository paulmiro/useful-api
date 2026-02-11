use crate::common::bitcoin::{Cache, get_price};
use crate::common::constants::{CONGRESSBEER_SATOSHI, MENSA_EINTOPF_EUR};
use crate::endpoints::ApiResponse;
use rocket::State;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct MensaBeerData {
    beers: f64,
    message: String,
}

#[openapi(tag = "Conversion")]
#[get("/mensabeer?<format>")]
pub async fn mensabeer(
    cache_state: &State<Cache>,
    format: Option<String>,
) -> ApiResponse<MensaBeerData> {
    let eur_per_btc = match get_price(cache_state).await {
        Ok(price) => price,
        Err(e) => return ApiResponse::Error(e),
    };
    let satoshi_per_eur = 100_000_000.0 / eur_per_btc;

    let mensa_satoshi = MENSA_EINTOPF_EUR * satoshi_per_eur;
    let beers = (mensa_satoshi / CONGRESSBEER_SATOSHI).floor();

    let message = format!(
        "Für den Preis eines Mensa-Eintopfs bekommt man aktuell {} Bier auf dem Congress.",
        beers
    );

    match format.as_deref() {
        Some("json") => ApiResponse::Json(MensaBeerData { beers, message }),
        _ => ApiResponse::Plain(message),
    }
}
