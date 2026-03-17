use crate::endpoints::{ApiError, ApiResponse};
use chrono::{Duration, Local, Timelike};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct GorgonzolaData {
    pub has_gorgonzola: bool,
}

async fn fetch_gorgonzola() -> Result<GorgonzolaData, ApiError> {
    let now = Local::now();
    let date = if now.hour() >= 14 {
        (now + Duration::days(1)).date_naive()
    } else {
        now.date_naive()
    };

    let url = format!(
        "https://openmensa.alexanderwallau.de/CAMPO/{}",
        date.format("%Y-%m-%d")
    );

    let body = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| ApiError {
            message: format!("Failed to fetch mensa menu: {}", e),
        })?
        .text()
        .await
        .map_err(|e| ApiError {
            message: format!("Failed to read mensa menu: {}", e),
        })?;

    let has_gorgonzola = body.to_lowercase().contains("gorgonzola");

    Ok(GorgonzolaData { has_gorgonzola })
}

#[openapi(tag = "Mensa")]
#[get("/mensa-gorgonzola?<format>")]
pub async fn mensa_gorgonzola(format: Option<String>) -> ApiResponse<GorgonzolaData> {
    match fetch_gorgonzola().await {
        Ok(data) => match format.as_deref() {
            Some("json") => ApiResponse::Json(data),
            _ => ApiResponse::Plain(data.has_gorgonzola.to_string()),
        },
        Err(e) => ApiResponse::Error(e),
    }
}
