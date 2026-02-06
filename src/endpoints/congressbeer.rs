use crate::common::constants::CONGRESSBEER_SATOSHI;
use crate::endpoints::ApiResponse;
use serde::Serialize;

#[derive(Serialize)]
pub struct CongressBeerData {
    congressbeers: i64,
    message: String,
}

#[get("/congressbeer?<satoshi>&<format>")]
pub fn congressbeer(satoshi: Option<f64>, format: Option<String>) -> ApiResponse<CongressBeerData> {
    let satoshi = satoshi.unwrap_or(CONGRESSBEER_SATOSHI);
    let congressbeers = (satoshi / CONGRESSBEER_SATOSHI).floor() as i64;
    let message = format!(
        "{} Satoshi entspricht {} Bier auf dem Congress.",
        satoshi, congressbeers
    );

    match format.as_deref() {
        Some("json") => ApiResponse::Json(CongressBeerData {
            congressbeers,
            message,
        }),
        _ => ApiResponse::Plain(message),
    }
}
