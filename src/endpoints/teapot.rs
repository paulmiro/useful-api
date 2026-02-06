use rocket::http::Status;

#[get("/teapot")]
pub fn teapot() -> Status {
    Status::ImATeapot
}
