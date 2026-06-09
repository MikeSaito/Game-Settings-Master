use crate::models::{GameOverride, GameProfile, SavedOverrides, SavedProfiles};
use std::fs;
use std::path::PathBuf;

pub fn app_data_dir() -> Result<PathBuf, String> {
    let dir = dirs::data_dir()
        .ok_or_else(|| "Не удалось определить каталог AppData".to_string())?
        .join("UESettingsMaster");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn profiles_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("games.json"))
}

fn overrides_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("overrides.json"))
}

pub fn load_saved_profiles() -> Result<Vec<GameProfile>, String> {
    let path = profiles_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let saved: SavedProfiles = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(saved.games)
}

pub fn save_profile(profile: &GameProfile) -> Result<(), String> {
    let mut games = load_saved_profiles()?;
    if let Some(existing) = games.iter_mut().find(|g| g.id == profile.id) {
        *existing = profile.clone();
    } else {
        games.push(profile.clone());
    }
    let path = profiles_path()?;
    let content =
        serde_json::to_string_pretty(&SavedProfiles { games }).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

pub fn remove_profile(id: &str) -> Result<(), String> {
    let mut games = load_saved_profiles()?;
    games.retain(|g| g.id != id);
    let path = profiles_path()?;
    let content =
        serde_json::to_string_pretty(&SavedProfiles { games }).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

pub fn load_overrides() -> Result<Vec<GameOverride>, String> {
    let path = overrides_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let saved: SavedOverrides = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(saved.overrides)
}

pub fn save_override(override_def: &GameOverride) -> Result<(), String> {
    let mut overrides = load_overrides()?;
    if let Some(existing) = overrides
        .iter_mut()
        .find(|o| o.game_id == override_def.game_id && o.name == override_def.name)
    {
        *existing = override_def.clone();
    } else {
        overrides.push(override_def.clone());
    }
    let path = overrides_path()?;
    let content =
        serde_json::to_string_pretty(&SavedOverrides { overrides }).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

pub fn get_overrides_for_game(game_id: &str) -> Result<Vec<GameOverride>, String> {
    Ok(load_overrides()?
        .into_iter()
        .filter(|o| o.game_id == game_id)
        .collect())
}

pub fn delete_override(game_id: &str, name: &str) -> Result<(), String> {
    let mut overrides = load_overrides()?;
    overrides.retain(|o| !(o.game_id == game_id && o.name == name));
    let path = overrides_path()?;
    let content =
        serde_json::to_string_pretty(&SavedOverrides { overrides }).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}
