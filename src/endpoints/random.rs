use crate::endpoints::ApiResponse;
use rand::seq::SliceRandom;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use serde::Serialize;

#[derive(Serialize, JsonSchema, Clone)]
pub struct EndpointInfo {
    path: String,
    description: String,
}

#[derive(Serialize, JsonSchema)]
pub struct RandomData {
    endpoints: Vec<EndpointInfo>,
}

/// Build a list of endpoints from route handler functions.
/// Paths are derived directly from the route handlers so they stay in sync automatically.
macro_rules! endpoint_list {
    ( $( $handler:path => $desc:literal ),* $(,)? ) => {{
        vec![$({
            let route = &rocket::routes![$handler][0];
            EndpointInfo {
                path: route.uri.to_string(),
                description: $desc.to_string(),
            }
        }),*]
    }};
}

fn all_endpoints() -> Vec<EndpointInfo> {
    // Endpoints that require meaningful user input (e.g. congressbeer needs satoshi)
    // are intentionally excluded.
    endpoint_list![
        crate::endpoints::hello::hello              => "Hello, World!",
        crate::endpoints::alditowels::alditowels    => "Check Aldi Süd towel availability",
        crate::endpoints::mensagorgonzola::mensa_gorgonzola => "Check if Mensa CAMPO has Gorgonzola today",
        crate::endpoints::mensatoshi::mensatoshi    => "Convert Mensa Eintopf price to Satoshis",
        crate::endpoints::shark::shark              => "Check IKEA Godorf stock for BLÅHAJ shark plushies",
        crate::endpoints::mensabeer::mensabeer      => "Convert Mensa Eintopf price to Congress beers",
        crate::endpoints::teapot::teapot            => "I'm a teapot",
    ]
}

#[openapi(tag = "Random")]
#[get("/random?<format>")]
pub fn random(format: Option<String>) -> ApiResponse<RandomData> {
    let mut rng = rand::thread_rng();
    let mut endpoints = all_endpoints();
    endpoints.shuffle(&mut rng);
    let count = rand::Rng::gen_range(&mut rng, 1usize..=3usize).min(endpoints.len());
    endpoints.truncate(count);

    match format.as_deref() {
        Some("json") => ApiResponse::Json(RandomData { endpoints }),
        _ => {
            let text = endpoints
                .iter()
                .map(|e| format!("{} - {}", e.path, e.description))
                .collect::<Vec<_>>()
                .join("\n");
            ApiResponse::Plain(text)
        }
    }
}
