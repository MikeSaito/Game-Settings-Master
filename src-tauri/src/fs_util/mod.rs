use std::fs;
use std::io::{Read, Write};
use std::path::Path;

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

    fs::read(path).map_err(|e| format_io_error("прочитать", path, e))
}

pub fn write_file_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Не удалось создать каталог {}: {e}", parent.display()))?;
    }

    clear_readonly(path);

    #[cfg(windows)]
    if write_file_shared(path, bytes).is_ok() {
        return Ok(());
    }

    fs::write(path, bytes).map_err(|e| format_io_error("записать", path, e))
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

pub fn format_io_error(action: &str, path: &Path, err: std::io::Error) -> String {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());
    format!("Не удалось {action} {name}: {err}{}", io_error_hint(&err))
}

fn io_error_hint(err: &std::io::Error) -> String {
    match err.raw_os_error() {
        Some(5) => {
            ". Закройте игру полностью — ini-файлы (особенно Engine.ini) часто блокируются процессом. \
             Если игра закрыта: снимите «Только чтение» с файла или запустите приложение от администратора."
                .to_string()
        }
        Some(32) => ". Файл занят другим процессом — закройте игру и повторите.".to_string(),
        _ => String::new(),
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

#[cfg(windows)]
pub fn is_exe_running(exe_name: &str) -> bool {
    let filter = if exe_name.to_ascii_lowercase().ends_with(".exe") {
        exe_name.to_string()
    } else {
        format!("{exe_name}.exe")
    };

    std::process::Command::new("tasklist")
        .args(["/FI", &format!("IMAGENAME eq {filter}"), "/FO", "CSV", "/NH"])
        .output()
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.to_ascii_lowercase().contains(&filter.to_ascii_lowercase())
                && !stdout.contains("INFO: No tasks")
        })
        .unwrap_or(false)
}

#[cfg(not(windows))]
pub fn is_exe_running(_exe_name: &str) -> bool {
    false
}

#[cfg(windows)]
pub fn kill_exe(exe_name: &str) -> Result<(), String> {
    let filter = if exe_name.to_ascii_lowercase().ends_with(".exe") {
        exe_name.to_string()
    } else {
        format!("{exe_name}.exe")
    };

    let output = std::process::Command::new("taskkill")
        .args(["/F", "/IM", &filter])
        .output()
        .map_err(|e| format!("Не удалось завершить процесс: {e}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("not found") || stderr.contains("не найден") {
        return Err(format!("Процесс «{filter}» не найден."));
    }

    Err(format!(
        "Не удалось завершить «{filter}»: {}",
        stderr.trim()
    ))
}

#[cfg(not(windows))]
pub fn kill_exe(_exe_name: &str) -> Result<(), String> {
    Err("Завершение процесса поддерживается только в Windows.".to_string())
}

const CONFIG_INI_FILES: [&str; 5] = [
    "GameUserSettings.ini",
    "Engine.ini",
    "Game.ini",
    "Scalability.ini",
    "Input.ini",
];

pub fn clear_config_readonly(config_dir: &Path) {
    if crate::forza::is_forza_config_dir(config_dir) {
        clear_readonly(&crate::forza::user_config_file(config_dir));
        return;
    }
    for file in CONFIG_INI_FILES {
        clear_readonly(&config_dir.join(file));
    }
}

pub fn ensure_config_writable(config_dir: &Path, exe_name: Option<&str>) -> Result<(), String> {
    if let Some(exe) = exe_name {
        if is_exe_running(exe) {
            return Err(format!(
                "Игра «{exe}» запущена. Закройте игру перед применением — иначе ini-файлы заблокированы."
            ));
        }
    }

    clear_config_readonly(config_dir);

    if crate::forza::is_forza_config_dir(config_dir) {
        let path = crate::forza::user_config_file(config_dir);
        let bytes = read_file_bytes(&path)?;
        write_file_bytes(&path, &bytes)?;
        return Ok(());
    }

    for file in ["GameUserSettings.ini", "Engine.ini"] {
        let path = config_dir.join(file);
        if !path.exists() {
            continue;
        }
        let bytes = read_file_bytes(&path)?;
        write_file_bytes(&path, &bytes)?;
    }

    probe_config_dir_writable(config_dir)?;

    Ok(())
}

fn probe_config_dir_writable(config_dir: &Path) -> Result<(), String> {
    let probe = config_dir.join(".uesm-write-test");
    write_file_bytes(&probe, b"ok").map_err(|e| {
        format!(
            "Папка config недоступна для записи ({}): {e}. Закройте игру или запустите приложение от администратора.",
            config_dir.display()
        )
    })?;
    let _ = fs::remove_file(&probe);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn read_write_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.ini");
        write_file_bytes(&path, b"hello").unwrap();
        assert_eq!(read_file_bytes(&path).unwrap(), b"hello");
    }

    #[test]
    fn clears_readonly_before_write() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("ro.ini");
        std::fs::File::create(&path).unwrap().write_all(b"old").unwrap();
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&path, perms).unwrap();

        write_file_bytes(&path, b"new").unwrap();
        assert_eq!(read_file_bytes(&path).unwrap(), b"new");
    }
}
