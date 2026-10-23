use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

pub fn yaml<T: Serialize>(data: &T) -> Result<String, String> {
    serde_yaml_ng::to_string(data).map_err(|e| e.to_string())
}

pub fn xml<T: Serialize>(data: &T) -> Result<String, String> {
    let body = quick_xml::se::to_string_with_root("response", data).map_err(|e| e.to_string())?;
    Ok(format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{body}\n"))
}

pub fn toml<T: Serialize>(data: &T) -> Result<String, String> {
    toml::to_string(data).map_err(|e| e.to_string())
}

/// INI and dotenv only support flat string maps, so nested data is flattened first.
fn flatten<T: Serialize>(data: &T, separator: &str) -> Result<BTreeMap<String, String>, String> {
    fn walk(value: &Value, path: &str, separator: &str, out: &mut BTreeMap<String, String>) {
        let join = |key: &str| {
            if path.is_empty() { key.to_string() } else { format!("{path}{separator}{key}") }
        };
        match value {
            Value::Object(map) => map.iter().for_each(|(k, v)| walk(v, &join(k), separator, out)),
            Value::Array(items) => items
                .iter()
                .enumerate()
                .for_each(|(i, v)| walk(v, &join(&i.to_string()), separator, out)),
            Value::Null => {}
            Value::String(s) => {
                out.insert(path.to_string(), s.clone());
            }
            other => {
                out.insert(path.to_string(), other.to_string());
            }
        }
    }

    let value = serde_json::to_value(data).map_err(|e| e.to_string())?;
    let mut out = BTreeMap::new();
    walk(&value, "", separator, &mut out);
    Ok(out)
}

pub fn ini<T: Serialize>(data: &T) -> Result<String, String> {
    let bytes = serde_ini::to_vec(&flatten(data, ".")?).map_err(|e| e.to_string())?;
    String::from_utf8(bytes)
        .map(|text| text.replace("\r\n", "\n"))
        .map_err(|e| e.to_string())
}

pub fn env<T: Serialize>(data: &T) -> Result<String, String> {
    let vars: BTreeMap<String, String> = flatten(data, "_")?
        .into_iter()
        .map(|(k, v)| (k.to_uppercase(), v))
        .collect();
    serde_envfile::to_string(&vars)
        .map(|text| format!("{}\n", text.trim_end()))
        .map_err(|e| e.to_string())
}
