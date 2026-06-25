mod manifest;
mod paths;
mod signal;

use crate::core::models::GameProfile;
use crate::discovery::merge_game_profile;
use std::collections::HashMap;
use std::fs;

pub use signal::epic_manifests_signal_mtime;

use manifest::parse_epic_manifest;
use paths::epic_manifest_dirs;

pub fn scan_epic_games() -> Vec<GameProfile> {
    let mut games: HashMap<String, GameProfile> = HashMap::new();
    let manifest_dirs = epic_manifest_dirs();

    for dir in manifest_dirs {
        if !dir.exists() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("item") {
                continue;
            }
            if let Some(game) = parse_epic_manifest(&path) {
                games
                    .entry(game.id.clone())
                    .and_modify(|existing| merge_game_profile(existing, &game))
                    .or_insert(game);
            }
        }
    }

    games.into_values().collect()
}

#[cfg(test)]
#[path = "epic_tests.rs"]
mod tests;
