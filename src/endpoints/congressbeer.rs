use crate::api::{ApiData, ApiResponse, ResponseFormat, UserAgent};
use crate::common::constants::CONGRESSBEER_SATOSHI;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct CongressBeerData {
    pub beers: i64,
    pub message: String,
}

impl ApiData for CongressBeerData {
    fn message(&self) -> &str {
        &self.message
    }
}

#[openapi(tag = "Conversion")]
#[get("/congressbeer?<satoshi>&<format>")]
pub fn congressbeer(
    ua: UserAgent,
    satoshi: Option<f64>,
    format: Option<String>,
) -> ApiResponse<CongressBeerData> {
    let satoshi = satoshi.unwrap_or(CONGRESSBEER_SATOSHI);
    let beers = (satoshi / CONGRESSBEER_SATOSHI).floor() as i64;
    let message = format!("{satoshi} Satoshi entspricht **{beers}** Bier auf dem Congress!",);

    let format = ResponseFormat::detect(&ua, format);
    ApiResponse::Ok(CongressBeerData { beers, message }, format)
}
