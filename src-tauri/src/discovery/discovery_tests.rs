use super::*;
use crate::core::models::GameProfile;

fn profile(id: &str, source: &str, install_dir: &str) -> GameProfile {
    GameProfile {
        id: id.to_string(),
        name: id.to_string(),
        source: source.to_string(),
        install_dir: install_dir.to_string(),
        config_dir: None,
        exe_name: None,
        is_ue: true,
        possible_ue: false,
        cover_url: if source == "steam" {
            Some("https://example.com/cover.jpg".to_string())
        } else {
            None
        },
        custom_cover: None,
        build_id: None,
        engine_family: "unknown".to_string(),
        engine_version: None,
    }
}

#[test]
fn manual_profile_rejects_empty_name() {
    assert!(profile_from_manual_path("   ", r"C:\Games\Any").is_err());
}

#[test]
fn dedupe_prefers_steam_over_manual_same_install() {
    let install = r"C:\Games\Subnautica2";
    let games = dedupe_games(vec![
        profile("manual-1", "manual", install),
        profile("steam-1962700", "steam", install),
    ]);
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].id, "steam-1962700");
    assert!(games[0].cover_url.is_some());
}

#[test]
fn dedupe_merges_same_steam_app_id() {
    let games = dedupe_games(vec![
        profile("steam-123", "steam", r"D:\Steam\common\Game"),
        profile("steam-123", "steam", r"D:\Steam\common\Game"),
    ]);
    assert_eq!(games.len(), 1);
}

#[test]
fn normalize_install_dir_is_case_insensitive() {
    assert_eq!(
        normalize_install_dir(r"C:\Games\Test"),
        normalize_install_dir(r"c:\games\test")
    );
}
