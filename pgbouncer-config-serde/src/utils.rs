use serde_json::json;
use crate::error::PgBouncerSerdeError;

pub(crate) fn parse_ini_to_value(s: &str) -> crate::error::Result<serde_json::Value> {
    let mut result = json!({});
    let mut current_section_name = String::new();

    for line in s.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with("#") || line.starts_with(";") {
            continue;
        }

        if line.starts_with("[") && line.ends_with("]") {
            current_section_name = line[1..line.len() - 1].to_string();
            continue
        }

        if let Some((key, value)) = line.split_once("=") {
            let mut current_map = result.as_object_mut()
                .ok_or(PgBouncerSerdeError::InvalidFormat(line.to_string()))?
                .entry(&current_section_name)
                .or_insert(json!({}));

            let keys: Vec<&str> = key.trim().split('.').collect();
            let (leaf_key, parent_keys) = keys.split_last().unwrap();

            for &parent_key in parent_keys {
                current_map = current_map.as_object_mut()
                    .ok_or(PgBouncerSerdeError::InvalidFormat(line.to_string()))?
                    .entry(parent_key)
                    .or_insert(json!({}));
            }

            let final_value = if value.contains(',') {
                json!(value.split(',').map(|s| s.trim()).collect::<Vec<&str>>())
            } else {
                json!(value.trim())
            };

            current_map.as_object_mut()
                .ok_or(PgBouncerSerdeError::InvalidFormat(line.to_string()))?
                .insert(leaf_key.to_string(), final_value);
        } else {
            return Err(PgBouncerSerdeError::InvalidFormat(line.to_string()));
        }
    }
    Ok(result)
}