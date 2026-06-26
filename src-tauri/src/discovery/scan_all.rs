use crate::core::models::GameProfile;
use std::collections::HashMap;

use super::dedupe::{dedupe_games, merge_game_profile};
use super::epic::scan_epic_games;
use super::steam::scan_steam_games;

pub fn scan_all_games() -> Vec<GameProfile> {
    let mut games: HashMap<String, GameProfile> = HashMap::new();
    let steam_handle = std::thread::spawn(scan_steam_games);
    let epic_handle = std::thread::spawn(scan_epic_games);

    for game in steam_handle.join().unwrap_or_default() {
        games
            .entry(game.id.clone())
            .and_modify(|existing| merge_game_profile(existing, &game))
            .or_insert(game);
    }
    for game in epic_handle.join().unwrap_or_default() {
        games
            .entry(game.id.clone())
            .and_modify(|existing| merge_game_profile(existing, &game))
            .or_insert(game);
    }

    dedupe_games(games.into_values().collect())
}
