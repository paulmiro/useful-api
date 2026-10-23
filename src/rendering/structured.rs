use serde::Serialize;

pub fn yaml<T: Serialize>(data: &T) -> Result<String, String> {
    serde_yaml_ng::to_string(data).map_err(|e| e.to_string())
}

pub fn xml<T: Serialize>(data: &T) -> Result<String, String> {
    let body = quick_xml::se::to_string_with_root("response", data).map_err(|e| e.to_string())?;
    Ok(format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{body}\n"))
}
