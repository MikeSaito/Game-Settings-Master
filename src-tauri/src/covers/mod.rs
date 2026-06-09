use std::fs;
use std::path::{Path, PathBuf};

const STEAM_CDN: &str = "https://cdn.cloudflare.steamstatic.com/steam/apps";

pub fn steam_header_url(app_id: &str) -> String {
    format!("{STEAM_CDN}/{app_id}/header.jpg")
}

pub fn covers_dir() -> Result<PathBuf, String> {
    let dir = dirs::data_dir()
        .ok_or_else(|| "Не удалось определить каталог AppData".to_string())?
        .join("UESettingsMaster")
        .join("covers");
    fs::create_dir_all(&dir).map_err(|e| format!("Не удалось создать каталог covers: {e}"))?;
    Ok(dir)
}

fn cover_filename(game_id: &str, ext: &str) -> String {
    let safe: String = game_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("{safe}.{ext}")
}

fn normalize_extension(ext: &str) -> &str {
    if ext.eq_ignore_ascii_case("jpeg") {
        "jpg"
    } else {
        ext
    }
}

fn extension_from_path(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .filter(|e| matches!(e.as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif"))
        .map(|e| normalize_extension(&e).to_string())
}

pub fn import_custom_cover(game_id: &str, source: &Path) -> Result<String, String> {
    if !source.exists() {
        return Err("Файл изображения не найден".to_string());
    }

    let ext = extension_from_path(source)
        .ok_or_else(|| "Поддерживаются PNG, JPG, WEBP, GIF".to_string())?;

    let dest = covers_dir()?.join(cover_filename(game_id, &ext));
    fs::copy(source, &dest).map_err(|e| format!("Не удалось сохранить обложку: {e}"))?;

    Ok(dest.to_string_lossy().to_string())
}

pub fn remove_custom_cover(game_id: &str) -> Result<(), String> {
    let dir = covers_dir()?;
    let prefix = cover_filename(game_id, "");
    let prefix = prefix.trim_end_matches('.');

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(prefix) && name.contains('.') {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    Ok(())
}

pub fn enrich_cover(profile: &mut crate::models::GameProfile) {
    if profile.custom_cover.is_some() {
        return;
    }

    if profile.cover_url.is_some() {
        return;
    }

    if let Some(app_id) = profile.id.strip_prefix("steam-") {
        profile.cover_url = Some(steam_header_url(app_id));
    }
}

pub fn merge_saved_cover(
    existing: &mut crate::models::GameProfile,
    saved: &crate::models::GameProfile,
) {
    if let Some(custom) = &saved.custom_cover {
        if Path::new(custom).exists() {
            existing.custom_cover = Some(custom.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_header_url_format() {
        assert_eq!(
            steam_header_url("1962700"),
            "https://cdn.cloudflare.steamstatic.com/steam/apps/1962700/header.jpg"
        );
    }
}
