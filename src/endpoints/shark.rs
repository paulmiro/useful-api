use crate::endpoints::{ApiError, ApiResponse};
use rocket::serde::{Deserialize, Serialize};
use rocket::State;

#[derive(Serialize)]
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

#[get("/shark?<format>")]
pub async fn shark(format: Option<String>, client: &State<reqwest::Client>) -> ApiResponse<SharkData> {
    let url = "https://api.salesitem.ingka.com/availabilities/ru/de?itemNos=30373588,20540663&expand=StoresList";

    let response = match client
        .get(url)
        .header("X-Client-ID", "ef382663-a2a5-40d4-8afe-f0634821c0ed")
        .send()
        .await
    {
        Ok(res) => res,
        Err(_) => {
            return ApiResponse::Error(ApiError {
                message: "Error fetching data from Ikea".to_string(),
            });
        }
    };

    let data = match response.json::<IkeaResponse>().await {
        Ok(json) => json,
        Err(_) => {
            return ApiResponse::Error(ApiError {
                message: "Error parsing Ikea response".to_string(),
            });
        }
    };

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

    if beeghaj_qty == 0 && smolhaj_qty == 0 {
        let msg = "Der IKEA Godorf hat aktuell überhaupt keine sharks auf lager :(".to_string();
        return match format.as_deref() {
            Some("json") => ApiResponse::Json(SharkData {
                beeghaj: 0,
                smolhaj: 0,
                message: msg,
            }),
            _ => ApiResponse::Plain(msg),
        };
    }

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

    let mut response_str = format!(
        "Der IKEA Godorf hat aktuell {} und {} auf Lager",
        beeghaj_str, smolhaj_str
    );

    if beeghaj_qty > 0 && smolhaj_qty > 0 {
        response_str.push_str(" :D");
    }

    match format.as_deref() {
        Some("json") => ApiResponse::Json(SharkData {
            beeghaj: beeghaj_qty,
            smolhaj: smolhaj_qty,
            message: response_str,
        }),
        _ => ApiResponse::Plain(response_str),
    }
}
