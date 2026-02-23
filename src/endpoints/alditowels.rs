use crate::endpoints::{ApiError, ApiResponse};
use chrono::{Datelike, Local};
use rocket::State;
use rocket::tokio::sync::RwLock;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use scraper::{Html, Selector};
use serde::Serialize;
use std::collections::HashSet;
use std::time::{Duration, Instant};

#[derive(Serialize, Clone, JsonSchema)]
pub struct AldiTowelData {
    pub sells_towels: bool,
    pub will_sell_towels: bool,
    pub message: String,
    pub availability: Vec<String>,
    pub products: Vec<String>,
}

#[derive(Clone)]
pub struct AldiTowelCache {
    pub data: AldiTowelData,
    pub time: Instant,
}

pub type Cache = RwLock<Option<AldiTowelCache>>;

async fn fetch_aldi_search(query: &str) -> Result<String, reqwest::Error> {
    let url = format!("https://www.aldi-sued.de/suchergebnisse?q={}", query);
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:135.0) Gecko/20100101 Firefox/135.0")
        .build()
        .unwrap();

    let response = client.get(url).send().await?;
    response.text().await
}

/// Simple helper to check if a German date string (DD.MM.YYYY) is in the future.
fn is_future_date(date_str: &str) -> bool {
    let now = Local::now();
    let current_day = now.day() as i32;
    let current_month = now.month() as i32;
    let current_year = now.year();

    // Expected format: something containing DD.MM.YYYY
    let parts: Vec<&str> = date_str
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() >= 3 {
        // Assume DD, MM, YYYY
        if let (Ok(d), Ok(m), Ok(y)) = (
            parts[0].parse::<i32>(),
            parts[1].parse::<i32>(),
            parts[2].parse::<i32>(),
        ) {
            if y > current_year {
                return true;
            }
            if y < current_year {
                return false;
            }
            if m > current_month {
                return true;
            }
            if m < current_month {
                return false;
            }
            if d > current_day {
                return true;
            }
        }
    }
    false
}

async fn get_aldi_towel_data() -> Result<AldiTowelData, ApiError> {
    let queries = vec!["handt%C3%BCcher", "handtuch"];
    let mut bodies = Vec::new();

    for query in queries {
        match fetch_aldi_search(query).await {
            Ok(body) => bodies.push(body),
            Err(e) => {
                return Err(ApiError {
                    message: format!("Failed to fetch Aldi website for query '{}': {}", query, e),
                });
            }
        }
    }

    let mut raw_availability = HashSet::new();
    let mut product_names = HashSet::new();
    let mut has_products = false;

    // Selectors for Aldi's typical structure
    let product_tile_selector = Selector::parse(".product-tile").unwrap();
    let product_title_selector =
        Selector::parse(".product-tile__title, .product-tile__name").unwrap();
    let availability_selector =
        Selector::parse(".product-tile__availability, .availability-label, .badge--availability")
            .unwrap();

    for body in bodies {
        let document = Html::parse_document(&body);

        for tile in document.select(&product_tile_selector) {
            let mut name_found = false;

            if let Some(name_element) = tile.select(&product_title_selector).next() {
                let name = name_element
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .to_string();
                let name_lower = name.to_lowercase();
                if name_lower.contains("handtuch") || name_lower.contains("handtücher") {
                    product_names.insert(name);
                    has_products = true;
                    name_found = true;
                }
            }

            // Only look for availability if this tile is actually a towel
            if name_found {
                if let Some(avail_element) = tile.select(&availability_selector).next() {
                    let avail_text = avail_element
                        .text()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .trim()
                        .to_string();
                    if !avail_text.is_empty() {
                        raw_availability.insert(avail_text);
                    }
                }
            }
        }

        // Fallback: If no structured tiles found, use the old string-based approach but cleaner
        if !has_products {
            for term in ["Handtuch", "Handtücher"] {
                let mut search_pos = 0;
                while let Some(start_idx) = body[search_pos..].find(term) {
                    let absolute_start = search_pos + start_idx;
                    let sub = &body[absolute_start..];
                    let end_idx = sub
                        .find(|c: char| {
                            !c.is_alphanumeric() && c != ' ' && c != ',' && c != '-' && c != '.'
                        })
                        .unwrap_or(40);
                    let name = sub[..end_idx].trim().trim_end_matches([',', '.']).trim();
                    if name.len() > 8 && name.len() < 100 {
                        product_names.insert(name.to_string());
                        has_products = true;
                    }
                    search_pos = absolute_start + end_idx.max(1);
                }
            }

            for prefix in ["Verfügbar ab ", "ab "] {
                let mut search_pos = 0;
                while let Some(start_idx) = body[search_pos..].find(prefix) {
                    let absolute_start = search_pos + start_idx;
                    let sub = &body[absolute_start..];
                    let end_idx = sub
                        .find(|c: char| !c.is_alphanumeric() && c != '.' && c != ' ' && c != ',')
                        .unwrap_or(30);
                    let info = sub[..end_idx].trim();
                    if info.len() >= 5 && info.contains('.') {
                        raw_availability.insert(info.to_string());
                    }
                    search_pos = absolute_start + end_idx.max(1);
                }
            }
        }
    }

    // Advanced deduplication and cleaning
    let mut final_availability = Vec::new();
    let mut availability_vec: Vec<String> = raw_availability.into_iter().collect();
    availability_vec.sort_by_key(|b| std::cmp::Reverse(b.len()));

    for item in availability_vec {
        if !final_availability
            .iter()
            .any(|existing: &String| existing.contains(&item) || item.contains(existing))
        {
            final_availability.push(item);
        }
    }
    final_availability.sort();

    let mut products: Vec<String> = product_names.into_iter().collect();
    products.sort();

    // Logic for now vs future
    let mut sells_towels = false;
    let mut will_sell_towels = false;

    if has_products {
        if final_availability.is_empty() {
            sells_towels = true;
        } else {
            for avail in &final_availability {
                if is_future_date(avail) {
                    will_sell_towels = true;
                } else {
                    sells_towels = true;
                }
            }
        }
    }

    let message = if sells_towels || will_sell_towels {
        let availability_str = if !final_availability.is_empty() {
            format!(" ({})", final_availability.join(", "))
        } else {
            "".to_string()
        };

        let product_str = if !products.is_empty() {
            format!(": {}", products.join(", "))
        } else {
            "".to_string()
        };

        if sells_towels {
            format!(
                "Ja, Aldi Süd hat aktuell Handtücher im Angebot!{}{}",
                availability_str, product_str
            )
        } else {
            format!(
                "Ja, Aldi Süd hat bald Handtücher im Angebot!{}{}",
                availability_str, product_str
            )
        }
    } else {
        "Nein, Aldi Süd hat aktuell keine Handtücher im Angebot.".to_string()
    };

    Ok(AldiTowelData {
        sells_towels,
        will_sell_towels,
        message,
        availability: final_availability,
        products,
    })
}

#[openapi(tag = "Scraping")]
#[get("/alditowels?<format>")]
pub async fn alditowels(
    cache_state: &State<Cache>,
    format: Option<String>,
) -> ApiResponse<AldiTowelData> {
    {
        let cache = cache_state.read().await;
        if let Some(c) = &*cache {
            if c.time.elapsed() < Duration::from_secs(600) {
                let data = c.data.clone();
                return match format.as_deref() {
                    Some("json") => ApiResponse::Json(data),
                    _ => ApiResponse::Plain(data.message),
                };
            }
        }
    }

    let data = match get_aldi_towel_data().await {
        Ok(d) => d,
        Err(e) => return ApiResponse::Error(e),
    };

    let mut cache = cache_state.write().await;
    *cache = Some(AldiTowelCache {
        data: data.clone(),
        time: Instant::now(),
    });

    match format.as_deref() {
        Some("json") => ApiResponse::Json(data),
        _ => ApiResponse::Plain(data.message),
    }
}
