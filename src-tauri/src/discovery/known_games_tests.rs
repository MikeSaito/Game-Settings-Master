use super::{known_app_id_for_game, known_config_dir, use_test_local_app_data_dir};
use tempfile::TempDir;

#[test]
fn epic_subnautica_resolves_to_steam_app_id() {
    assert_eq!(
        known_app_id_for_game("epic-Subnautica2").as_deref(),
        Some("1962700")
    );
    assert_eq!(
        known_app_id_for_game("steam-1962700").as_deref(),
        Some("1962700")
    );
}

#[test]
fn epic_subnautica_not_confused_with_subnautica2() {
    assert_ne!(
        known_app_id_for_game("epic-Subnautica").as_deref(),
        Some("1962700")
    );
}

#[test]
fn palworld_known_dir_resolves_pal_folder() {
    let temp = TempDir::new().unwrap();
    let _local_app_data = use_test_local_app_data_dir(temp.path());
    let platform = temp
        .path()
        .join("Pal")
        .join("Saved")
        .join("Config")
        .join("Windows");
    std::fs::create_dir_all(&platform).unwrap();
    std::fs::write(
        platform.join("GameUserSettings.ini"),
        "[ScalabilityGroups]\n",
    )
    .unwrap();

    let resolved = known_config_dir("1623730").expect("Palworld config path");
    assert!(resolved.ends_with("Pal\\Saved\\Config\\Windows"));
}

#[test]
fn pubg_known_dir_without_gus() {
    let temp = TempDir::new().unwrap();
    let _local_app_data = use_test_local_app_data_dir(temp.path());
    let platform = temp
        .path()
        .join("TslGame")
        .join("Saved")
        .join("Config")
        .join("WindowsNoEditor");
    std::fs::create_dir_all(&platform).unwrap();

    let resolved = known_config_dir("578080").expect("PUBG config path");
    assert!(resolved.ends_with("WindowsNoEditor"));
}
