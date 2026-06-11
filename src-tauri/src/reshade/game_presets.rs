use super::presets::{list_presets as list_generic, ReShadePresetInfo};
use crate::discovery::{known_app_id_for_game, load_known_games};
use crate::remote_presets::{find_reshade_ini_pack_cached, PackPresetEntry};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct GamePackManifest {
    #[serde(default)]
    pub presets: Vec<GamePackPresetEntry>,
}

#[derive(Debug, Deserialize)]
struct GamePackPresetEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub ini: String,
    #[serde(default = "default_true")]
    pub author: bool,
}

fn default_true() -> bool {
    true
}

pub fn list_presets_for_game(game_id: Option<&str>) -> Vec<ReShadePresetInfo> {
    let mut out = Vec::new();
    if let Some(gid) = game_id {
        out.extend(load_author_presets(gid));
    }
    for generic in list_generic() {
        if !out.iter().any(|p| p.id == generic.id) {
            out.push(generic);
        }
    }
    out
}

pub fn load_author_presets(game_id: &str) -> Vec<ReShadePresetInfo> {
    let mut out = load_local_author_presets(game_id);
    for remote in load_remote_author_presets(game_id) {
        if !out.iter().any(|p| p.id == remote.id) {
            out.push(remote);
        }
    }
    out
}

fn load_local_author_presets(game_id: &str) -> Vec<ReShadePresetInfo> {
    let Some(dir) = game_pack_dir(game_id) else {
        return Vec::new();
    };
    let manifest_path = dir.join("manifest.json");
    if !manifest_path.is_file() {
        return scan_ini_presets(&dir);
    }
    let Ok(raw) = fs::read_to_string(&manifest_path) else {
        return scan_ini_presets(&dir);
    };
    let Ok(manifest) = serde_json::from_str::<GamePackManifest>(&raw) else {
        return scan_ini_presets(&dir);
    };
    manifest
        .presets
        .into_iter()
        .map(|p| ReShadePresetInfo {
            id: p.id,
            name: p.name,
            description: p.description,
            author: p.author,
        })
        .collect()
}

pub fn load_remote_author_presets(game_id: &str) -> Vec<ReShadePresetInfo> {
    let engine_family = engine_family_for_known_game(game_id);
    let Some(pack) = find_reshade_ini_pack_cached(Some(game_id), engine_family.as_deref()) else {
        return Vec::new();
    };
    pack.manifest
        .presets
        .iter()
        .filter_map(|entry| {
            let PackPresetEntry::ReShade(p) = entry else {
                return None;
            };
            Some(ReShadePresetInfo {
                id: p.id.clone(),
                name: p.name.clone(),
                description: p.description.clone(),
                author: true,
            })
        })
        .collect()
}

fn scan_ini_presets(dir: &Path) -> Vec<ReShadePresetInfo> {
    let mut out = Vec::new();
    let entries = fs::read_dir(dir).ok();
    for entry in entries.into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("ini") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if !crate::fs_util::is_safe_pack_id(id) {
            continue;
        }
        let id = id.to_string();
        out.push(ReShadePresetInfo {
            id: id.clone(),
            name: id.replace('-', " "),
            description: "Авторский пресет для игры.".to_string(),
            author: true,
        });
    }
    out
}

pub fn game_pack_dir(game_id: &str) -> Option<PathBuf> {
    let app_id = known_app_id_for_game(game_id)?;
    let pack_id = load_known_games()
        .get(&app_id)
        .and_then(|e| e.reshade_pack.clone())?;
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("presets")
        .join("games")
        .join(pack_id);
    if dir.is_dir() { Some(dir) } else { None }
}

fn engine_family_for_known_game(game_id: &str) -> Option<String> {
    known_app_id_for_game(game_id).and_then(|app_id| {
        load_known_games()
            .get(&app_id)
            .and_then(|e| e.engine_family.clone())
    })
}

pub fn preset_ini_path_for(preset_id: &str, game_id: Option<&str>) -> Result<PathBuf, String> {
    if let Some(gid) = game_id {
        if let Some(dir) = game_pack_dir(gid) {
            if let Some(path) = resolve_pack_ini(&dir, preset_id) {
                return Ok(path);
            }
        }
        if let Some(pack) = find_reshade_ini_pack_cached(
            Some(gid),
            engine_family_for_known_game(gid).as_deref(),
        ) {
            if let Some(path) = pack.load_reshade_ini_path(preset_id) {
                return Ok(path);
            }
        }
    }
    super::presets::preset_ini_path(preset_id)
}

pub fn preset_exists_for(preset_id: &str, game_id: Option<&str>) -> bool {
    if is_author_preset(preset_id, game_id) {
        return true;
    }
    super::presets::preset_exists(preset_id)
}

fn resolve_pack_ini(pack_dir: &Path, preset_id: &str) -> Option<PathBuf> {
    if !crate::fs_util::is_safe_pack_id(preset_id) {
        return None;
    }
    let manifest_path = pack_dir.join("manifest.json");
    if manifest_path.is_file() {
        let raw = fs::read_to_string(&manifest_path).ok()?;
        let manifest: GamePackManifest = serde_json::from_str(&raw).ok()?;
        if let Some(entry) = manifest.presets.iter().find(|p| p.id == preset_id) {
            if let Some(path) = crate::fs_util::resolve_file_within_root(pack_dir, &entry.ini) {
                return Some(path);
            }
        }
    }
    crate::fs_util::resolve_file_within_root(pack_dir, &format!("{preset_id}.ini"))
}

pub fn read_preset_ini_for(preset_id: &str, game_id: Option<&str>) -> Result<String, String> {
    if !crate::fs_util::is_safe_pack_id(preset_id) {
        return Err(format!("Недопустимый идентификатор пресета: {preset_id}"));
    }
    if !preset_exists_for(preset_id, game_id) {
        return Err(format!("Неизвестный пресет ReShade: {preset_id}"));
    }
    if let Some(gid) = game_id {
        if let Some(dir) = game_pack_dir(gid) {
            if let Some(path) = resolve_pack_ini(&dir, preset_id) {
                return fs::read_to_string(&path)
                    .map_err(|e| format!("Не удалось прочитать {}: {e}", path.display()));
            }
        }
        if let Some(pack) = find_reshade_ini_pack_cached(
            Some(gid),
            engine_family_for_known_game(gid).as_deref(),
        ) {
            if let Some(result) = pack.load_reshade_ini_preset(preset_id) {
                return result;
            }
        }
    }
    let path = preset_ini_path_for(preset_id, game_id)?;
    fs::read_to_string(&path).map_err(|e| format!("Не удалось прочитать {}: {e}", path.display()))
}

pub fn suggested_reshade_api_for_game(game_id: &str, engine_family: Option<&str>) -> Option<String> {
    if let Some(app_id) = known_app_id_for_game(game_id) {
        if let Some(api) = load_known_games()
            .get(&app_id)
            .and_then(|e| e.suggested_reshade_api.clone())
        {
            return Some(api);
        }
    }
    match engine_family {
        Some("ue5") => Some("dx12".to_string()),
        Some("ue4") => Some("dx11".to_string()),
        Some("unity") => Some("dx11".to_string()),
        _ => None,
    }
}

pub fn is_author_preset(preset_id: &str, game_id: Option<&str>) -> bool {
    let Some(gid) = game_id else {
        return false;
    };
    load_author_presets(gid).iter().any(|p| p.id == preset_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::remote_presets::{
        PackApply, PackManifest, PackMatch, PackPresetEntry, ReShadeIniPresetEntry, ResolvedPack,
    };

    #[test]
    fn read_preset_ini_rejects_traversal_id() {
        let err = read_preset_ini_for("../secret", Some("steam-1962700")).unwrap_err();
        assert!(err.contains("Недопустимый"));
    }

    #[test]
    fn sn2_pack_lists_author_preset_for_epic() {
        let presets = load_author_presets("epic-Subnautica2");
        assert!(presets.iter().any(|p| p.id == "sn2-underwater-clarity"));
    }

    #[test]
    fn sn2_pack_lists_author_preset() {
        let presets = load_author_presets("steam-1962700");
        assert!(presets.iter().any(|p| p.id == "sn2-underwater-clarity"));
    }

    #[test]
    fn list_for_game_includes_generic() {
        let all = list_presets_for_game(Some("steam-1962700"));
        assert!(all.iter().any(|p| p.id == "performance"));
        assert!(all.iter().any(|p| p.author));
    }

    #[test]
    fn merge_remote_presets_skips_local_duplicates() {
        let local = vec![ReShadePresetInfo {
            id: "sn2-underwater-clarity".to_string(),
            name: "Local".to_string(),
            description: "local".to_string(),
            author: true,
        }];
        let remote = vec![ReShadePresetInfo {
            id: "sn2-underwater-clarity".to_string(),
            name: "Remote".to_string(),
            description: "remote".to_string(),
            author: true,
        }];
        let merged = merge_author_presets(local, remote);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].name, "Local");
    }

    #[test]
    fn remote_reshade_ini_path_resolves_from_pack() {
        let dir = tempfile::TempDir::new().unwrap();
        let presets = dir.path().join("presets");
        std::fs::create_dir_all(&presets).unwrap();
        std::fs::write(presets.join("sn2-underwater-clarity.ini"), "Techniques=\n").unwrap();

        let pack = ResolvedPack {
            manifest: PackManifest {
                schema_version: 1,
                pack_id: "subnautica2-reshade".to_string(),
                title: None,
                revision: None,
                updated_at: None,
                match_rules: PackMatch {
                    steam_app_ids: vec!["1962700".into()],
                    game_ids: vec!["steam-1962700".into()],
                    engine_families: vec!["ue5".into()],
                    overlay_ids: vec![],
                },
                apply: PackApply::ReShadeIni {
                    presets_root: "presets".to_string(),
                },
                bundle: None,
                presets: vec![PackPresetEntry::ReShade(ReShadeIniPresetEntry {
                    id: "sn2-underwater-clarity".to_string(),
                    name: "Underwater Clarity".to_string(),
                    description: String::new(),
                    ini_file: "sn2-underwater-clarity.ini".to_string(),
                })],
                policy: None,
            },
            root: dir.path().to_path_buf(),
        };

        let path = pack
            .load_reshade_ini_path("sn2-underwater-clarity")
            .expect("path");
        assert!(path.is_file());
    }

    fn merge_author_presets(
        local: Vec<ReShadePresetInfo>,
        remote: Vec<ReShadePresetInfo>,
    ) -> Vec<ReShadePresetInfo> {
        let mut out = local;
        for remote in remote {
            if !out.iter().any(|p| p.id == remote.id) {
                out.push(remote);
            }
        }
        out
    }
}
