use std::fs;

use crate::core::models::{GameOverride, SavedOverrides};

use super::storage::{overrides_path, write_json_atomic};

const MAX_OVERRIDE_JSON_BYTES: usize = 512 * 1024;
const MAX_OVERRIDES_JSON_BYTES: usize = 1024 * 1024;
const MAX_SAVED_OVERRIDES: usize = 256;

pub fn validate_override_bounds(override_def: &GameOverride) -> Result<(), String> {
    if let Some(meta) = &override_def.metadata {
        if meta.format_version != 2
            || !matches!(meta.mode.as_str(), "changes" | "profile")
            || meta.description.len() > 4096
            || meta.source_game_id.len() > 128
            || meta.source_game_name.len() > 256
            || meta
                .min_dedicated_memory_mb
                .is_some_and(|value| value > 1024 * 1024)
            || [&meta.game_build, &meta.engine_family, &meta.engine_version]
                .iter()
                .any(|value| value.as_ref().is_some_and(|value| value.len() > 256))
            || meta.gpu_vendor.as_ref().is_some_and(|value| {
                !matches!(value.as_str(), "nvidia" | "amd" | "intel" | "unknown")
            })
        {
            return Err(crate::i18n::t(
                "Недопустимые метаданные пресета или версия формата",
                "Invalid preset metadata or format version",
            ));
        }
    }
    if let Some(updates) = &override_def.input_updates {
        if updates.len() > 512
            || updates.iter().any(|update| {
                update.id.as_ref().is_some_and(|id| {
                    id.len() > 128 || !id.chars().all(|ch| ch.is_ascii_hexdigit() || ch == ':')
                }) || !crate::fs_util::is_safe_ini_value(&update.value)
                    || !crate::fs_util::is_safe_ini_value(&update.expected)
            })
        {
            return Err(crate::i18n::t(
                "Недопустимые записи Input.ini",
                "Invalid Input.ini entries",
            ));
        }
    }
    if override_def.game_id.trim().is_empty() {
        return Err(crate::i18n::t(
            "game_id override не указан",
            "game_id override is not specified",
        ));
    }
    if override_def.name.trim().is_empty() || override_def.name.len() > 120 {
        return Err(crate::i18n::t(
            "Недопустимое имя override",
            "Invalid override name",
        ));
    }
    let raw = serde_json::to_string(override_def).map_err(|e| e.to_string())?;
    if raw.len() > MAX_OVERRIDE_JSON_BYTES {
        return Err(crate::i18n::t(
            "Override слишком большой",
            "Override is too large",
        ));
    }
    Ok(())
}

pub(crate) fn validate_override_payload(override_def: &GameOverride) -> Result<(), String> {
    validate_override_bounds(override_def)?;
    for filename in override_def.files.keys() {
        if !crate::fs_util::is_allowed_config_ini_filename(filename) {
            return Err(crate::i18n::t(
                &format!("Недопустимый ini в override: {filename}"),
                &format!("Invalid ini in override: {filename}"),
            ));
        }
    }
    for filename in override_def.removals.keys() {
        if !crate::fs_util::is_allowed_config_ini_filename(filename) {
            return Err(crate::i18n::t(
                &format!("Недопустимый ini в removals: {filename}"),
                &format!("Invalid ini in removals: {filename}"),
            ));
        }
    }
    for (filename, sections) in &override_def.files {
        for (section, entries) in sections {
            if !crate::fs_util::is_safe_ini_section_name(section) {
                return Err(crate::i18n::t(
                    &format!("Недопустимая INI-секция в override {filename}: {section}"),
                    &format!("Invalid INI section in override {filename}: {section}"),
                ));
            }
            for (key, value) in entries {
                if !crate::fs_util::is_safe_ini_key_name(key) {
                    return Err(crate::i18n::t(
                        &format!("Недопустимый INI-ключ в override {filename}: {key}"),
                        &format!("Invalid INI key in override {filename}: {key}"),
                    ));
                }
                if !crate::fs_util::is_safe_ini_value(value) {
                    return Err(crate::i18n::t(
                        &format!("Недопустимое INI-значение для {key}"),
                        &format!("Invalid INI value for {key}"),
                    ));
                }
            }
        }
    }
    for (filename, sections) in &override_def.removals {
        for (section, keys) in sections {
            if !crate::fs_util::is_safe_ini_section_name(section) {
                return Err(crate::i18n::t(
                    &format!("Недопустимая INI-секция в removals {filename}: {section}"),
                    &format!("Invalid INI section in removals {filename}: {section}"),
                ));
            }
            for key in keys {
                if !crate::fs_util::is_safe_ini_key_name(key) {
                    return Err(crate::i18n::t(
                        &format!("Недопустимый INI-ключ в removals {filename}: {key}"),
                        &format!("Invalid INI key in removals {filename}: {key}"),
                    ));
                }
            }
        }
    }
    Ok(())
}

pub fn load_overrides() -> Result<Vec<GameOverride>, String> {
    let path = overrides_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
    if meta.len() as usize > MAX_OVERRIDES_JSON_BYTES {
        return Err(crate::i18n::t(
            &format!(
                "overrides.json слишком большой ({} KB, лимит {} KB)",
                meta.len() / 1024,
                MAX_OVERRIDES_JSON_BYTES / 1024
            ),
            &format!(
                "overrides.json is too large ({} KB, limit {} KB)",
                meta.len() / 1024,
                MAX_OVERRIDES_JSON_BYTES / 1024
            ),
        ));
    }
    let (content, had_bom) = crate::fs_util::read_utf8_text_file(&path)?;
    let saved: SavedOverrides = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    if saved.overrides.len() > MAX_SAVED_OVERRIDES {
        return Err(crate::i18n::t(
            &format!(
                "Слишком много override ({} > {MAX_SAVED_OVERRIDES})",
                saved.overrides.len()
            ),
            &format!(
                "Too many overrides ({} > {MAX_SAVED_OVERRIDES})",
                saved.overrides.len()
            ),
        ));
    }
    if had_bom {
        write_json_atomic(&path, &content)?;
    }
    Ok(saved.overrides)
}

pub fn save_override(override_def: &GameOverride) -> Result<(), String> {
    validate_override_payload(override_def)?;
    let mut overrides = load_overrides()?;
    if let Some(existing) = overrides
        .iter_mut()
        .find(|o| o.game_id == override_def.game_id && o.name == override_def.name)
    {
        *existing = override_def.clone();
    } else {
        overrides.push(override_def.clone());
    }
    let path = overrides_path()?;
    let content =
        serde_json::to_string_pretty(&SavedOverrides { overrides }).map_err(|e| e.to_string())?;
    write_json_atomic(&path, &content)
}

pub fn get_overrides_for_game(game_id: &str) -> Result<Vec<GameOverride>, String> {
    Ok(load_overrides()?
        .into_iter()
        .filter(|o| o.game_id == game_id)
        .collect())
}

pub fn delete_override(game_id: &str, name: &str) -> Result<(), String> {
    let mut overrides = load_overrides()?;
    overrides.retain(|o| !(o.game_id == game_id && o.name == name));
    let path = overrides_path()?;
    let content =
        serde_json::to_string_pretty(&SavedOverrides { overrides }).map_err(|e| e.to_string())?;
    write_json_atomic(&path, &content)
}
