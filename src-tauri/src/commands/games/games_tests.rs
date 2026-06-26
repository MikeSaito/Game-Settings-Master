use super::*;

#[test]
fn remove_game_profile_rejects_empty_id() {
    assert!(remove_game_profile("   ".to_string()).is_err());
}

#[test]
fn remove_game_profile_rejects_unknown_id() {
    assert!(remove_game_profile("steam-999999999".to_string()).is_err());
}

#[test]
fn set_game_config_dir_requires_known_game() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = dir.path().join("Saved").join("Config").join("Windows");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(config.join("GameUserSettings.ini"), b"[x]").unwrap();
    let path = config.to_string_lossy().to_string();
    assert!(set_game_config_dir("steam-999999999".to_string(), path).is_err());
}
