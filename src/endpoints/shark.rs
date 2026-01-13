use rocket::serde::Deserialize;

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

#[get("/shark")]
pub async fn shark() -> String {
    let url = "https://api.salesitem.ingka.com/availabilities/ru/de?itemNos=30373588,20540663&expand=StoresList";
    let client = reqwest::Client::new();

    let response = match client
        .get(url)
        .header("X-Client-ID", "ef382663-a2a5-40d4-8afe-f0634821c0ed")
        .send()
        .await
    {
        Ok(res) => res,
        Err(_) => return "Error fetching data from Ikea".to_string(),
    };

    let data = match response.json::<IkeaResponse>().await {
        Ok(json) => json,
        Err(_) => return "Error parsing Ikea response".to_string(),
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
        return "Der IKEA Godorf hat aktuell überhaupt keine sharks auf lager :(".to_string();
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

    response_str
}
