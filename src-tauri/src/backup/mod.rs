use crate::fs_util::{read_file_bytes, write_file_bytes};
use chrono::Local;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

const BACKUP_DIR_LEGACY: &str = ".uesm-backups";
const INI_FILES: [&str; 5] = [
    "GameUserSettings.ini",
    "Engine.ini",
    "Game.ini",
    "Scalability.ini",
    "Input.ini",
];

/// Override-файлы пресетов; при сбросе удаляются, GameUserSettings.ini остаётся.
pub const OVERRIDE_INI_FILES: [&str; 4] =
    ["Engine.ini", "Game.ini", "Scalability.ini", "Input.ini"];

pub fn backup_store_dir(config_dir: &Path) -> PathBuf {
    let canonical = config_dir
        .canonicalize()
        .unwrap_or_else(|_| config_dir.to_path_buf());

    let mut hasher = DefaultHasher::new();
    canonical.to_string_lossy().hash(&mut hasher);
    let id = format!("{:016x}", hasher.finish());

    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ue-settings-master")
        .join("backups")
        .join(id)
}

fn legacy_backup_root(config_dir: &Path) -> PathBuf {
    config_dir.join(BACKUP_DIR_LEGACY)
}

pub fn backup_forza_config_dir(config_dir: &Path) -> Result<String, String> {
    let backup_root = backup_store_dir(config_dir);
    fs::create_dir_all(&backup_root)
        .map_err(|e| format!("Не удалось создать каталог backup: {e}"))?;

    let backup_id = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_path = backup_root.join(&backup_id);
    fs::create_dir_all(&backup_path).map_err(|e| format!("Не удалось создать backup: {e}"))?;

    let src = crate::forza::user_config_file(config_dir);
    if src.is_file() {
        let dst = backup_path.join("UserConfigSelections");
        let bytes = read_file_bytes(&src)?;
        write_file_bytes(&dst, &bytes)
            .map_err(|e| format!("Не удалось сохранить backup UserConfigSelections: {e}"))?;
    }

    Ok(backup_id)
}

pub fn backup_config_dir(config_dir: &Path) -> Result<String, String> {
    if crate::forza::is_forza_config_dir(config_dir) {
        return backup_forza_config_dir(config_dir);
    }
    let backup_root = backup_store_dir(config_dir);
    fs::create_dir_all(&backup_root)
        .map_err(|e| format!("Не удалось создать каталог backup: {e}"))?;

    let backup_id = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_path = backup_root.join(&backup_id);
    fs::create_dir_all(&backup_path).map_err(|e| format!("Не удалось создать backup: {e}"))?;

    for file in INI_FILES {
        let src = config_dir.join(file);
        if !src.exists() {
            continue;
        }
        let dst = backup_path.join(file);
        let bytes = read_file_bytes(&src)?;
        write_file_bytes(&dst, &bytes)
            .map_err(|e| format!("Не удалось сохранить backup {file}: {e}"))?;
    }

    Ok(backup_id)
}

pub fn list_backups(config_dir: &Path) -> Result<Vec<(String, String, Vec<String>)>, String> {
    let mut backups = list_backups_in(&backup_store_dir(config_dir))?;

    let legacy = legacy_backup_root(config_dir);
    if legacy.exists() {
        let mut legacy_backups = list_backups_in(&legacy)?;
        for (id, _, _) in &backups {
            legacy_backups.retain(|(lid, _, _)| lid != id);
        }
        backups.extend(legacy_backups);
    }

    backups.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(backups)
}

fn list_backups_in(backup_root: &Path) -> Result<Vec<(String, String, Vec<String>)>, String> {
    if !backup_root.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    for entry in fs::read_dir(backup_root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        let mut files = Vec::new();
        for file in fs::read_dir(entry.path()).map_err(|e| e.to_string())? {
            let file = file.map_err(|e| e.to_string())?;
            if file.file_type().map_err(|e| e.to_string())?.is_file() {
                files.push(file.file_name().to_string_lossy().to_string());
            }
        }
        files.sort();
        backups.push((id.clone(), id, files));
    }

    Ok(backups)
}

fn resolve_backup_path(config_dir: &Path, backup_id: &str) -> Result<PathBuf, String> {
    let primary = backup_store_dir(config_dir).join(backup_id);
    if primary.exists() {
        return Ok(primary);
    }

    let legacy = legacy_backup_root(config_dir).join(backup_id);
    if legacy.exists() {
        return Ok(legacy);
    }

    Err(format!("Backup '{backup_id}' не найден"))
}

/// Удаляет override ini, оставляя только GameUserSettings.ini (настройки из меню игры).
pub fn reset_config_to_user_settings(config_dir: &Path) -> Result<(String, Vec<String>), String> {
    let backup_id = backup_config_dir(config_dir)?;
    let mut deleted = Vec::new();

    for file in OVERRIDE_INI_FILES {
        let path = config_dir.join(file);
        if !path.exists() {
            continue;
        }
        crate::fs_util::clear_readonly(&path);
        fs::remove_file(&path).map_err(|e| format!("Не удалось удалить {file}: {e}"))?;
        deleted.push(file.to_string());
    }

    Ok((backup_id, deleted))
}

pub fn restore_backup(config_dir: &Path, backup_id: &str) -> Result<Vec<String>, String> {
    let backup_path = resolve_backup_path(config_dir, backup_id)?;

    let mut restored = Vec::new();
    for entry in fs::read_dir(&backup_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_file() {
            let name = entry.file_name().to_string_lossy().to_string();
            let dst = config_dir.join(&name);
            let bytes = read_file_bytes(&entry.path())
                .map_err(|e| format!("Не удалось прочитать backup {name}: {e}"))?;
            write_file_bytes(&dst, &bytes)
                .map_err(|e| format!("Не удалось восстановить {name}: {e}"))?;
            restored.push(name);
        }
    }

    Ok(restored)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_store_dir_is_stable() {
        let dir = PathBuf::from(r"C:\Games\Test\Saved\Config\Windows");
        assert_eq!(backup_store_dir(&dir), backup_store_dir(&dir));
    }

    #[test]
    fn reset_keeps_gus_deletes_overrides() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = tmp.path();
        fs::write(config.join("GameUserSettings.ini"), b"[Settings]\nFoo=1\n").unwrap();
        fs::write(
            config.join("Engine.ini"),
            b"[SystemSettings]\nr.Streaming=1\n",
        )
        .unwrap();
        fs::write(config.join("Scalability.ini"), b"[ScalabilityGroups]\n").unwrap();

        let (_, deleted) = reset_config_to_user_settings(config).expect("reset");

        assert!(config.join("GameUserSettings.ini").exists());
        assert!(!config.join("Engine.ini").exists());
        assert!(!config.join("Scalability.ini").exists());
        assert!(deleted.contains(&"Engine.ini".to_string()));
        assert!(deleted.contains(&"Scalability.ini".to_string()));
    }
}
