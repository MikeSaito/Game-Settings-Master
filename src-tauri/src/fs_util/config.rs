use std::fs;
use std::path::Path;

use super::io::{read_file_bytes, write_file_bytes};
use super::process::is_exe_running_uncached;
use crate::core::app_error::AppInvokeError;

pub fn ensure_config_writable(
    config_dir: &Path,
    exe_name: Option<&str>,
) -> Result<(), AppInvokeError> {
    if let Some(exe) = exe_name {
        if is_exe_running_uncached(exe) {
            return Err(crate::core::app_error::running_game_ini_blocked(exe));
        }
    }

    for file in ["GameUserSettings.ini", "Engine.ini"] {
        let path = config_dir.join(file);
        if !path.exists() {
            continue;
        }
        super::ensure_safe_child_file(config_dir, &path)?;
        // A preflight must never rewrite files or change their attributes.
        read_file_bytes(&path)?;
    }

    probe_config_dir_writable(config_dir)?;

    Ok(())
}

fn probe_config_dir_writable(config_dir: &Path) -> Result<(), AppInvokeError> {
    crate::transaction::ensure_clear(config_dir)?;
    let probe = config_dir.join(format!(".uesm-write-test-{}", uuid::Uuid::new_v4()));
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
