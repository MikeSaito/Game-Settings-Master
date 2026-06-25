use super::atomic::write_file_bytes_atomic;
use super::permissions::{clear_readonly, format_io_error};
use std::fs;
use std::io::Write;
use std::path::Path;

#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

#[cfg(windows)]
const FILE_SHARE_READ: u32 = 0x0000_0001;
#[cfg(windows)]
const FILE_SHARE_WRITE: u32 = 0x0000_0002;

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

#[cfg(windows)]
pub(crate) fn write_file_shared(path: &Path, bytes: &[u8]) -> Result<(), std::io::Error> {
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
