use super::overrides::validate_override_payload;
use super::persist::{remove_profile, save_profile};
use super::trust::{
    ensure_known_game_id, is_stale_saved_profile, resolve_trusted_profile,
    validate_profile_paths,
};
use crate::core::models::{GameOverride, GameProfile};

#[test]
fn validate_profile_rejects_missing_install() {
    let profile = GameProfile {
        id: "test".to_string(),
        name: "Test".to_string(),
        source: "manual".to_string(),
        install_dir: r"C:\nonexistent-game-path-xyz".to_string(),
        config_dir: None,
        exe_name: None,
        is_ue: true,
        possible_ue: false,
        cover_url: None,
        custom_cover: None,
        build_id: None,
        engine_family: "ue5".to_string(),
        engine_version: None,
    };
    assert!(validate_profile_paths(&profile).is_err());
}

#[test]
fn resolve_trusted_profile_rejects_unknown_game() {
    let profile = GameProfile {
        id: "test".to_string(),
        name: "Test".to_string(),
        source: "manual".to_string(),
        install_dir: r"C:\nonexistent-game-path-xyz".to_string(),
        config_dir: None,
        exe_name: None,
        is_ue: true,
        possible_ue: false,
        cover_url: None,
        custom_cover: None,
        build_id: None,
        engine_family: "ue5".to_string(),
        engine_version: None,
    };
    assert!(resolve_trusted_profile(&profile).is_err());
}

#[test]
fn ensure_known_game_id_rejects_unknown() {
    assert!(ensure_known_game_id("steam-999999999").is_err());
}

#[test]
fn ensure_known_game_id_rejects_oversized_id() {
    assert!(ensure_known_game_id(&"a".repeat(129)).is_err());
}

#[test]
fn resolve_trusted_profile_rejects_forged_install_dir() {
    let forged_install = tempfile::tempdir().expect("forged install");
    let game_id = format!("ipc-security-{}", uuid::Uuid::new_v4());
    let trusted_install = std::env::current_dir()
        .expect("cwd")
        .join("target")
        .join(format!("test-trusted-{game_id}"));
    std::fs::create_dir_all(&trusted_install).expect("trusted install dir");

    let trusted = GameProfile {
        id: game_id.clone(),
        name: "Trusted".to_string(),
        source: "manual".to_string(),
        install_dir: trusted_install.to_string_lossy().to_string(),
        config_dir: None,
        exe_name: None,
        is_ue: true,
        possible_ue: false,
        cover_url: None,
        custom_cover: None,
        build_id: None,
        engine_family: "ue5".to_string(),
        engine_version: None,
    };
    save_profile(&trusted).expect("save trusted profile");

    let profile = GameProfile {
        id: game_id.clone(),
        name: "Forged".to_string(),
        source: "manual".to_string(),
        install_dir: forged_install.path().to_string_lossy().to_string(),
        ..trusted
    };
    assert!(resolve_trusted_profile(&profile).is_err());
    remove_profile(&game_id).expect("cleanup test profile");
    let _ = std::fs::remove_dir_all(trusted_install);
}

#[test]
fn stale_saved_profile_flags_ipc_security_test_ids() {
    let profile = GameProfile {
        id: "ipc-security-deadbeef".to_string(),
        name: "Trusted".to_string(),
        source: "manual".to_string(),
        install_dir: r"C:\Games\Real".to_string(),
        config_dir: None,
        exe_name: None,
        is_ue: true,
        possible_ue: false,
        cover_url: None,
        custom_cover: None,
        build_id: None,
        engine_family: "ue5".to_string(),
        engine_version: None,
    };
    assert!(is_stale_saved_profile(&profile));
}

#[test]
fn override_rejects_ini_injection_payload() {
    let override_def = GameOverride {
        game_id: "steam-1962700".to_string(),
        name: "bad".to_string(),
        files: std::collections::HashMap::from([(
            "Engine.ini".to_string(),
            std::collections::HashMap::from([(
                "SystemSettings".to_string(),
                std::collections::HashMap::from([(
                    "r.Safe".to_string(),
                    "1\nInjected=True".to_string(),
                )]),
            )]),
        )]),
        removals: std::collections::HashMap::new(),
    };
    assert!(validate_override_payload(&override_def).is_err());
}
