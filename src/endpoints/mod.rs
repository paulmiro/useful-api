pub mod congressbeer;
pub mod hello;
pub mod mensabeer;
pub mod mensatoshi;
pub mod shark;
pub mod teapot;
pub mod random;

use rocket::http::ContentType;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket::serde::json::Json;
use serde::Serialize;
use std::io::Cursor;

#[derive(Serialize)]
pub struct ApiError {
    pub message: String,
}

pub enum ApiResponse<T> {
    Plain(String),
    Json(T),
    Error(ApiError),
}

impl<'r, T: Serialize> Responder<'r, 'static> for ApiResponse<T> {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        match self {
            ApiResponse::Plain(text) => Response::build()
                .header(ContentType::Plain)
                .sized_body(text.len(), Cursor::new(text))
                .ok(),
            ApiResponse::Json(data) => Json(data).respond_to(req),
            ApiResponse::Error(error) => Json(error).respond_to(req),
        }
    }
}
