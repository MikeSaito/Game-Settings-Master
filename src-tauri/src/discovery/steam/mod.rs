mod manifest;
mod paths;
mod signal;

use crate::discovery::dedupe_paths;
use crate::discovery::known_games::load_known_games;
use crate::core::models::GameProfile;
use std::collections::HashMap;
use std::fs;

pub use signal::collect_steam_library_mtimes;

use manifest::parse_steam_manifest;
use paths::{find_steam_install_paths, parse_library_folders};

pub fn scan_steam_games() -> Vec<GameProfile> {
    let mut games: HashMap<String, GameProfile> = HashMap::new();
    let known = load_known_games();
    let steam_paths = dedupe_paths(find_steam_install_paths());

    for steam_root in steam_paths {
        let library_folders = dedupe_paths(parse_library_folders(&steam_root));
        for library in library_folders {
            let steamapps = library.join("steamapps");
            if !steamapps.exists() {
                continue;
            }
            let Ok(entries) = fs::read_dir(&steamapps) else {
                continue;
            };
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.starts_with("appmanifest_") || !name.ends_with(".acf") {
                    continue;
                }
                if let Some(game) = parse_steam_manifest(&entry.path(), &library, &known) {
                    games
                        .entry(game.id.clone())
                        .and_modify(|existing| {
                            if existing.config_dir.is_none() {
                                existing.config_dir = game.config_dir.clone();
                            }
                            if existing.exe_name.is_none() {
                                existing.exe_name = game.exe_name.clone();
                            }
                        })
                        .or_insert(game);
                }
            }
        }
    }

    games.into_values().collect()
}
