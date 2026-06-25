use super::{ue_path_has_saved_segment, validate_config_dir};
use std::fs;
use tempfile::TempDir;

#[test]
fn saved_segment_requires_path_component_not_substring() {
    let dir = TempDir::new().unwrap();
    let fake = dir.path().join("SavedGames");
    fs::create_dir_all(&fake).unwrap();
    assert!(!ue_path_has_saved_segment(&fake));

    let real = dir.path().join("Saved").join("Config").join("Windows");
    fs::create_dir_all(&real).unwrap();
    assert!(ue_path_has_saved_segment(&real));
}

#[test]
fn validate_rejects_non_ue_without_gus_or_saved() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("RandomConfig");
    fs::create_dir_all(&path).unwrap();
    assert!(validate_config_dir(path.to_str().unwrap()).is_err());
}
