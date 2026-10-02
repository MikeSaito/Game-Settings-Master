use crate::core::app_error::{AppError, AppInvokeError};
use crate::ini::document::{Document, Entry};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct InputEntry {
    pub id: String,
    pub line: u32,
    pub key: String,
    pub value: String,
    pub fields: BTreeMap<String, String>,
    #[serde(default)]
    pub axis_properties: Option<BTreeMap<String, String>>,
    pub editable: bool,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct InputDocument {
    pub revision: String,
    pub entries: Vec<InputEntry>,
}

/// Split a UE struct, respecting nested structs, quoted commas and escapes.
pub(crate) fn fields(value: &str) -> Option<BTreeMap<String, String>> {
    let inner = value.trim().strip_prefix('(')?.strip_suffix(')')?;
    let mut result = BTreeMap::new();
    let mut depth = 0i32;
    let mut quoted = false;
    let mut escaped = false;
    let mut start = 0;
    for (index, ch) in inner
        .char_indices()
        .chain(std::iter::once((inner.len(), ',')))
    {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && quoted {
            escaped = true;
            continue;
        }
        if ch == '"' {
            quoted = !quoted;
        }
        if !quoted {
            if ch == '(' {
                depth += 1;
            }
            if ch == ')' {
                depth -= 1;
            }
            if depth < 0 {
                return None;
            }
            if ch == ',' && depth == 0 {
                let (key, value) = inner[start..index].trim().split_once('=')?;
                if key.trim().is_empty()
                    || result
                        .insert(key.trim().into(), value.trim().into())
                        .is_some()
                {
                    return None;
                }
                start = index + 1;
            }
        }
    }
    if quoted || depth != 0 {
        None
    } else {
        Some(result)
    }
}

pub(crate) fn identified_entries(document: &Document) -> Vec<(Entry, String)> {
    let mut counts = BTreeMap::new();
    document
        .entries()
        .into_iter()
        .filter(supported)
        .map(|entry| {
            let parsed = fields(&entry.value).unwrap_or_default();
            let name = ["ActionName", "AxisName", "AxisKeyName"]
                .iter()
                .find_map(|key| parsed.get(*key))
                .map(|value| value.trim_matches('"'))
                .unwrap_or("");
            let identity = format!(
                "{}|{}|{name}",
                entry.section.to_ascii_lowercase(),
                entry.key.to_ascii_lowercase()
            );
            let occurrence = counts.entry(identity.clone()).or_insert(0u32);
            let id = format!(
                "{}:{}",
                crate::changes::digest(identity.as_bytes()),
                occurrence
            );
            *occurrence += 1;
            (entry, id)
        })
        .collect()
}

fn base_key(key: &str) -> &str {
    key.trim_start_matches(['+', '-', '!', '.'])
}
fn supported(entry: &Entry) -> bool {
    entry
        .section
        .eq_ignore_ascii_case("/Script/Engine.InputSettings")
        && matches!(
            base_key(&entry.key),
            "ActionMappings" | "AxisMappings" | "AxisConfig"
        )
}

fn editable(document: &Document, entry: &Entry) -> bool {
    if !supported(entry) || entry.key.starts_with(['!', '-']) || fields(&entry.value).is_none() {
        return false;
    }
    let base = base_key(&entry.key);
    let preceding: Vec<_> = document
        .entries()
        .into_iter()
        .filter(|other| {
            other.line <= entry.line
                && other.section.eq_ignore_ascii_case(&entry.section)
                && base_key(&other.key) == base
        })
        .collect();
    let locally_defined = preceding
        .iter()
        .any(|other| !other.key.starts_with(['+', '-', '!', '.']) || other.key.starts_with('!'));
    locally_defined
        && !document.entries().iter().any(|other| {
            other.section.eq_ignore_ascii_case(&entry.section)
                && base_key(&other.key) == base
                && (other.key.starts_with('-')
                    || (other.line > entry.line
                        && (other.key.starts_with('!')
                            || !other.key.starts_with(['+', '-', '!', '.']))))
        })
}

/// UE rewrites numeric, boolean and FName formatting during serialization.
pub(crate) fn values_equal(expected: &str, actual: &str) -> bool {
    let (Some(expected), Some(actual)) = (fields(expected), fields(actual)) else {
        return expected.trim() == actual.trim();
    };
    expected.iter().all(|(key, value)| {
        let Some(other) = actual.get(key) else {
            return false;
        };
        match key.as_str() {
            "AxisProperties" => values_equal(value, other),
            "Scale" | "Sensitivity" | "DeadZone" | "Exponent" => {
                match (value.parse::<f64>(), other.parse::<f64>()) {
                    (Ok(left), Ok(right)) => left.is_finite() && right.is_finite() && left == right,
                    _ => value == other,
                }
            }
            "bShift" | "bCtrl" | "bAlt" | "bCmd" | "bInvert" => value.eq_ignore_ascii_case(other),
            "ActionName" | "AxisName" | "AxisKeyName" | "Key" => value
                .trim_matches('"')
                .eq_ignore_ascii_case(other.trim_matches('"')),
            _ => value == other,
        }
    })
}

pub(crate) fn validate_array_uniqueness(
    document: &Document,
    updates: &[(usize, String)],
) -> Result<(), String> {
    let mut entries = document.entries();
    for (line, value) in updates {
        if let Some(entry) = entries.iter_mut().find(|entry| entry.line == *line) {
            entry.value = value.clone();
        }
    }
    let original = document.entries();
    for (line, _) in updates {
        let Some(entry) = entries.iter().find(|entry| entry.line == *line) else {
            continue;
        };
        let Some(old) = original.iter().find(|entry| entry.line == *line) else {
            continue;
        };
        if values_equal(&entry.value, &old.value) && values_equal(&old.value, &entry.value) {
            continue;
        }
        if entries.iter().any(|other| {
            other.line != *line
                && other.section.eq_ignore_ascii_case(&entry.section)
                && base_key(&other.key) == base_key(&entry.key)
                && !other.key.starts_with(['!', '-'])
                && (entry.key.starts_with('+') || other.key.starts_with('+'))
                && editable(document, other)
                && values_equal(&entry.value, &other.value)
                && values_equal(&other.value, &entry.value)
        }) {
            return Err(crate::i18n::t("Изменение создаёт одинаковые привязки: операция + объединит записи. Измените набор правок", "This change creates identical bindings: the + operation would merge entries. Adjust the selected changes"));
        }
    }
    Ok(())
}

pub(crate) fn validate_update(
    document: &Document,
    entry: &Entry,
    value: &str,
) -> Result<(), AppInvokeError> {
    let fail = || {
        AppError::validation(crate::i18n::t(
            "Недопустимая или неоднозначная привязка Input.ini",
            "Invalid or ambiguous Input.ini binding",
        ))
    };
    if !editable(document, entry) || !crate::fs_util::is_safe_ini_value(value) {
        return Err(fail());
    }
    let old = fields(&entry.value).ok_or_else(fail)?;
    let new = fields(value).ok_or_else(fail)?;
    if old.keys().collect::<Vec<_>>() != new.keys().collect::<Vec<_>>() {
        return Err(fail());
    }
    let allowed: &[&str] = match base_key(&entry.key) {
        "ActionMappings" => &["Key", "bShift", "bCtrl", "bAlt", "bCmd"],
        "AxisMappings" => &["Key", "Scale"],
        "AxisConfig" => &["AxisProperties"],
        _ => return Err(fail()),
    };
    for (key, value) in &new {
        if !allowed.contains(&key.as_str()) && old.get(key) != Some(value) {
            return Err(fail());
        }
        if key == "Scale" && !value.parse::<f64>().is_ok_and(|value| value.is_finite()) {
            return Err(fail());
        }
        if key.starts_with('b')
            && allowed.contains(&key.as_str())
            && !matches!(value.to_ascii_lowercase().as_str(), "true" | "false")
        {
            return Err(fail());
        }
        if key == "Key"
            && (value.trim_matches('"').is_empty()
                || !value
                    .trim_matches('"')
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_'))
        {
            return Err(fail());
        }
        if key == "AxisProperties" {
            let props = fields(value).ok_or_else(fail)?;
            let old_props = old
                .get(key)
                .and_then(|value| fields(value))
                .ok_or_else(fail)?;
            if props.keys().collect::<Vec<_>>() != old_props.keys().collect::<Vec<_>>() {
                return Err(fail());
            }
            for (name, value) in &props {
                match name.as_str() {
                    "DeadZone" | "Sensitivity" | "Exponent" => {
                        let value = value.parse::<f64>().map_err(|_| fail())?;
                        if !value.is_finite()
                            || value < 0.0
                            || (name == "DeadZone" && value > 1.0)
                            || (name == "Exponent" && value == 0.0)
                        {
                            return Err(fail());
                        }
                    }
                    "bInvert"
                        if matches!(value.to_ascii_lowercase().as_str(), "true" | "false") => {}
                    _ if old_props.get(name) == Some(value) => {}
                    _ => return Err(fail()),
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn get_input_document(
    game_id: String,
    config_dir: String,
) -> Result<InputDocument, AppInvokeError> {
    crate::commands::helpers::guard_config_dir_for_read(Some(&game_id), &config_dir)?;
    let profile = crate::commands::helpers::find_profile_by_id(&game_id)?
        .ok_or_else(|| AppError::validation("Game not found"))?;
    let hints =
        crate::discovery::platform_hints_for_game(Some(&game_id), Some(&profile.engine_family));
    let dir = crate::ini::platform::reconcile_config_dir(
        &crate::ini::paths::inspect_config_dir(&config_dir)?,
        &hints,
    );
    let bytes = crate::changes::read_optional(&dir, "Input.ini")?.unwrap_or_default();
    let (text, _) = crate::ini::encoding::decode_bytes(&bytes)?;
    let document = Document::parse(&text);
    let entries = identified_entries(&document)
        .into_iter()
        .map(|(entry, id)| {
            let can_edit = editable(&document, &entry);
            let parsed = fields(&entry.value).unwrap_or_default();
            let axis_properties = parsed.get("AxisProperties").and_then(|value| fields(value));
            InputEntry {
                id,
                line: entry.line as u32,
                key: entry.key,
                fields: parsed,
                axis_properties,
                value: entry.value,
                editable: can_edit,
                reason: (!can_edit).then(|| {
                    crate::i18n::t(
                        "Неизвестная цепочка наследования или операция массива",
                        "Unknown inheritance chain or array operation",
                    )
                }),
            }
        })
        .collect();
    Ok(InputDocument {
        revision: crate::changes::digest(&bytes),
        entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn later_clear_makes_previous_array_entries_read_only() {
        let doc = Document::parse("[/Script/Engine.InputSettings]\n!AxisMappings=ClearArray\n+AxisMappings=(AxisName=Forward,Key=W,Scale=1)\n!AxisMappings=ClearArray\n+AxisMappings=(AxisName=Forward,Key=S,Scale=1)\n");
        assert!(!editable(&doc, &doc.entries()[1]));
        assert!(editable(&doc, &doc.entries()[3]));
    }
    #[test]
    fn selected_binding_swap_is_valid_but_partial_swap_would_merge() {
        let first = "(AxisName=Forward,Key=W,Scale=1)";
        let second = "(AxisName=Forward,Key=S,Scale=1)";
        let doc = Document::parse(&format!("[/Script/Engine.InputSettings]\n!AxisMappings=ClearArray\n+AxisMappings={first}\n+AxisMappings={second}\n"));
        assert!(validate_array_uniqueness(&doc, &[(2, second.into())]).is_err());
        assert!(validate_array_uniqueness(&doc, &[(2, second.into()), (3, first.into())]).is_ok());
    }
    #[test]
    fn diagnostic_binding_equality_uses_ue_values() {
        assert!(values_equal(
            "(AxisName=Forward,Key=W,Scale=1)",
            "(Scale=1.000000,Key=\"w\",AxisName=\"forward\")"
        ));
        assert!(values_equal(
            "(AxisKeyName=MouseX,AxisProperties=(Sensitivity=1,bInvert=False))",
            "(AxisKeyName=\"MouseX\",AxisProperties=(bInvert=false,Sensitivity=1.0,DeadZone=0))"
        ));
        assert!(!values_equal(
            "(AxisName=Forward,Key=W,Scale=1)",
            "(AxisName=Forward,Key=W,Scale=-1)"
        ));
    }
    #[test]
    fn nested_fields_and_quoted_names() {
        let parsed =
            fields("(AxisName=\"Move,Forward\",AxisProperties=(Sensitivity=1,DeadZone=0),Key=W)")
                .unwrap();
        assert_eq!(parsed["AxisName"], "\"Move,Forward\"");
        assert_eq!(parsed["AxisProperties"], "(Sensitivity=1,DeadZone=0)");
        assert!(fields("(Key=W,Key=S)").is_none());
    }
    #[test]
    fn inherited_arrays_are_read_only_but_local_arrays_can_change() {
        let unknown = Document::parse(
            "[/Script/Engine.InputSettings]\n+AxisMappings=(AxisName=Forward,Key=W,Scale=1)\n",
        );
        assert!(!editable(&unknown, &unknown.entries()[0]));
        let local = Document::parse("[/Script/Engine.InputSettings]\n!AxisMappings=ClearArray\n+AxisMappings=(AxisName=Forward,Key=W,Scale=1)\n+AxisMappings=(AxisName=Forward,Key=S,Scale=-1)\n");
        assert!(validate_update(
            &local,
            &local.entries()[1],
            "(AxisName=Forward,Key=Up,Scale=1)"
        )
        .is_ok());
        assert!(validate_update(
            &local,
            &local.entries()[1],
            "(AxisName=Other,Key=Up,Scale=1)"
        )
        .is_err());
    }
}
