use crate::common::constants::CONGRESSBEER_SATOSHI;
use crate::endpoints::{ApiData, ApiResponse, ResponseFormat, UserAgent};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct CongressBeerData {
    pub congressbeers: i64,
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
    let congressbeers = (satoshi / CONGRESSBEER_SATOSHI).floor() as i64;
    let message = format!(
        "{} Satoshi entspricht {} Bier auf dem Congress.",
        satoshi, congressbeers
    );

    let format = ResponseFormat::detect(&ua, format);
    ApiResponse::Ok(
        CongressBeerData {
            congressbeers,
            message,
        },
        format,
    )
}
