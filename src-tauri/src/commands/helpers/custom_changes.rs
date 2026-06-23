use crate::app_error::AppError;
use crate::core::models::CustomChanges;
use crate::fs_util::{is_safe_ini_key_name, is_safe_ini_section_name, is_safe_ini_value};
use std::path::Path;

pub(crate) const MAX_CUSTOM_CHANGES_JSON_BYTES: usize = 256 * 1024;
pub(crate) const MAX_CUSTOM_CHANGE_FILES: usize = 16;

pub(crate) fn validate_custom_changes_payload(
    changes: &CustomChanges,
    _config_path: &Path,
) -> Result<(), String> {
    let file_count = changes.files.len() + changes.removals.len();
    if file_count > MAX_CUSTOM_CHANGE_FILES {
        return Err(AppError::validation(crate::i18n::t(
            &format!(
                "Слишком много файлов в custom apply ({file_count} > {MAX_CUSTOM_CHANGE_FILES})"
            ),
            &format!("Too many files in custom apply ({file_count} > {MAX_CUSTOM_CHANGE_FILES})"),
        ))
        .to_invoke_string());
    }
    let raw = serde_json::to_string(changes)
        .map_err(|e| AppError::validation(e.to_string()).to_invoke_string())?;
    if raw.len() > MAX_CUSTOM_CHANGES_JSON_BYTES {
        return Err(AppError::validation(crate::i18n::t(
            &format!(
                "Custom apply слишком большой ({} KB, лимит {} KB)",
                raw.len() / 1024,
                MAX_CUSTOM_CHANGES_JSON_BYTES / 1024
            ),
            &format!(
                "Custom apply is too large ({} KB, limit {} KB)",
                raw.len() / 1024,
                MAX_CUSTOM_CHANGES_JSON_BYTES / 1024
            ),
        ))
        .to_invoke_string());
    }
    for name in changes.files.keys().chain(changes.removals.keys()) {
        if !crate::fs_util::is_allowed_config_ini_filename(name) {
            return Err(AppError::validation(crate::i18n::t(
                &format!("Недопустимое имя ini-файла: {name}"),
                &format!("Invalid ini file name: {name}"),
            ))
            .to_invoke_string());
        }
    }
    for (file, sections) in &changes.files {
        for (section, entries) in sections {
            if !is_safe_ini_section_name(section) {
                return Err(AppError::validation(crate::i18n::t(
                    &format!("Недопустимая INI-секция в {file}: {section}"),
                    &format!("Invalid INI section in {file}: {section}"),
                ))
                .to_invoke_string());
            }
            for (key, value) in entries {
                if !is_safe_ini_key_name(key) {
                    return Err(AppError::validation(crate::i18n::t(
                        &format!("Недопустимый INI-ключ в {file}: {key}"),
                        &format!("Invalid INI key in {file}: {key}"),
                    ))
                    .to_invoke_string());
                }
                if !is_safe_ini_value(value) {
                    return Err(AppError::validation(crate::i18n::t(
                        &format!("Недопустимое INI-значение для {key}"),
                        &format!("Invalid INI value for {key}"),
                    ))
                    .to_invoke_string());
                }
            }
        }
    }
    for (file, sections) in &changes.removals {
        for (section, keys) in sections {
            if !is_safe_ini_section_name(section) {
                return Err(AppError::validation(crate::i18n::t(
                    &format!("Недопустимая INI-секция в removals {file}: {section}"),
                    &format!("Invalid INI section in removals {file}: {section}"),
                ))
                .to_invoke_string());
            }
            for key in keys {
                if !is_safe_ini_key_name(key) {
                    return Err(AppError::validation(crate::i18n::t(
                        &format!("Недопустимый INI-ключ в removals {file}: {key}"),
                        &format!("Invalid INI key in removals {file}: {key}"),
                    ))
                    .to_invoke_string());
                }
            }
        }
    }
    Ok(())
}
