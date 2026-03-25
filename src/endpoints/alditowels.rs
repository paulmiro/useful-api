use crate::api::{ApiData, ApiError, ApiResponse, ResponseFormat, UserAgent};
use chrono::{Datelike, Local};
use rocket::State;
use rocket::tokio::sync::RwLock;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use scraper::{Html, Selector};
use serde::Serialize;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

#[derive(Serialize, Clone, JsonSchema, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Product {
    pub name: String,
    pub link: Option<String>,
    pub availability: Option<String>,
}

#[derive(Serialize, Clone, JsonSchema)]
pub struct AldiTowelData {
    pub sells_towels: bool,
    pub will_sell_towels: bool,
    pub message: String,
    pub products: Vec<Product>,
}

impl ApiData for AldiTowelData {
    fn message(&self) -> &str {
        &self.message
    }
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
    let queries = vec![
        "handtuch",
        "handt%C3%BCcher",
        "duschtuch",
        "duscht%C3%BCcher",
        "badetuch",
        "badet%C3%BCcher",
    ];
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

    let mut product_map = BTreeMap::new();
    let mut has_products = false;

    // Selectors for Aldi's typical structure
    let product_tile_selector = Selector::parse(".product-tile").unwrap();
    let product_title_selector =
        Selector::parse(".product-tile__title, .product-tile__name").unwrap();
    let product_link_selector = Selector::parse("a.product-tile, .product-tile a").unwrap();
    let availability_selector =
        Selector::parse(".product-tile__availability, .availability-label, .badge--availability")
            .unwrap();

    let keywords = [
        "handtuch",
        "handtücher",
        "duschtuch",
        "duschtücher",
        "badetuch",
        "badetücher",
        "saunatuch",
        "saunatücher",
        "gästetuch",
        "gästetücher",
        "strandtuch",
        "strandtücher",
        "waschhandschuh",
        "waschhandschuhe",
        "frottier",
    ];

    for body in bodies {
        let document = Html::parse_document(&body);

        for tile in document.select(&product_tile_selector) {
            let mut name_found = false;
            let mut product_name = String::new();

            if let Some(name_element) = tile.select(&product_title_selector).next() {
                let name = name_element
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .to_string();
                let name_lower = name.to_lowercase();
                if keywords.iter().any(|&k| name_lower.contains(k)) {
                    product_name = name;
                    has_products = true;
                    name_found = true;
                }
            }

            // Only look for availability if this tile is actually a towel
            if name_found {
                let mut product_link = None;
                if let Some(link_element) = tile.select(&product_link_selector).next() {
                    if let Some(href) = link_element.value().attr("href") {
                        let full_link = if href.starts_with('/') {
                            format!("https://www.aldi-sued.de{}", href)
                        } else {
                            href.to_string()
                        };
                        product_link = Some(full_link);
                    }
                } else if tile.value().name() == "a" {
                    if let Some(href) = tile.value().attr("href") {
                        let full_link = if href.starts_with('/') {
                            format!("https://www.aldi-sued.de{}", href)
                        } else {
                            href.to_string()
                        };
                        product_link = Some(full_link);
                    }
                }

                let availability = tile
                    .select(&availability_selector)
                    .next()
                    .map(|avail_element| {
                        avail_element
                            .text()
                            .collect::<Vec<_>>()
                            .join(" ")
                            .trim()
                            .to_string()
                    })
                    .filter(|avail_text| !avail_text.is_empty());

                product_map
                    .entry((product_name.clone(), product_link.clone()))
                    .and_modify(|product: &mut Product| {
                        if product.availability.is_none() {
                            product.availability = availability.clone();
                        }
                    })
                    .or_insert(Product {
                        name: product_name,
                        link: product_link,
                        availability,
                    });
            }
        }

        // Fallback: If no structured tiles found, use the old string-based approach but cleaner
        if !has_products {
            for term in keywords {
                // Capitalize first letter for fallback search if needed, but Aldi search is often case insensitive in HTML
                // We search for both lowercase and title case just in case
                let search_terms = [term.to_string(), {
                    let mut c = term.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                }];

                for s_term in search_terms {
                    let mut search_pos = 0;
                    while let Some(start_idx) = body[search_pos..].find(&s_term) {
                        let absolute_start = search_pos + start_idx;
                        let sub = &body[absolute_start..];
                        let end_idx = sub
                            .find(|c: char| {
                                !c.is_alphanumeric() && c != ' ' && c != ',' && c != '-' && c != '.'
                            })
                            .unwrap_or(40);
                        let name = sub[..end_idx].trim().trim_end_matches([',', '.']).trim();
                        if name.len() > 5 && name.len() < 100 {
                            product_map
                                .entry((name.to_string(), None))
                                .or_insert(Product {
                                    name: name.to_string(),
                                    link: None,
                                    availability: None,
                                });
                            has_products = true;
                        }
                        search_pos = absolute_start + end_idx.max(1);
                    }
                }
            }
        }
    }

    let products: Vec<Product> = product_map.into_values().collect();

    // Logic for now vs future
    let mut sells_towels = false;
    let mut will_sell_towels = false;

    if has_products {
        for product in &products {
            match &product.availability {
                Some(avail) if is_future_date(avail) => will_sell_towels = true,
                _ => sells_towels = true,
            }
        }
    }

    let message = if sells_towels || will_sell_towels {
        let product_str = if !products.is_empty() {
            let product_lines: Vec<String> = products
                .iter()
                .map(|p| {
                    let product_label = match &p.link {
                        Some(link) => format!("[{}]({})", p.name, link),
                        None => p.name.clone(),
                    };

                    match &p.availability {
                        Some(availability) => format!("- {} ({})", product_label, availability),
                        None => format!("- {}", product_label),
                    }
                })
                .collect();
            format!(
                r#"## Produkte

{}"#,
                product_lines.join("\n")
            )
        } else {
            "".to_string()
        };

        if sells_towels {
            format!(
                r#"# Aldi-Handtücher

**Ja**, Aldi Süd hat aktuell Handtücher im Angebot.
{product_str}"#,
            )
        } else {
            format!(
                r#"# Aldi-Handtücher

**Ja**, Aldi Süd hat bald Handtücher im Angebot.
{product_str}"#,
            )
        }
    } else {
        r#"# Aldi-Handtücher

**Nein**, Aldi Süd hat aktuell keine Handtücher im Angebot."#
            .to_string()
    };

    Ok(AldiTowelData {
        sells_towels,
        will_sell_towels,
        message,
        products,
    })
}

#[openapi(tag = "Scraping")]
#[get("/alditowels?<format>")]
pub async fn alditowels(
    ua: UserAgent,
    cache_state: &State<Cache>,
    format: Option<String>,
) -> ApiResponse<AldiTowelData> {
    let format = ResponseFormat::detect(&ua, format);
    {
        let cache = cache_state.read().await;
        if let Some(c) = &*cache {
            if c.time.elapsed() < Duration::from_secs(600) {
                return ApiResponse::Ok(c.data.clone(), format);
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

    ApiResponse::Ok(data, format)
}
