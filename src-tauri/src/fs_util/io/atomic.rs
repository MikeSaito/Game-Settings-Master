use super::permissions::{clear_readonly, format_io_error};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

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
    let tmp = parent.join(format!(
        ".{file_name}.tmp-{}",
        uuid::Uuid::new_v4().simple()
    ));

    let write_result = (|| -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&tmp)
            .map_err(|e| format_io_error("создать", "create", &tmp, e))?;
        file.write_all(bytes)
            .map_err(|e| format_io_error("записать", "write", &tmp, e))?;
        file.sync_all()
            .map_err(|e| format_io_error("синхронизировать", "sync", &tmp, e))?;
        drop(file);

        replace_file(&tmp, path)
    })();

    if write_result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    write_result
}

#[cfg(windows)]
fn replace_file(tmp: &Path, path: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    let tmp_wide: Vec<u16> = tmp.as_os_str().encode_wide().chain(Some(0)).collect();
    let path_wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let move_file = || unsafe {
        MoveFileExW(
            tmp_wide.as_ptr(),
            path_wide.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    let mut result = move_file();
    let mut error = std::io::Error::last_os_error();

    if result == 0 && error.raw_os_error() == Some(5) {
        let original_permissions = fs::metadata(path).ok().map(|meta| meta.permissions());
        let was_readonly = original_permissions
            .as_ref()
            .is_some_and(std::fs::Permissions::readonly);
        if was_readonly {
            clear_readonly(path);
            result = move_file();
            if result == 0 {
                error = std::io::Error::last_os_error();
                if let Some(permissions) = original_permissions {
                    let _ = fs::set_permissions(path, permissions);
                }
            }
        }
    }
    if result == 0 {
        return Err(format_io_error("заменить", "replace", path, error));
    }
    Ok(())
}

#[cfg(not(windows))]
fn replace_file(tmp: &Path, path: &Path) -> Result<(), String> {
    fs::rename(tmp, path).map_err(|e| format_io_error("заменить", "replace", path, e))?;
    if let Some(parent) = path.parent() {
        if let Ok(dir) = fs::File::open(parent) {
            let _ = dir.sync_all();
        }
    }
    Ok(())
}
