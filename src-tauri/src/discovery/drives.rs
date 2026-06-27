use std::path::PathBuf;

/// Local drive roots suitable for launcher library discovery (fixed + removable).
#[cfg(windows)]
pub fn scannable_drive_roots() -> Vec<PathBuf> {
    use windows_sys::Win32::Storage::FileSystem::{GetDriveTypeW, GetLogicalDrives};

    const DRIVE_REMOVABLE: u32 = 2;
    const DRIVE_FIXED: u32 = 3;

    let mask = unsafe { GetLogicalDrives() };
    if mask == 0 {
        return Vec::new();
    }

    let mut roots = Vec::new();
    for index in 0..26u32 {
        if mask & (1 << index) == 0 {
            continue;
        }
        let letter = (b'A' + index as u8) as char;
        let root = format!("{letter}:\\");
        let wide: Vec<u16> = root.encode_utf16().chain(std::iter::once(0)).collect();
        let drive_type = unsafe { GetDriveTypeW(wide.as_ptr()) };
        if drive_type != DRIVE_FIXED && drive_type != DRIVE_REMOVABLE {
            continue;
        }
        roots.push(PathBuf::from(root));
    }
    roots
}

#[cfg(not(windows))]
pub fn scannable_drive_roots() -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scannable_drive_roots_returns_unique_letters() {
        let roots = scannable_drive_roots();
        let letters: Vec<char> = roots
            .iter()
            .filter_map(|p| {
                p.to_str()
                    .and_then(|s| s.chars().next())
                    .map(|c| c.to_ascii_uppercase())
            })
            .collect();
        let mut deduped = letters.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(letters.len(), deduped.len());
    }
}
