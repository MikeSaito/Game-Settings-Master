use std::fs;
use std::path::{Component, Path, PathBuf};

use super::io::format_io_error;

/// Relative path from a pack manifest: no `..`, not absolute, normal components only.
pub fn is_safe_manifest_relative_path(rel: &str) -> bool {
    if rel.is_empty() {
        return false;
    }
    if rel.contains(':') || rel.contains("..") {
        return false;
    }
    let path = Path::new(rel);
    if path.is_absolute() {
        return false;
    }
    path.components().all(|c| matches!(c, Component::Normal(_)))
}

/// Flat INI filename inside a pack directory (`preset.ini` only).
pub fn is_safe_pack_ini_filename(name: &str) -> bool {
    is_safe_manifest_relative_path(name) && Path::new(name).components().count() == 1
}

/// UE config INI files that GSM may read or write.
pub const GAME_USER_SETTINGS_INI: &str = "GameUserSettings.ini";

pub const ALLOWED_CONFIG_INI_FILES: [&str; 6] = [
    GAME_USER_SETTINGS_INI,
    "Engine.ini",
    "Game.ini",
    "Scalability.ini",
    "Input.ini",
    "DeviceProfiles.ini",
];

/// Override ini removed on reset; GameUserSettings.ini is kept.
pub const OVERRIDE_INI_FILES: [&str; 5] = [
    "Engine.ini",
    "Game.ini",
    "Scalability.ini",
    "Input.ini",
    "DeviceProfiles.ini",
];

pub fn is_allowed_config_ini_filename(name: &str) -> bool {
    is_safe_pack_ini_filename(name) && ALLOWED_CONFIG_INI_FILES.contains(&name)
}

pub fn normalize_ini_section_name(section: &str) -> String {
    let trimmed = section.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed.len() >= 2 {
        trimmed[1..trimmed.len() - 1].trim().to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn is_safe_ini_section_name(section: &str) -> bool {
    let section = normalize_ini_section_name(section);
    !section.is_empty()
        && section.len() <= 256
        && !section
            .chars()
            .any(|c| c == '\0' || c == '\r' || c == '\n' || c == '[' || c == ']')
}

pub fn is_safe_ini_key_name(key: &str) -> bool {
    let key = key.trim();
    !key.is_empty()
        && key.len() <= 256
        && !key
            .chars()
            .any(|c| c.is_control() || matches!(c, '=' | '[' | ']'))
}

pub fn is_safe_ini_value(value: &str) -> bool {
    value.len() <= 8192 && !value.chars().any(|c| c == '\0' || c == '\r' || c == '\n')
}

/// Flat filename allowed when restoring from a backup snapshot.
pub fn is_allowed_restore_filename(name: &str) -> bool {
    if is_allowed_config_ini_filename(name) || name == "UserConfigSelections" {
        return true;
    }
    if name == "prefs" {
        return is_safe_pack_ini_filename(name);
    }
    if name.ends_with(".json") {
        return is_safe_pack_ini_filename(name);
    }
    false
}

/// Backup folder id from list/restore — no path separators or traversal.
pub fn is_safe_backup_id(id: &str) -> bool {
    if id.is_empty() || id.len() > 64 {
        return false;
    }
    id.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Process basename for running/kill checks — single filename, no path separators.
pub fn is_safe_exe_basename(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.len() > 260 {
        return false;
    }
    if trimmed.contains("..") {
        return false;
    }
    !trimmed.chars().any(|c| {
        c.is_control() || matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
    })
}

pub fn path_within_root(root: &Path, path: &Path) -> bool {
    let Ok(root_canon) = root.canonicalize() else {
        return false;
    };
    let Ok(path_canon) = path.canonicalize() else {
        return false;
    };
    path_canon.starts_with(&root_canon)
}

pub fn ensure_safe_child_file(root: &Path, path: &Path) -> Result<(), String> {
    let root_canon = root.canonicalize().map_err(|e| {
        crate::i18n::t(
            &format!("Некорректный корневой путь {}: {e}", root.display()),
            &format!("Invalid root path {}: {e}", root.display()),
        )
    })?;
    let parent = path.parent().ok_or_else(|| {
        crate::i18n::t(
            &format!("Не удалось определить каталог для {}", path.display()),
            &format!("Failed to determine directory for {}", path.display()),
        )
    })?;
    let parent_canon = parent.canonicalize().map_err(|e| {
        crate::i18n::t(
            &format!("Некорректный каталог {}: {e}", parent.display()),
            &format!("Invalid directory {}: {e}", parent.display()),
        )
    })?;
    if !parent_canon.starts_with(&root_canon) {
        return Err(crate::i18n::t(
            &format!("Путь выходит за пределы config root: {}", path.display()),
            &format!("Path escapes config root: {}", path.display()),
        ));
    }
    if path.exists() {
        let meta = fs::symlink_metadata(path)
            .map_err(|e| format_io_error("проверить", "check", path, e))?;
        if meta.file_type().is_symlink() {
            return Err(crate::i18n::t(
                &format!(
                    "Символические ссылки в config не поддерживаются: {}",
                    path.display()
                ),
                &format!("Symlinks in config are not supported: {}", path.display()),
            ));
        }
        let path_canon = path.canonicalize().map_err(|e| {
            crate::i18n::t(
                &format!("Некорректный путь {}: {e}", path.display()),
                &format!("Invalid path {}: {e}", path.display()),
            )
        })?;
        if !path_canon.starts_with(&root_canon) {
            return Err(crate::i18n::t(
                &format!("Путь выходит за пределы config root: {}", path.display()),
                &format!("Path escapes config root: {}", path.display()),
            ));
        }
    }
    Ok(())
}

pub fn safe_child_path(root: &Path, file_name: &str) -> Result<PathBuf, String> {
    if !is_allowed_config_ini_filename(file_name) {
        return Err(crate::i18n::t(
            &format!("Недопустимое имя ini-файла: {file_name}"),
            &format!("Invalid ini file name: {file_name}"),
        ));
    }
    let path = root.join(file_name);
    ensure_safe_child_file(root, &path)?;
    Ok(path)
}
