use crate::endpoints::{ApiData, ApiError, ApiResponse, ResponseFormat, UserAgent};
use chrono::{Duration, Local, Timelike};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct GorgonzolaData {
    pub has_gorgonzola: bool,
    pub message: String,
}

impl ApiData for GorgonzolaData {
    fn message(&self) -> &str {
        &self.message
    }
}

async fn fetch_gorgonzola() -> Result<GorgonzolaData, ApiError> {
    let now = Local::now();
    let date = if now.hour() >= 14 {
        (now + Duration::days(1)).date_naive()
    } else {
        now.date_naive()
    };
    let day_label = if date > now.date_naive() {
        "morgen"
    } else {
        "heute"
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
    let message = format!(
        "Die Mensa hat {} {}Gorgonzola im Angebot.",
        day_label,
        if has_gorgonzola { "" } else { "keinen " }
    )
    .replace("  ", " ");

    Ok(GorgonzolaData {
        has_gorgonzola,
        message,
    })
}

#[openapi(tag = "Mensa")]
#[get("/mensagorgonzola?<format>")]
pub async fn mensagorgonzola(ua: UserAgent, format: Option<String>) -> ApiResponse<GorgonzolaData> {
    let format = ResponseFormat::detect(&ua, format);
    match fetch_gorgonzola().await {
        Ok(data) => ApiResponse::Ok(data, format),
        Err(e) => ApiResponse::Error(e),
    }
}
