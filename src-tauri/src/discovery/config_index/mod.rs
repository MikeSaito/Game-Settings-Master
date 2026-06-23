mod matcher;
mod scan;
mod types;

pub use matcher::{build_match_candidates, match_config_from_index, normalize_key};
pub use scan::scan_local_appdata_configs;
pub use types::ConfigIndexEntry;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn normalize_removes_spaces() {
        assert_eq!(normalize_key("Subnautica 2"), "subnautica2");
        assert_eq!(normalize_key("Subnautica2"), "subnautica2");
    }

    #[test]
    fn no_substring_false_match() {
        let index = vec![
            ConfigIndexEntry {
                folder_name: "Subnautica2".to_string(),
                config_dir: PathBuf::from("C:\\Subnautica2"),
            },
            ConfigIndexEntry {
                folder_name: "ASTRONEER".to_string(),
                config_dir: PathBuf::from("C:\\ASTRONEER"),
            },
        ];
        assert_eq!(
            match_config_from_index(&index, &["ASTRONEER".to_string()]),
            Some(PathBuf::from("C:\\ASTRONEER"))
        );
        assert_eq!(
            match_config_from_index(&index, &["Subnautica 2".to_string()]),
            Some(PathBuf::from("C:\\Subnautica2"))
        );
    }
}
