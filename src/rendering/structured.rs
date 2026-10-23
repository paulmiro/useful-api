use serde::Serialize;

pub fn yaml<T: Serialize>(data: &T) -> Result<String, String> {
    serde_yaml_ng::to_string(data).map_err(|e| e.to_string())
}
