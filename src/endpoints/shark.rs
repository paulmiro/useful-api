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

    let mut beeghaj_count = 0;
    let mut smolhaj_count = 0;
    let mut found_store = false;

    for availability in data.availabilities {
        if availability.class_unit_key.class_unit_code == "147" {
            found_store = true;
            let quantity = availability
                .buying_option
                .cash_carry
                .as_ref()
                .and_then(|cc| cc.availability.as_ref())
                .map(|a| a.quantity)
                .unwrap_or(0);

            if availability.item_key.item_no == "30373588" {
                beeghaj_count = quantity;
            } else if availability.item_key.item_no == "20540663" {
                smolhaj_count = quantity;
            }
        }
    }

    if !found_store {
        return "Store 147 not found in Ikea response".to_string();
    }

    let format_count = |n: i32| -> String {
        if n == 0 {
            "keine".to_string()
        } else {
            n.to_string()
        }
    };

    let format_suffix = |n: i32| -> &str {
        if n == 0 || n > 1 {
            "s"
        } else {
            ""
        }
    };

    let beeghaj_str = format_count(beeghaj_count);
    let beeghaj_suffix = format_suffix(beeghaj_count);
    let smolhaj_str = format_count(smolhaj_count);
    let smolhaj_suffix = format_suffix(smolhaj_count);

    let mut result = format!(
        "Der IKEA Godorf hat aktuell {} beeghaj{} und {} smolhaj{} auf Lager",
        beeghaj_str, beeghaj_suffix, smolhaj_str, smolhaj_suffix
    );

    if beeghaj_count == 0 && smolhaj_count == 0 {
        result.push_str(":(");
    }

    result
}
