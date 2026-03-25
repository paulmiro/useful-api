use crate::api::{ApiData, ApiResponse, ResponseFormat, UserAgent};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct RootData {
    pub message: String,
}

impl ApiData for RootData {
    fn message(&self) -> &str {
        &self.message
    }
}

#[openapi(tag = "Root")]
#[get("/?<format>")]
pub fn root(ua: UserAgent, format: Option<String>) -> ApiResponse<RootData> {
    let format = ResponseFormat::detect(&ua, format);
    let prefix = if format == ResponseFormat::Html {
        ""
    } else {
        "https://useful-api.party"
    };
    let message = format!(
        r#"# Useful API
## Endpoints

- [hello]({prefix}/hello)
- [alditowels]({prefix}/alditowels)
- [mensagorgonzola]({prefix}/mensagorgonzola)
- [mensatoshi]({prefix}/mensatoshi)
- [congressbeer]({prefix}/congressbeer)
- [shark]({prefix}/shark)
- [mensabeer]({prefix}/mensabeer)
- [teapot]({prefix}/teapot)

## Docs

- [swagger-ui]({prefix}/swagger-ui/)
"#
    );

    ApiResponse::Ok(RootData { message }, format)
}
