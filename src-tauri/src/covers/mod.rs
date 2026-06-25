use std::fs;
use std::path::{Path, PathBuf};

const STEAM_CDN: &str = "https://cdn.cloudflare.steamstatic.com/steam/apps";

pub fn steam_header_url(app_id: &str) -> String {
    format!("{STEAM_CDN}/{app_id}/header.jpg")
}

pub fn covers_dir() -> Result<PathBuf, String> {
    let dir = dirs::data_dir()
        .ok_or_else(|| {
            crate::i18n::t(
                "Не удалось определить каталог AppData",
                "Failed to determine AppData directory",
            )
        })?
        .join("UESettingsMaster")
        .join("covers");
    fs::create_dir_all(&dir).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось создать каталог covers: {e}"),
            &format!("Failed to create covers directory: {e}"),
        )
    })?;
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
        return Err(crate::i18n::t(
            "Файл изображения не найден",
            "Image file not found",
        ));
    }
    if source.is_dir() {
        return Err(crate::i18n::t(
            "Укажите файл изображения, а не папку",
            "Specify an image file, not a folder",
        ));
    }
    let canonical = source.canonicalize().map_err(|e| {
        crate::i18n::t(
            &format!("Некорректный путь к изображению: {e}"),
            &format!("Invalid image path: {e}"),
        )
    })?;
    if canonical.is_dir() {
        return Err(crate::i18n::t(
            "Укажите файл изображения, а не папку",
            "Specify an image file, not a folder",
        ));
    }
    let size = fs::metadata(&canonical)
        .map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось прочитать изображение: {e}"),
                &format!("Failed to read image: {e}"),
            )
        })?
        .len();
    const MAX_COVER_BYTES: u64 = 10 * 1024 * 1024;
    if size > MAX_COVER_BYTES {
        return Err(crate::i18n::t(
            "Изображение слишком большое (макс. 10 МБ)",
            "Image is too large (max 10 MB)",
        ));
    }
    if canonical
        .symlink_metadata()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err(crate::i18n::t(
            "Символические ссылки не поддерживаются",
            "Symbolic links are not supported",
        ));
    }

    let ext = extension_from_path(&canonical).ok_or_else(|| {
        crate::i18n::t(
            "Поддерживаются PNG, JPG, WEBP, GIF",
            "Supported formats: PNG, JPG, WEBP, GIF",
        )
    })?;

    let dest = covers_dir()?.join(cover_filename(game_id, &ext));
    fs::copy(&canonical, &dest).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось сохранить обложку: {e}"),
            &format!("Failed to save cover: {e}"),
        )
    })?;

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

pub fn enrich_cover(profile: &mut crate::core::models::GameProfile) {
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
    existing: &mut crate::core::models::GameProfile,
    saved: &crate::core::models::GameProfile,
) {
    if let Some(custom) = &saved.custom_cover {
        if Path::new(custom).exists() {
            existing.custom_cover = Some(custom.clone());
        }
    }
}

#[cfg(test)]
#[path = "covers_tests.rs"]
mod tests;
