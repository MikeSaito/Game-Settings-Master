use super::super::types::ParameterCatalogEntry;

pub(crate) fn derive_catalog_recommended(entry: &ParameterCatalogEntry) -> bool {
    if entry.catalog_recommended {
        return true;
    }
    if entry.key.starts_with("sg.") || entry.category == "Scalability" || entry.category == "Display" {
        return true;
    }
    entry
        .file
        .as_deref()
        .is_some_and(|file| file == "Engine.ini" || file == "Scalability.ini")
}

pub(crate) fn catalog_default_value(entry: &ParameterCatalogEntry) -> String {
    if let Some(hint) = &entry.value_hint {
        if let Some(num) = extract_hint_number(hint) {
            return num;
        }
    }
    match entry.value_type.as_str() {
        "bool" => "True".to_string(),
        "int" => {
            if let Some(max) = &entry.max {
                if let Ok(m) = max.parse::<i64>() {
                    if m <= 5 {
                        return max.clone();
                    }
                }
            }
            entry.min.clone().unwrap_or_else(|| "1".to_string())
        }
        "float" => "1.0".to_string(),
        _ => String::new(),
    }
}

fn extract_hint_number(hint: &str) -> Option<String> {
    let token = hint
        .split(|c: char| c == ',' || c == '—' || c == '-' || c == ' ')
        .find_map(|part| {
            let t = part.trim();
            if t.parse::<f64>().is_ok() || t.parse::<i64>().is_ok() {
                Some(t.to_string())
            } else {
                None
            }
        })?;
    Some(token)
}
