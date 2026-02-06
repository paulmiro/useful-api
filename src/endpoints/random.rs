use crate::{all_routes, endpoints::ApiResponse};
use rocket::serde::Serialize;
use rocket::rand::seq::SliceRandom;
use rocket::rand::thread_rng;

#[derive(Serialize)]
pub struct RandomRoute {
    route: String,
}

#[get("/random?<format>")]
pub fn random(format: Option<String>) -> ApiResponse<RandomRoute> {
    // Get the list of routes from the central definition in main.rs
    // and exclude /random itself to avoid recursion.
    let routes: Vec<&str> = all_routes()
        .iter()
        .copied()
        .filter(|route| *route != "/random")
        .collect();

    let mut rng = thread_rng();
    let chosen = routes
        .choose(&mut rng)
        .unwrap_or(&"/")
        .to_string();

    match format.as_deref() {
        Some("json") => ApiResponse::Json(RandomRoute {
            route: chosen.clone(),
        }),
        _ => ApiResponse::Plain(chosen),
    }
}
