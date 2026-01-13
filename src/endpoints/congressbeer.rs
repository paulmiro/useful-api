#[get("/congressbeer?<satoshi>")]
pub fn congressbeer(satoshi: f64) -> String {
    const CONGRESSBEER_SATOSHI: f64 = 69.0;
    let congressbeers = (satoshi / CONGRESSBEER_SATOSHI).floor() as i64;
    format!(
        "{} Satoshi entspricht {} Congressbeers.",
        satoshi, congressbeers
    )
}
