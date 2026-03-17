pub mod alditowels;
pub mod congressbeer;
pub mod hello;
pub mod mensabeer;
pub mod mensagorgonzola;
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
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::response::OpenApiResponder;
use serde::Serialize;
use std::io::Cursor;

#[derive(Serialize, JsonSchema)]
pub struct ApiError {
    pub message: String,
}

use rocket::request::{FromRequest, Outcome};
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};

pub struct UserAgent(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAgent {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(UserAgent(
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
    Plain,
    Html,
}

impl ResponseFormat {
    pub fn detect(ua: &UserAgent, format_param: Option<String>) -> Self {
        if let Some(f) = format_param {
            match f.to_lowercase().as_str() {
                "json" => return ResponseFormat::Json,
                "html" => return ResponseFormat::Html,
                "plaintext" | "plain" | "text" => return ResponseFormat::Plain,
                _ => {}
            }
        }

        if ua.0.contains("Mozilla") {
            ResponseFormat::Html
        } else {
            ResponseFormat::Plain
        }
    }
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
                ResponseFormat::Plain => {
                    let text = format!("{}\n", data.message());
                    RocketResponse::build()
                        .header(ContentType::Plain)
                        .sized_body(text.len(), Cursor::new(text))
                        .ok()
                }
                ResponseFormat::Html => {
                    let message = data.message();
                    let options = pulldown_cmark::Options::all();
                    let parser = pulldown_cmark::Parser::new_ext(message, options);
                    let mut html_output = String::new();
                    pulldown_cmark::html::push_html(&mut html_output, parser);

                    let html = format!(
                        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Useful API</title>
    <style>
        body {{
            background-color: #222;
            color: #eee;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            text-align: center;
            padding: 2rem;
            box-sizing: border-box;
            font-size: 1.5rem;
            line-height: 1.4;
        }}
        div.container {{
            max-width: 800px;
        }}
        a {{
            color: #8ebcf1;
            text-decoration: none;
        }}
        a:hover {{
            text-decoration: underline;
        }}
        ul {{
            text-align: left;
            display: inline-block;
            margin: 1rem 0;
        }}
        li {{
            margin: 0.5rem 0;
        }}
    </style>
</head>
<body>
    <div class="container">{}</div>
</body>
</html>"#,
                        html_output
                    );
                    RocketResponse::build()
                        .header(ContentType::HTML)
                        .sized_body(html.len(), Cursor::new(html))
                        .ok()
                }
            },
            ApiResponse::Error(error) => Json(error).respond_to(req),
        }
    }
}
