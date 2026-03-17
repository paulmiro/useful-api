use crate::endpoints::{ApiData, ApiResponse, ResponseFormat, UserAgent};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct HelloData {
    pub message: String,
}

impl ApiData for HelloData {
    fn message(&self) -> &str {
        &self.message
    }
}

#[openapi(tag = "Hello")]
#[get("/?<format>")]
pub fn hello(ua: UserAgent, format: Option<String>) -> ApiResponse<HelloData> {
    let msg = "Hello, World!";
    let data = HelloData {
        message: msg.to_string(),
    };
    let format = ResponseFormat::detect(&ua, format);
    ApiResponse::Ok(data, format)
}
