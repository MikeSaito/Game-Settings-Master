use super::presets::{parse_parameters, parse_techniques, PresetParameter};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ReShadeBehaviorSettings {
    #[serde(default)]
    pub performance_mode: bool,
    #[serde(default)]
    pub key_overlay: Option<String>,
    #[serde(default)]
    pub key_toggle_effects: Option<String>,
}

const MAX_TECHNIQUES: usize = 32;
const MAX_OVERRIDE_EFFECTS: usize = 32;
const MAX_KEYS_PER_EFFECT: usize = 64;
const MAX_OVERRIDE_KEY_LEN: usize = 128;
const MAX_OVERRIDE_VALUE_LEN: usize = 256;

pub fn validate_preset_overrides(overrides: &PresetOverrides) -> Result<(), String> {
    if let Some(ref techniques) = overrides.techniques {
        if techniques.len() > MAX_TECHNIQUES {
            return Err(crate::i18n::t(
                &format!(
                    "Слишком много techniques в override ({} > {MAX_TECHNIQUES})",
                    techniques.len()
                ),
                &format!(
                    "Too many techniques in override ({} > {MAX_TECHNIQUES})",
                    techniques.len()
                ),
            ));
        }
        for technique in techniques {
            if technique.is_empty() || technique.len() > MAX_OVERRIDE_KEY_LEN {
                return Err(crate::i18n::t(
                    "Недопустимое имя technique в override",
                    "Invalid technique name in override",
                ));
            }
        }
    }
    if overrides.parameters.len() > MAX_OVERRIDE_EFFECTS {
        return Err(crate::i18n::t(
            &format!(
                "Слишком много эффектов в override ({} > {MAX_OVERRIDE_EFFECTS})",
                overrides.parameters.len()
            ),
            &format!(
                "Too many effects in override ({} > {MAX_OVERRIDE_EFFECTS})",
                overrides.parameters.len()
            ),
        ));
    }
    for (effect, keys) in &overrides.parameters {
        if effect.is_empty() || effect.len() > MAX_OVERRIDE_KEY_LEN {
            return Err(crate::i18n::t(
                "Недопустимое имя эффекта в override",
                "Invalid effect name in override",
            ));
        }
        if keys.len() > MAX_KEYS_PER_EFFECT {
            return Err(crate::i18n::t(
                &format!(
                    "Слишком много параметров для {effect} ({} > {MAX_KEYS_PER_EFFECT})",
                    keys.len()
                ),
                &format!(
                    "Too many parameters for {effect} ({} > {MAX_KEYS_PER_EFFECT})",
                    keys.len()
                ),
            ));
        }
        for (key, value) in keys {
            if key.is_empty() || key.len() > MAX_OVERRIDE_KEY_LEN {
                return Err(crate::i18n::t(
                    &format!("Недопустимый ключ параметра в {effect}"),
                    &format!("Invalid parameter key in {effect}"),
                ));
            }
            if value.len() > MAX_OVERRIDE_VALUE_LEN {
                return Err(crate::i18n::t(
                    &format!("Слишком длинное значение для {effect}.{key}"),
                    &format!("Value too long for {effect}.{key}"),
                ));
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct PresetOverrides {
    #[serde(default)]
    pub techniques: Option<Vec<String>>,
    #[serde(default)]
    pub parameters: HashMap<String, HashMap<String, String>>,
    #[serde(default)]
    pub behavior: Option<ReShadeBehaviorSettings>,
}

pub fn apply_overrides_to_preset(base_ini: &str, overrides: &PresetOverrides) -> String {
    let techniques = overrides
        .techniques
        .clone()
        .unwrap_or_else(|| parse_techniques(base_ini));
    let mut ini = set_techniques_in_ini(base_ini, &techniques);

    for (effect, keys) in &overrides.parameters {
        for (key, value) in keys {
            ini = set_parameter_in_ini(&ini, effect, key, value);
        }
    }

    ini
}

pub fn apply_behavior_to_base(base_ini: &str, behavior: &ReShadeBehaviorSettings) -> String {
    let mut lines: Vec<String> = base_ini.lines().map(str::to_string).collect();
    set_ini_value(&mut lines, "GENERAL", "PerformanceMode", if behavior.performance_mode { "1" } else { "0" });
    if let Some(key) = &behavior.key_overlay {
        set_ini_value(&mut lines, "INPUT", "KeyOverlay", key);
    }
    if let Some(key) = &behavior.key_toggle_effects {
        set_ini_value(&mut lines, "INPUT", "KeyEffects", key);
    }
    let mut out = lines.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

pub fn set_techniques_in_ini(ini: &str, techniques: &[String]) -> String {
    let params = parse_parameters(ini);
    build_preset_ini(techniques, &params)
}

pub fn set_parameter_in_ini(ini: &str, effect: &str, key: &str, value: &str) -> String {
    let techniques = parse_techniques(ini);
    let mut params = parse_parameters(ini);
    if let Some(p) = params.iter_mut().find(|p| p.effect == effect && p.key == key) {
        p.value = value.to_string();
    } else {
        params.push(PresetParameter {
            effect: effect.to_string(),
            key: key.to_string(),
            value: value.to_string(),
        });
    }
    build_preset_ini(&techniques, &params)
}

fn build_preset_ini(techniques: &[String], params: &[PresetParameter]) -> String {
    let mut out = String::new();
    if techniques.is_empty() {
        out.push_str("Techniques=\nTechniqueSorting=\n");
    } else {
        out.push_str(&format!("Techniques={}\n", techniques.join(",")));
        out.push_str(&format!(
            "TechniqueSorting={}\n",
            techniques
                .iter()
                .map(|t| format!("{t}@{t}"))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    let mut current_effect: Option<String> = None;
    for param in params {
        if current_effect.as_deref() != Some(param.effect.as_str()) {
            current_effect = Some(param.effect.clone());
            out.push_str(&format!("\n[{}.fx]\n", param.effect));
        }
        out.push_str(&format!("{}={}\n", param.key, param.value));
    }
    out
}

fn set_ini_value(lines: &mut Vec<String>, section: &str, key: &str, value: &str) {
    let section_header = format!("[{section}]");
    let mut in_section = false;
    let mut replaced = false;
    for line in lines.iter_mut() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed == section_header;
            continue;
        }
        if in_section && trimmed.starts_with(&format!("{key}=")) {
            *line = format!("{key}={value}");
            replaced = true;
            break;
        }
    }
    if !replaced {
        if let Some(idx) = lines.iter().position(|l| l.trim() == section_header) {
            lines.insert(idx + 1, format!("{key}={value}"));
        } else {
            lines.push(section_header);
            lines.push(format!("{key}={value}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn rejects_oversized_override_payload() {
        let mut parameters = HashMap::new();
        parameters.insert(
            "Clarity".to_string(),
            HashMap::from([("x".repeat(MAX_OVERRIDE_VALUE_LEN + 1), "1".to_string())]),
        );
        let overrides = PresetOverrides {
            techniques: None,
            parameters,
            behavior: None,
        };
        assert!(validate_preset_overrides(&overrides).is_err());
    }

    #[test]
    fn accepts_small_override_payload() {
        let mut parameters = HashMap::new();
        parameters.insert(
            "Clarity".to_string(),
            HashMap::from([("ClarityRadius".to_string(), "2.5".to_string())]),
        );
        let overrides = PresetOverrides {
            techniques: Some(vec!["Clarity".to_string()]),
            parameters,
            behavior: None,
        };
        assert!(validate_preset_overrides(&overrides).is_ok());
    }
}
