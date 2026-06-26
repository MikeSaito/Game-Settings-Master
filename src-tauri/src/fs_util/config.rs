use std::fs;
use std::path::Path;

use super::io::{clear_readonly, read_file_bytes, write_file_bytes};
use super::path_safety::ALLOWED_CONFIG_INI_FILES;
use super::process::is_exe_running_uncached;
use crate::core::app_error::AppInvokeError;

pub fn clear_config_readonly(config_dir: &Path) {
    for file in ALLOWED_CONFIG_INI_FILES {
        clear_readonly(&config_dir.join(file));
    }
}

pub fn ensure_config_writable(
    config_dir: &Path,
    exe_name: Option<&str>,
) -> Result<(), AppInvokeError> {
    if let Some(exe) = exe_name {
        if is_exe_running_uncached(exe) {
            return Err(crate::core::app_error::running_game_ini_blocked(exe));
        }
    }

    clear_config_readonly(config_dir);

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

fn probe_config_dir_writable(config_dir: &Path) -> Result<(), AppInvokeError> {
    let probe = config_dir.join(".uesm-write-test");
    write_file_bytes(&probe, b"ok").map_err(|e| {
        crate::i18n::t(
            &format!(
                "Папка config недоступна для записи ({}): {e}. Закройте игру или запустите приложение от администратора.",
                config_dir.display()
            ),
            &format!(
                "Config folder is not writable ({}): {e}. Close the game or run the app as administrator.",
                config_dir.display()
            ),
        )
    })?;
    let _ = fs::remove_file(&probe);
    Ok(())
}
