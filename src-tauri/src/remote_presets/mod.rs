mod config;
mod manifest;
mod sync;

pub use config::{cache_root, effective_base_url, load_config, set_base_url, PresetServerConfig};
pub use manifest::{PackPolicy, ResolvedPack};
pub use sync::{load_cached_catalog, load_cached_pack, sync_now, sync_pack_by_id, SyncReport};

use crate::models::{PresetDefinition, PresetInfo};
use manifest::PackApply;
use std::sync::Mutex;
use std::time::{Duration, Instant};

static LAST_SYNC_ATTEMPT: Mutex<Option<Instant>> = Mutex::new(None);
const SYNC_COOLDOWN: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, serde::Serialize)]
pub struct RemotePresetStatus {
    pub configured: bool,
    pub base_url: Option<String>,
    pub auto_sync: bool,
    pub last_sync_at: Option<String>,
    pub last_sync_ok: bool,
    pub last_sync_error: Option<String>,
    pub catalog_version: Option<String>,
    pub cached_packs: Vec<String>,
}

pub fn get_status() -> RemotePresetStatus {
    let cfg = load_config().unwrap_or_default();
    let base_url = effective_base_url();
    let cached_packs = list_cached_pack_ids();

    RemotePresetStatus {
        configured: base_url.is_some(),
        base_url,
        auto_sync: cfg.auto_sync,
        last_sync_at: cfg.last_sync_at,
        last_sync_ok: cfg.last_sync_ok,
        last_sync_error: cfg.last_sync_error,
        catalog_version: cfg.catalog_version,
        cached_packs,
    }
}

fn list_cached_pack_ids() -> Vec<String> {
    let Ok(root) = cache_root() else {
        return Vec::new();
    };
    let packs_dir = root.join("packs");
    let Ok(entries) = std::fs::read_dir(packs_dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().join("extracted").is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect()
}

pub fn ensure_synced() {
    if crate::process_util::is_app_background() {
        return;
    }
    if effective_base_url().is_none() {
        return;
    }
    let cfg = match load_config() {
        Ok(c) => c,
        Err(_) => return,
    };
    if !cfg.auto_sync {
        return;
    }

    let should_try = {
        let Ok(mut guard) = LAST_SYNC_ATTEMPT.lock() else {
            return;
        };
        let now = Instant::now();
        match guard.as_ref() {
            Some(last) if now.duration_since(*last) < SYNC_COOLDOWN => false,
            _ => {
                *guard = Some(now);
                true
            }
        }
    };

    if !should_try {
        return;
    }

    if let Err(e) = sync_now(false) {
        let _ = sync::mark_sync_error(&e);
    }
}

fn all_resolved_packs() -> Vec<ResolvedPack> {
    let catalog = match load_cached_catalog() {
        Some(c) => c,
        None => return Vec::new(),
    };

    catalog
        .packs
        .iter()
        .filter_map(|p| {
            load_cached_pack(&p.id).map(|(manifest, root)| ResolvedPack { manifest, root })
        })
        .collect()
}

/// Только локальный кэш — без сетевой синхронизации (для списка пресетов в UI).
pub fn find_pack_cached(
    game_id: Option<&str>,
    engine_family: Option<&str>,
    overlay_id: Option<&str>,
) -> Option<ResolvedPack> {
    all_resolved_packs()
        .into_iter()
        .find(|pack| pack.matches(game_id, engine_family, overlay_id))
}

pub fn find_pack(
    game_id: Option<&str>,
    engine_family: Option<&str>,
    overlay_id: Option<&str>,
) -> Option<ResolvedPack> {
    ensure_synced();
    find_pack_cached(game_id, engine_family, overlay_id)
}

pub fn find_packs<F>(predicate: F) -> Vec<ResolvedPack>
where
    F: Fn(&ResolvedPack) -> bool,
{
    ensure_synced();
    all_resolved_packs().into_iter().filter(predicate).collect()
}

pub fn find_ue_json_pack(engine_family: Option<&str>) -> Option<ResolvedPack> {
    find_packs(|pack| {
        matches!(pack.manifest.apply, PackApply::UeJson { .. })
            && pack.matches(None, engine_family, None)
    })
    .into_iter()
    .next()
}

pub fn find_ue_json_pack_cached(engine_family: Option<&str>) -> Option<ResolvedPack> {
    all_resolved_packs().into_iter().find(|pack| {
        matches!(pack.manifest.apply, PackApply::UeJson { .. })
            && pack.matches(None, engine_family, None)
    })
}

pub fn find_unity_pack() -> Option<ResolvedPack> {
    find_packs(|pack| matches!(pack.manifest.apply, PackApply::Unity { .. }))
        .into_iter()
        .next()
}

pub fn find_unity_pack_cached() -> Option<ResolvedPack> {
    all_resolved_packs()
        .into_iter()
        .find(|pack| matches!(pack.manifest.apply, PackApply::Unity { .. }))
}

pub fn find_catalog_packs() -> Vec<ResolvedPack> {
    find_packs(|pack| matches!(pack.manifest.apply, PackApply::Catalog { .. }))
}

pub fn forza_presets_from_cache(game_id: Option<&str>) -> Option<Vec<PresetInfo>> {
    let pack = find_pack_cached(game_id, Some("forza"), None)?;
    if !matches!(pack.manifest.apply, PackApply::Forza { .. }) {
        return None;
    }
    if pack.manifest.presets.is_empty() {
        return None;
    }
    Some(pack.manifest.presets_info())
}

pub fn forza_pack_ready(game_id: Option<&str>) -> bool {
    find_pack_cached(game_id, Some("forza"), None)
        .is_some_and(|pack| pack.forza_pack_has_profiles())
}

pub fn sync_forza_pack_if_needed(force: bool) -> Result<(), String> {
    if forza_presets_from_cache(None).is_some_and(|p| !p.is_empty()) {
        return Ok(());
    }
    if effective_base_url().is_none() {
        return Err("Не удалось загрузить пресеты Forza.".into());
    }
    sync_pack_by_id("forza-fh6", force)?;
    Ok(())
}

pub fn find_forza_pack(game_id: Option<&str>) -> Option<ResolvedPack> {
    find_pack(game_id, Some("forza"), None)
}

pub fn load_ue_overlay(overlay_id: &str) -> Option<Result<PresetDefinition, String>> {
    let pack = find_pack(None, None, Some(overlay_id))?;
    pack.load_ue_overlay()
}

#[cfg(test)]
mod tests {
    use super::manifest::{pack_matches, PackMatch};

    #[test]
    fn match_forza_by_engine() {
        let rules = PackMatch {
            steam_app_ids: vec!["2483190".into()],
            game_ids: vec!["steam-2483190".into()],
            engine_families: vec!["forza".into()],
            overlay_ids: vec![],
        };
        assert!(pack_matches(
            &rules,
            Some("steam-2483190"),
            Some("forza"),
            None
        ));
        assert!(pack_matches(&rules, None, Some("forza"), None));
    }

    #[test]
    fn match_overlay_by_id() {
        let rules = PackMatch {
            steam_app_ids: vec![],
            game_ids: vec![],
            engine_families: vec![],
            overlay_ids: vec!["subnautica2".into()],
        };
        assert!(pack_matches(&rules, None, None, Some("subnautica2")));
        assert!(!pack_matches(&rules, None, None, Some("other")));
    }
}
