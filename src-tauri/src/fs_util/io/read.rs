use super::permissions::{clear_readonly, format_io_error};
use std::fs;
use std::io::Read;
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

#[cfg(windows)]
pub(crate) fn read_file_shared(path: &Path) -> Result<Vec<u8>, std::io::Error> {
    use std::fs::OpenOptions;

    let mut file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .open(path)?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}
