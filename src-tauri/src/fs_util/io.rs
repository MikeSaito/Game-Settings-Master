use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

#[cfg(windows)]
const FILE_SHARE_READ: u32 = 0x0000_0001;
#[cfg(windows)]
const FILE_SHARE_WRITE: u32 = 0x0000_0002;
#[cfg(windows)]
const FILE_SHARE_DELETE: u32 = 0x0000_0004;

pub fn read_file_bytes(path: &Path) -> Result<Vec<u8>, String> {
    clear_readonly(path);

    #[cfg(windows)]
    if let Ok(bytes) = read_file_shared(path) {
        return Ok(bytes);
    }

    fs::read(path).map_err(|e| format_io_error("прочитать", "read", path, e))
}

/// Strips UTF-8 BOM (often added by PowerShell `Set-Content -Encoding UTF8`).
pub fn strip_utf8_bom(bytes: &[u8]) -> &[u8] {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        bytes
    }
}

/// Reads a text file as UTF-8; second element is whether a BOM was present (for auto-fix).
pub fn read_utf8_text_file(path: &Path) -> Result<(String, bool), String> {
    let bytes = read_file_bytes(path)?;
    let had_bom = bytes.starts_with(&[0xEF, 0xBB, 0xBF]);
    let text = std::str::from_utf8(strip_utf8_bom(&bytes))
        .map_err(|e| {
            crate::i18n::t(
                &format!("Файл не в UTF-8 ({}): {e}", path.display()),
                &format!("File is not UTF-8 ({}): {e}", path.display()),
            )
        })?
        .to_string();
    Ok((text, had_bom))
}

pub fn write_file_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    write_file_bytes_opts(path, bytes, false)
}

pub fn write_file_bytes_opts(path: &Path, bytes: &[u8], atomic: bool) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось создать каталог {}: {e}", parent.display()),
                &format!("Failed to create directory {}: {e}", parent.display()),
            )
        })?;
    }

    if atomic {
        return write_file_bytes_atomic(path, bytes);
    }

    clear_readonly(path);

    #[cfg(windows)]
    if write_file_shared(path, bytes).is_ok() {
        return Ok(());
    }

    fs::write(path, bytes).map_err(|e| format_io_error("записать", "write", path, e))
}

fn write_file_bytes_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
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
pub fn clear_readonly(path: &Path) {
    if !path.exists() {
        return;
    }
    if let Ok(meta) = fs::metadata(path) {
        let mut perms = meta.permissions();
        if perms.readonly() {
            #[allow(clippy::permissions_set_readonly_false)]
            perms.set_readonly(false);
            let _ = fs::set_permissions(path, perms);
        }
    }
}

pub fn format_io_error(
    action_ru: &str,
    action_en: &str,
    path: &Path,
    err: std::io::Error,
) -> String {
    let action = crate::i18n::t(action_ru, action_en);
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());
    let (hint_ru, hint_en) = io_error_hint(&err);
    crate::i18n::t(
        &format!("Не удалось {action} {name}: {err}{hint_ru}"),
        &format!("Failed to {action} {name}: {err}{hint_en}"),
    )
}

fn io_error_hint(err: &std::io::Error) -> (&'static str, &'static str) {
    match err.raw_os_error() {
        Some(5) => (
            ". Доступ запрещён. Полностью закройте игру и лаунчер (Steam/Epic), отключите \
             игровые оверлеи (Steam/Discord/NVIDIA) и проверьте антивирус. Если игра в защищённой \
             папке (Program Files) — запустите приложение от имени администратора. \
             Также снимите атрибут «Только чтение» с файла.",
            ". Access denied. Fully close the game and launcher (Steam/Epic), disable \
             game overlays (Steam/Discord/NVIDIA), and check your antivirus. If the game is in a \
             protected folder (Program Files), run the app as administrator. \
             Also remove the Read-only attribute from the file.",
        ),
        Some(32) => (
            ". Файл занят другим процессом — закройте игру, лаунчер и оверлеи, затем повторите.",
            ". The file is in use by another process — close the game, launcher, and overlays, then try again.",
        ),
        _ => ("", ""),
    }
}
#[cfg(windows)]
fn read_file_shared(path: &Path) -> Result<Vec<u8>, std::io::Error> {
    use std::fs::OpenOptions;

    let mut file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .open(path)?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

#[cfg(windows)]
fn write_file_shared(path: &Path, bytes: &[u8]) -> Result<(), std::io::Error> {
    use std::fs::OpenOptions;

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .open(path)?;

    file.write_all(bytes)?;
    file.flush()?;
    Ok(())
}
