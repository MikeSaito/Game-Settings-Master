use std::fs;

use crate::core::models::{GameProfile, SavedOverrides, SavedProfiles};

use super::overrides::load_overrides;
use super::storage::{overrides_path, profiles_path, write_json_atomic};
use super::trust::{is_stale_saved_profile, validate_profile_paths};

const MAX_PROFILES_JSON_BYTES: usize = 2 * 1024 * 1024;
const MAX_SAVED_GAMES: usize = 512;

pub fn load_saved_profiles() -> Result<Vec<GameProfile>, String> {
    let path = profiles_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
    if meta.len() as usize > MAX_PROFILES_JSON_BYTES {
        return Err(crate::i18n::t(
            &format!(
                "games.json слишком большой ({} KB, лимит {} KB)",
                meta.len() / 1024,
                MAX_PROFILES_JSON_BYTES / 1024
            ),
            &format!(
                "games.json is too large ({} KB, limit {} KB)",
                meta.len() / 1024,
                MAX_PROFILES_JSON_BYTES / 1024
            ),
        ));
    }
    let (content, had_bom) = crate::fs_util::read_utf8_text_file(&path)?;
    let saved: SavedProfiles = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    if saved.games.len() > MAX_SAVED_GAMES {
        return Err(crate::i18n::t(
            &format!(
                "Слишком много сохранённых игр ({} > {MAX_SAVED_GAMES})",
                saved.games.len()
            ),
            &format!(
                "Too many saved games ({} > {MAX_SAVED_GAMES})",
                saved.games.len()
            ),
        ));
    }
    if had_bom {
        write_json_atomic(&path, &content)?;
    }
    Ok(saved.games)
}

/// Removes test/broken entries from games.json (temp install, ipc-security-*).
pub fn prune_stale_saved_profiles() -> Result<usize, String> {
    let mut games = load_saved_profiles()?;
    let before = games.len();
    games.retain(|g| !is_stale_saved_profile(g));
    let removed = before.saturating_sub(games.len());
    if removed == 0 {
        return Ok(0);
    }
    let path = profiles_path()?;
    let content =
        serde_json::to_string_pretty(&SavedProfiles { games }).map_err(|e| e.to_string())?;
    write_json_atomic(&path, &content)?;
    Ok(removed)
}

pub fn save_profile(profile: &GameProfile) -> Result<(), String> {
    validate_profile_paths(profile)?;
    let mut games = load_saved_profiles()?;
    if let Some(existing) = games.iter_mut().find(|g| g.id == profile.id) {
        *existing = profile.clone();
    } else {
        games.push(profile.clone());
    }
    let path = profiles_path()?;
    let content =
        serde_json::to_string_pretty(&SavedProfiles { games }).map_err(|e| e.to_string())?;
    write_json_atomic(&path, &content)
}

pub fn remove_profile(id: &str) -> Result<(), String> {
    let mut games = load_saved_profiles()?;
    games.retain(|g| g.id != id);
    let path = profiles_path()?;
    let content =
        serde_json::to_string_pretty(&SavedProfiles { games }).map_err(|e| e.to_string())?;
    write_json_atomic(&path, &content)?;
    let mut overrides = load_overrides()?;
    let before = overrides.len();
    overrides.retain(|o| o.game_id != id);
    if overrides.len() != before {
        let opath = overrides_path()?;
        let ocontent = serde_json::to_string_pretty(&SavedOverrides { overrides })
            .map_err(|e| e.to_string())?;
        write_json_atomic(&opath, &ocontent)?;
    }
    Ok(())
}
