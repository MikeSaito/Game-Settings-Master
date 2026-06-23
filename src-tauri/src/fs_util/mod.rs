mod config;
mod io;
mod path_safety;
mod process;

pub use config::{clear_config_readonly, ensure_config_writable};
pub use io::{
    format_io_error, read_file_bytes, read_utf8_text_file, strip_utf8_bom, write_file_bytes,
    write_file_bytes_opts, clear_readonly,
};
pub use path_safety::{
    ensure_safe_child_file, is_allowed_config_ini_filename, is_allowed_restore_filename,
    is_safe_backup_id, is_safe_exe_basename, is_safe_ini_key_name, is_safe_ini_section_name,
    is_safe_ini_value, is_safe_manifest_relative_path, is_safe_pack_ini_filename,
    normalize_ini_section_name, path_within_root, safe_child_path, ALLOWED_CONFIG_INI_FILES,
};
pub use process::{is_exe_running, is_exe_running_uncached, kill_exe};

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
    fn atomic_write_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("atomic.ini");
        write_file_bytes_opts(&path, b"v1", true).unwrap();
        write_file_bytes_opts(&path, b"v2", true).unwrap();
        assert_eq!(read_file_bytes(&path).unwrap(), b"v2");
    }

    #[test]
    fn rejects_traversal_in_pack_ini_path() {
        let secret = TempDir::new().unwrap();
        std::fs::write(secret.path().join("secret.ini"), "secret").unwrap();

        let rel = format!(
            "..{}..{}secret.ini",
            std::path::MAIN_SEPARATOR,
            std::path::MAIN_SEPARATOR
        );
        assert!(!is_safe_pack_ini_filename(&rel));
    }

    #[test]
    fn allowed_config_ini_whitelist() {
        assert!(is_allowed_config_ini_filename("Engine.ini"));
        assert!(!is_allowed_config_ini_filename("../Engine.ini"));
        assert!(!is_allowed_config_ini_filename("evil.ini"));
    }

    #[test]
    fn ini_payload_validation_rejects_injection_fragments() {
        assert!(is_safe_ini_section_name("[SystemSettings]"));
        assert!(is_safe_ini_key_name("r.ViewDistanceScale"));
        assert!(is_safe_ini_value("(A=1,B=2)"));
        assert!(!is_safe_ini_section_name("System]\n[Injected"));
        assert!(!is_safe_ini_key_name("r.Safe=evil"));
        assert!(!is_safe_ini_value("1\nInjected=True"));
    }

    #[test]
    fn safe_backup_id_rejects_traversal() {
        assert!(is_safe_backup_id("20250611_120000"));
        assert!(!is_safe_backup_id("../evil"));
        assert!(!is_safe_backup_id(""));
    }

    #[test]
    fn allowed_restore_filename_covers_ue_files() {
        assert!(is_allowed_restore_filename("DeviceProfiles.ini"));
        assert!(is_allowed_restore_filename("UserConfigSelections"));
        assert!(is_allowed_restore_filename("prefs"));
        assert!(is_allowed_restore_filename("settings.json"));
        assert!(!is_allowed_restore_filename("../evil.ini"));
        assert!(!is_allowed_restore_filename("evil.ini"));
    }

    #[test]
    fn strip_utf8_bom_allows_json_parse() {
        let with_bom = b"\xEF\xBB\xBF{\"games\":[]}";
        let text = std::str::from_utf8(strip_utf8_bom(with_bom)).unwrap();
        let _: serde_json::Value = serde_json::from_str(text).unwrap();
    }

    #[test]
    fn safe_exe_basename_rejects_paths() {
        assert!(is_safe_exe_basename("Game.exe"));
        assert!(is_safe_exe_basename("Game"));
        assert!(is_safe_exe_basename("The Stanley Parable Ultra Deluxe.exe"));
        assert!(!is_safe_exe_basename(r"C:\Game.exe"));
        assert!(!is_safe_exe_basename("../evil.exe"));
        assert!(!is_safe_exe_basename("bad|name.exe"));
    }

    #[test]
    fn clears_readonly_before_write() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("ro.ini");
        std::fs::File::create(&path)
            .unwrap()
            .write_all(b"old")
            .unwrap();
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&path, perms).unwrap();

        write_file_bytes(&path, b"new").unwrap();
        assert_eq!(read_file_bytes(&path).unwrap(), b"new");
    }
}
