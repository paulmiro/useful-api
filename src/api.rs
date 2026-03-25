use crate::rendering::{html, markdown, plaintext, shell};
use rocket::http::ContentType;
use rocket::request::Request;
use rocket::response::{self, Responder, Response as RocketResponse};
use rocket::serde::json::Json;
use rocket_okapi::OpenApiError;
use rocket_okapi::r#gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::{MediaType, Response, Responses};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use rocket_okapi::response::OpenApiResponder;
use serde::Serialize;
use std::io::Cursor;

#[derive(Serialize, JsonSchema)]
pub struct ApiError {
    pub message: String,
}

pub struct UserAgent(pub String);

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for UserAgent {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        rocket::request::Outcome::Success(UserAgent(
            req.headers()
                .get_one("User-Agent")
                .unwrap_or("")
                .to_string(),
        ))
    }
}

impl<'a> OpenApiFromRequest<'a> for UserAgent {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> Result<RequestHeaderInput, OpenApiError> {
        Ok(RequestHeaderInput::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseFormat {
    Json,
    Markdown,
    Plain,
    Html,
    Shell,
}

impl ResponseFormat {
    pub fn detect(ua: &UserAgent, format_param: Option<String>) -> Self {
        if let Some(f) = format_param {
            match f.to_lowercase().as_str() {
                "json" => return ResponseFormat::Json,
                "markdown" | "md" => return ResponseFormat::Markdown,
                "html" => return ResponseFormat::Html,
                "shell" | "sh" => return ResponseFormat::Shell,
                "plaintext" | "plain" | "text" => return ResponseFormat::Plain,
                _ => {}
            }
        }

        let user_agent = ua.0.to_lowercase();

        if user_agent.contains("mozilla") {
            ResponseFormat::Html
        } else if is_terminal_client(&user_agent) {
            ResponseFormat::Shell
        } else {
            ResponseFormat::Plain
        }
    }
}

fn is_terminal_client(user_agent: &str) -> bool {
    ["curl/", "wget/", "httpie/", "xh/", "powershell/"]
        .iter()
        .any(|marker| user_agent.contains(marker))
}

pub trait ApiData: Serialize + JsonSchema {
    fn message(&self) -> &str;
}

pub enum ApiResponse<T: ApiData> {
    Ok(T, ResponseFormat),
    Error(ApiError),
}

impl<'r, T: ApiData> OpenApiResponder<'r, 'static> for ApiResponse<T> {
    fn responses(r#gen: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Responses::default();

        let json_schema = r#gen.json_schema::<T>();
        let plain_schema = r#gen.json_schema::<String>();

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
                schema: Some(plain_schema.clone()),
                ..MediaType::default()
            },
        );

        content.insert(
            "text/markdown".to_string(),
            MediaType {
                schema: Some(plain_schema.clone()),
                ..MediaType::default()
            },
        );

        content.insert(
            "text/html".to_string(),
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

impl<'r, T: ApiData> Responder<'r, 'static> for ApiResponse<T> {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        match self {
            ApiResponse::Ok(data, format) => match format {
                ResponseFormat::Json => Json(data).respond_to(req),
                ResponseFormat::Markdown => {
                    let text = markdown::render(data.message());
                    RocketResponse::build()
                        .header(ContentType::new("text", "markdown"))
                        .sized_body(text.len(), Cursor::new(text))
                        .ok()
                }
                ResponseFormat::Plain => {
                    let text = plaintext::render(data.message());
                    RocketResponse::build()
                        .header(ContentType::Plain)
                        .sized_body(text.len(), Cursor::new(text))
                        .ok()
                }
                ResponseFormat::Html => {
                    let output = html::render(data.message());
                    RocketResponse::build()
                        .header(ContentType::HTML)
                        .sized_body(output.len(), Cursor::new(output))
                        .ok()
                }
                ResponseFormat::Shell => {
                    let text = shell::render(data.message());
                    RocketResponse::build()
                        .header(ContentType::Plain)
                        .sized_body(text.len(), Cursor::new(text))
                        .ok()
                }
            },
            ApiResponse::Error(error) => Json(error).respond_to(req),
        }
    }
}
