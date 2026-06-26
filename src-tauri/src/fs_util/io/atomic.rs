use super::permissions::{clear_readonly, format_io_error};
use super::write::write_file_shared;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn write_file_bytes_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| {
        crate::i18n::t(
            &format!("Не удалось определить каталог для {}", path.display()),
            &format!("Failed to determine directory for {}", path.display()),
        )
    })?;
    let file_name = path.file_name().and_then(|s| s.to_str()).ok_or_else(|| {
        crate::i18n::t(
            &format!("Некорректное имя файла: {}", path.display()),
            &format!("Invalid file name: {}", path.display()),
        )
    })?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp = parent.join(format!(".{file_name}.tmp-{nonce}"));

    clear_readonly(&tmp);
    clear_readonly(path);

    #[cfg(windows)]
    {
        if write_file_shared(&tmp, bytes).is_err() {
            fs::write(&tmp, bytes).map_err(|e| format_io_error("записать", "write", &tmp, e))?;
        }
    }
    #[cfg(not(windows))]
    {
        fs::write(&tmp, bytes).map_err(|e| format_io_error("записать", "write", &tmp, e))?;
    }

    if path.exists() {
        clear_readonly(path);
        fs::remove_file(path).map_err(|e| format_io_error("заменить", "replace", path, e))?;
    }
    fs::rename(&tmp, path).map_err(|e| format_io_error("заменить", "replace", path, e))?;
    Ok(())
}
