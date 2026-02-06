use rocket::http::Status;
use rocket_okapi::openapi;

#[openapi(tag = "Teapot")]
#[get("/teapot")]
pub fn teapot() -> Status {
    Status::ImATeapot
}
