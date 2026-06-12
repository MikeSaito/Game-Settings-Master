use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

/// Minimum size of a real ReShade addon DLL (dev stubs are 2-byte "MZ").
pub const MIN_RESHADE_DLL_BYTES: u64 = 64 * 1024;
const BINARY_HASHES_JSON: &str = include_str!("../../presets/reshade/binary-hashes.json");

#[derive(Debug, Deserialize)]
struct BinaryHashes {
    files: std::collections::BTreeMap<String, String>,
}

pub fn is_valid_reshade_dll(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    fs_len(path).map(|n| n >= MIN_RESHADE_DLL_BYTES).unwrap_or(false) && has_mz_header(path)
}

pub fn is_valid_bundled_json(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let Ok(raw) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    v.get("layer")
        .and_then(|layer| layer.get("name"))
        .and_then(|name| name.as_str())
        .is_some_and(|name| !name.trim().is_empty())
}

pub fn is_valid_bundled_file(name: &str, path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".dll") {
        return is_valid_reshade_dll(path);
    }
    if lower.ends_with(".json") {
        return is_valid_bundled_json(path);
    }
    true
}

pub fn validate_bundled_file(name: &str, path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Err(crate::i18n::t(
            &format!("Файл «{name}» не найден в бандле ReShade."),
            &format!("File «{name}» not found in ReShade bundle."),
        ));
    }
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".dll") && !is_valid_reshade_dll(path) {
        let size = fs_len(path).unwrap_or(0);
        return Err(crate::i18n::t(
            &format!(
                "«{name}» в бандле GSM — заглушка ({size} байт), не настоящий ReShade addon. \
                 Скачайте addon с https://reshade.me и положите в src-tauri/presets/reshade/bin/ \
                 (см. ATTRIBUTION.txt). Без этого игра не запустится — proxy DLL ломает DXGI/D3D."
            ),
            &format!(
                "«{name}» in GSM bundle is a stub ({size} bytes), not a real ReShade addon. \
                 Download the addon from https://reshade.me and place it in src-tauri/presets/reshade/bin/ \
                 (see ATTRIBUTION.txt). Without it the game won't start — proxy DLL breaks DXGI/D3D."
            ),
        ));
    }
    if lower.ends_with(".json") && !is_valid_bundled_json(path) {
        return Err(crate::i18n::t(
            &format!("«{name}» в бандле повреждён или пуст."),
            &format!("«{name}» in bundle is corrupted or empty."),
        ));
    }
    if let Some(expected) = pinned_hash_for(name) {
        let actual = sha256_file(path)?;
        if !actual.eq_ignore_ascii_case(&expected) {
            return Err(crate::i18n::t(
                &format!(
                    "«{name}» не прошёл SHA256-проверку (ожидался {expected}, получен {actual}). \
                     Обновите бандл ReShade и binary-hashes.json."
                ),
                &format!(
                    "«{name}» failed SHA256 check (expected {expected}, got {actual}). \
                     Update the ReShade bundle and binary-hashes.json."
                ),
            ));
        }
    }
    Ok(())
}

pub fn is_installed_proxy_valid(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if name.ends_with(".dll") {
        return is_valid_reshade_dll(path);
    }
    if name.ends_with(".json") {
        return is_valid_bundled_json(path);
    }
    true
}

fn fs_len(path: &Path) -> Result<u64, std::io::Error> {
    std::fs::metadata(path).map(|m| m.len())
}

fn has_mz_header(path: &Path) -> bool {
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut header = [0u8; 2];
    file.read_exact(&mut header).is_ok() && header == *b"MZ"
}

fn pinned_hash_for(name: &str) -> Option<String> {
    let Ok(parsed) = parse_binary_hashes() else {
        return None;
    };
    parsed
        .files
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.to_string())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let data = std::fs::read(path).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось прочитать {:?}: {e}", path),
            &format!("Failed to read {:?}: {e}", path),
        )
    })?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

fn parse_binary_hashes() -> Result<BinaryHashes, serde_json::Error> {
    let raw = BINARY_HASHES_JSON.trim_start_matches('\u{feff}');
    serde_json::from_str(raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reshade::presets::bundled_file;
    use std::collections::BTreeMap;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_min_dll(path: &Path) {
        let bytes = vec![0u8; MIN_RESHADE_DLL_BYTES as usize];
        std::fs::write(path, &bytes).unwrap();
    }

    #[test]
    fn rejects_tiny_dll() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("dxgi.dll");
        std::fs::File::create(&path).unwrap().write_all(b"MZ").unwrap();
        assert!(!is_valid_reshade_dll(&path));
        assert!(!is_valid_bundled_file("dxgi.dll", &path));
        assert!(validate_bundled_file("dxgi.dll", &path).is_err());
    }

    #[test]
    fn rejects_dll_just_below_min_size() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("dxgi.dll");
        let bytes = vec![0u8; (MIN_RESHADE_DLL_BYTES - 1) as usize];
        std::fs::write(&path, &bytes).unwrap();
        assert!(!is_valid_reshade_dll(&path));
        assert!(validate_bundled_file("dxgi.dll", &path).is_err());
    }

    #[test]
    fn accepts_dll_at_min_size() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("dxgi.dll");
        let mut bytes = vec![0u8; MIN_RESHADE_DLL_BYTES as usize];
        bytes[0] = b'M';
        bytes[1] = b'Z';
        std::fs::write(&path, &bytes).unwrap();
        assert!(is_valid_reshade_dll(&path));
        assert!(is_valid_bundled_file("dxgi.dll", &path));
        assert!(validate_bundled_file("dummy.dll", &path).is_ok());
    }

    #[test]
    fn rejects_missing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("missing.dll");
        assert!(validate_bundled_file("dxgi.dll", &path).is_err());
        assert!(!is_valid_bundled_file("dxgi.dll", &path));
    }

    #[test]
    fn rejects_tiny_vulkan_json() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("ReShade64.json");
        std::fs::write(&path, b"{}").unwrap();
        assert!(!is_valid_bundled_file("ReShade64.json", &path));
        assert!(validate_bundled_file("ReShade64.json", &path).is_err());
    }

    #[test]
    fn accepts_valid_json() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("ReShade64.json");
        std::fs::write(
            &path,
            br#"{"file_format_version": "1.0.0", "layer": {"name": "VK_LAYER_RESHADE", "type": "GLOBAL"}}"#,
        )
        .unwrap();
        assert!(is_valid_bundled_file("ReShade64.json", &path));
    }

    #[test]
    fn rejects_large_dll_without_mz() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("dxgi.dll");
        write_min_dll(&path);
        assert!(!is_valid_reshade_dll(&path));
    }

    #[test]
    fn parses_hashes_json() {
        let parsed = parse_binary_hashes().unwrap();
        assert!(!parsed.files.is_empty());
    }

    #[test]
    fn validates_pinned_hash_for_json() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("ReShade64.json");
        let parsed = parse_binary_hashes().unwrap();
        let mut files = BTreeMap::new();
        files.extend(parsed.files);
        let expected = files.get("ReShade64.json").unwrap().clone();
        let content = std::fs::read_to_string(bundled_file("ReShade64.json")).unwrap();
        std::fs::write(&path, content).unwrap();
        assert_eq!(sha256_file(&path).unwrap(), expected);
    }

}
