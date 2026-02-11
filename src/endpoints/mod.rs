pub mod congressbeer;
pub mod hello;
pub mod mensabeer;
pub mod mensatoshi;
pub mod shark;
pub mod teapot;

use rocket::http::ContentType;
use rocket::request::Request;
use rocket::response::{self, Responder, Response as RocketResponse};
use rocket::serde::json::Json;
use rocket_okapi::OpenApiError;
use rocket_okapi::r#gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::{MediaType, Response, Responses};
use rocket_okapi::response::OpenApiResponder;
use schemars::JsonSchema;
use serde::Serialize;
use std::io::Cursor;

#[derive(Serialize, JsonSchema)]
pub struct ApiError {
    pub message: String,
}

pub enum ApiResponse<T> {
    Plain(String),
    Json(T),
    Error(ApiError),
}

impl<'r, T: Serialize + JsonSchema> OpenApiResponder<'r, 'static> for ApiResponse<T> {
    fn responses(r#gen: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Responses::default();

        let json_schema = r#gen.json_schema::<T>();
        let plain_schema = r#gen.json_schema::<String>();
        // ApiError schema could be added too

        let mut content = rocket_okapi::okapi::Map::new();

        content.insert(
            "application/json".to_string(),
            MediaType {
                schema: Some(json_schema),
                ..MediaType::default()
            },
        );

        content.insert(
            "text/plain".to_string(),
            MediaType {
                schema: Some(plain_schema),
                ..MediaType::default()
            },
        );

        responses.responses.insert(
            "200".to_string(),
            rocket_okapi::okapi::openapi3::RefOr::Object(Response {
                description: "Successful response".to_string(),
                content,
                ..Response::default()
            }),
        );

        Ok(responses)
    }
}

impl<'r, T: Serialize> Responder<'r, 'static> for ApiResponse<T> {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        match self {
            ApiResponse::Plain(text) => RocketResponse::build()
                .header(ContentType::Plain)
                .sized_body(text.len(), Cursor::new(text))
                .ok(),
            ApiResponse::Json(data) => Json(data).respond_to(req),
            ApiResponse::Error(error) => Json(error).respond_to(req),
        }
    }
}
