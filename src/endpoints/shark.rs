use crate::endpoints::{ApiError, ApiResponse};
use rocket::State;
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct SharkCache {
    pub data: SharkData,
    pub time: Instant,
}

pub type Cache = RwLock<Option<SharkCache>>;

#[derive(Serialize, Clone)]
pub struct SharkData {
    beeghaj: i32,
    smolhaj: i32,
    message: String,
}

#[derive(Deserialize)]
struct IkeaResponse {
    availabilities: Vec<Availability>,
}

#[derive(Deserialize)]
struct Availability {
    #[serde(rename = "buyingOption")]
    buying_option: BuyingOption,
    #[serde(rename = "classUnitKey")]
    class_unit_key: ClassUnitKey,
    #[serde(rename = "itemKey")]
    item_key: ItemKey,
}

#[derive(Deserialize)]
struct ItemKey {
    #[serde(rename = "itemNo")]
    item_no: String,
}

#[derive(Deserialize)]
struct BuyingOption {
    #[serde(rename = "cashCarry")]
    cash_carry: Option<CashCarry>,
}

#[derive(Deserialize)]
struct CashCarry {
    availability: Option<CashCarryAvailability>,
}

#[derive(Deserialize)]
struct CashCarryAvailability {
    quantity: i32,
}

#[derive(Deserialize)]
struct ClassUnitKey {
    #[serde(rename = "classUnitCode")]
    class_unit_code: String,
}

async fn fetch_shark_data() -> Result<SharkData, ApiError> {
    let url = "https://api.salesitem.ingka.com/availabilities/ru/de?itemNos=30373588,20540663&expand=StoresList";
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .header("X-Client-ID", "ef382663-a2a5-40d4-8afe-f0634821c0ed")
        .send()
        .await
        .map_err(|_| ApiError {
            message: "Error fetching data from Ikea".to_string(),
        })?;

    let data = response
        .json::<IkeaResponse>()
        .await
        .map_err(|_| ApiError {
            message: "Error parsing Ikea response".to_string(),
        })?;

    let get_quantity = |item_id: &str| -> i32 {
        data.availabilities
            .iter()
            .find(|a| a.class_unit_key.class_unit_code == "147" && a.item_key.item_no == item_id)
            .and_then(|store| {
                store
                    .buying_option
                    .cash_carry
                    .as_ref()
                    .and_then(|cc| cc.availability.as_ref())
                    .map(|a| a.quantity)
            })
            .unwrap_or(0)
    };

    let beeghaj_qty = get_quantity("30373588");
    let smolhaj_qty = get_quantity("20540663");

    let message = if beeghaj_qty == 0 && smolhaj_qty == 0 {
        "Der IKEA Godorf hat aktuell überhaupt keine sharks auf lager :(".to_string()
    } else {
        let format_part = |qty: i32, name: &str| -> String {
            let suffix = if qty == 1 { "" } else { "s" };
            let count = if qty == 0 {
                "keine".to_string()
            } else {
                qty.to_string()
            };
            format!("{} {}{}", count, name, suffix)
        };

        let beeghaj_str = format_part(beeghaj_qty, "beeghaj");
        let smolhaj_str = format_part(smolhaj_qty, "smolhaj");

        let mut msg = format!(
            "Der IKEA Godorf hat aktuell {} und {} auf Lager",
            beeghaj_str, smolhaj_str
        );

        if beeghaj_qty > 0 && smolhaj_qty > 0 {
            msg.push_str(" :D");
        }
        msg
    };

    Ok(SharkData {
        beeghaj: beeghaj_qty,
        smolhaj: smolhaj_qty,
        message,
    })
}

async fn get_shark_data(cache_state: &State<Cache>) -> Result<SharkData, ApiError> {
    // First, try to get a read lock.
    {
        let cache = cache_state.read().await;
        if let Some(shark_cache) = &*cache {
            if shark_cache.time.elapsed() < Duration::from_secs(300) {
                return Ok(shark_cache.data.clone());
            }
        }
    }

    // If the cache is stale or empty, get a write lock to update it.
    let mut cache = cache_state.write().await;

    // Check again if another request updated the cache.
    if let Some(shark_cache) = &*cache {
        if shark_cache.time.elapsed() < Duration::from_secs(300) {
            return Ok(shark_cache.data.clone());
        }
    }

    let data = fetch_shark_data().await?;

    *cache = Some(SharkCache {
        data: data.clone(),
        time: Instant::now(),
    });

    Ok(data)
}

#[get("/shark?<format>")]
pub async fn shark(cache_state: &State<Cache>, format: Option<String>) -> ApiResponse<SharkData> {
    let data = match get_shark_data(cache_state).await {
        Ok(data) => data,
        Err(e) => return ApiResponse::Error(e),
    };

    match format.as_deref() {
        Some("json") => ApiResponse::Json(data),
        _ => ApiResponse::Plain(data.message),
    }
}
