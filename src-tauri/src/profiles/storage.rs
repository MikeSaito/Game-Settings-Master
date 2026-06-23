use std::fs;
use std::path::PathBuf;

pub fn app_data_dir() -> Result<PathBuf, String> {
    let dir = dirs::data_dir()
        .ok_or_else(|| {
            crate::i18n::t(
                "Не удалось определить каталог AppData",
                "Failed to determine AppData directory",
            )
        })?
        .join("UESettingsMaster");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub(crate) fn profiles_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("games.json"))
}

pub(crate) fn overrides_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("overrides.json"))
}

pub(crate) fn write_json_atomic(path: &std::path::Path, content: &str) -> Result<(), String> {
    crate::fs_util::write_file_bytes_opts(path, content.as_bytes(), true)
}
