use crate::endpoints::ApiResponse;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct HelloData {
    message: String,
}

#[openapi(tag = "Hello")]
#[get("/?<format>")]
pub fn hello(format: Option<String>) -> ApiResponse<HelloData> {
    let msg = "Hello, World!";
    match format.as_deref() {
        Some("json") => ApiResponse::Json(HelloData {
            message: msg.to_string(),
        }),
        _ => ApiResponse::Plain(msg.to_string()),
    }
}
