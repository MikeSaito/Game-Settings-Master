use crate::core::models::GameProfile;
use std::collections::HashMap;

use super::dedupe::{dedupe_games, merge_game_profile};
use super::epic::scan_epic_games;
use super::steam::scan_steam_games;

pub fn scan_all_games() -> Result<Vec<GameProfile>, String> {
    super::other_stores::clear_warnings();
    let mut games: HashMap<String, GameProfile> = HashMap::new();
    let steam_handle = std::thread::spawn(scan_steam_games);
    let epic_handle = std::thread::spawn(scan_epic_games);
    let gog_handle = std::thread::spawn(super::other_stores::scan_gog);
    let xbox_handle = std::thread::spawn(super::other_stores::scan_xbox);

    let steam_games = join_scan("Steam", steam_handle);
    let epic_games = join_scan("Epic Games", epic_handle);
    let mut failures = Vec::new();
    let steam_games = steam_games.unwrap_or_else(|error| {
        failures.push(error);
        Vec::new()
    });
    let epic_games = epic_games.unwrap_or_else(|error| {
        failures.push(error);
        Vec::new()
    });
    let extra = [join_scan("GOG", gog_handle), join_scan("Xbox", xbox_handle)]
        .into_iter()
        .flat_map(|result| {
            result.unwrap_or_else(|error| {
                failures.push(error);
                Vec::new()
            })
        })
        .collect::<Vec<_>>();
    for failure in failures {
        super::other_stores::warn(failure);
    }

    for game in steam_games {
        games
            .entry(game.id.clone())
            .and_modify(|existing| merge_game_profile(existing, &game))
            .or_insert(game);
    }
    for game in epic_games.into_iter().chain(extra) {
        games
            .entry(game.id.clone())
            .and_modify(|existing| merge_game_profile(existing, &game))
            .or_insert(game);
    }

    Ok(dedupe_games(games.into_values().collect()))
}

fn join_scan(
    source: &str,
    handle: std::thread::JoinHandle<Vec<GameProfile>>,
) -> Result<Vec<GameProfile>, String> {
    match handle.join() {
        Ok(games) => Ok(games),
        Err(payload) => {
            let detail = payload
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                .unwrap_or("unknown panic");
            Err(format!("{source} discovery failed: {detail}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::join_scan;

    #[test]
    fn scanner_panics_are_reported() {
        let handle = std::thread::spawn(|| -> Vec<crate::core::models::GameProfile> {
            panic!("broken scanner")
        });
        let error = join_scan("Test", handle).unwrap_err();
        assert!(error.contains("Test discovery failed"));
        assert!(error.contains("broken scanner"));
    }
}
