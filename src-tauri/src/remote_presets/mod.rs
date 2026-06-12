mod config;
mod manifest;
mod sync;

pub use config::{cache_root, effective_base_url, load_config, set_base_url, PresetServerConfig};
pub use manifest::{PackApply, PackPolicy, PackPresetEntry, ResolvedPack};

#[cfg(test)]
pub use manifest::{PackManifest, PackMatch, ReShadeIniPresetEntry};
pub use sync::{load_cached_catalog, load_cached_pack, sync_now, sync_pack_by_id, SyncReport};

use crate::models::PresetInfo;
use std::sync::Mutex;
use std::time::{Duration, Instant};

static LAST_SYNC_ATTEMPT: Mutex<Option<Instant>> = Mutex::new(None);
const SYNC_COOLDOWN: Duration = Duration::from_secs(60);

struct ResolvedPacksCache {
    catalog_version: String,
    packs: Vec<ResolvedPack>,
}

static RESOLVED_PACKS_CACHE: Mutex<Option<ResolvedPacksCache>> = Mutex::new(None);

pub fn invalidate_resolved_packs_cache() {
    if let Ok(mut guard) = RESOLVED_PACKS_CACHE.lock() {
        *guard = None;
    }
}

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
        None => {
            invalidate_resolved_packs_cache();
            return Vec::new();
        }
    };

    if let Ok(mut guard) = RESOLVED_PACKS_CACHE.lock() {
        if let Some(cached) = guard.as_ref() {
            if cached.catalog_version == catalog.version {
                return cached.packs.clone();
            }
        }

        let packs: Vec<ResolvedPack> = catalog
            .packs
            .iter()
            .filter_map(|p| {
                load_cached_pack(&p.id).map(|(manifest, root)| ResolvedPack { manifest, root })
            })
            .collect();
        *guard = Some(ResolvedPacksCache {
            catalog_version: catalog.version.clone(),
            packs: packs.clone(),
        });
        return packs;
    }

    catalog
        .packs
        .iter()
        .filter_map(|p| {
            load_cached_pack(&p.id).map(|(manifest, root)| ResolvedPack { manifest, root })
        })
        .collect()
}

fn reshade_ini_packs() -> Vec<ResolvedPack> {
    all_resolved_packs()
        .into_iter()
        .filter(|pack| matches!(pack.manifest.apply, PackApply::ReShadeIni { .. }))
        .collect()
}

/// Local cache only — no network sync (for preset list in UI).
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

pub fn find_reshade_ini_pack_cached(
    game_id: Option<&str>,
    engine_family: Option<&str>,
) -> Option<ResolvedPack> {
    reshade_ini_packs()
        .into_iter()
        .find(|pack| pack.matches(game_id, engine_family, None))
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
    if forza_pack_ready(None) {
        return Ok(());
    }
    if effective_base_url().is_none() {
        return Err(crate::i18n::t("Не удалось загрузить пресеты Forza.", "Failed to load Forza presets."));
    }
    sync_pack_by_id("forza-fh6", force)?;
    Ok(())
}

pub fn find_forza_pack(game_id: Option<&str>) -> Option<ResolvedPack> {
    find_pack(game_id, Some("forza"), None)
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
    fn ue_json_pack_prefers_game_specific_match() {
        let sn2 = PackMatch {
            steam_app_ids: vec!["1962700".into()],
            game_ids: vec!["steam-1962700".into()],
            engine_families: vec!["ue5".into()],
            overlay_ids: vec![],
        };
        let generic = PackMatch {
            steam_app_ids: vec![],
            game_ids: vec![],
            engine_families: vec!["ue5".into(), "ue4".into()],
            overlay_ids: vec![],
        };
        assert!(pack_matches(
            &sn2,
            Some("steam-1962700"),
            Some("ue5"),
            None
        ));
        assert!(pack_matches(&generic, None, Some("ue5"), None));
        assert!(pack_matches(
            &generic,
            Some("steam-1962700"),
            Some("ue5"),
            None
        ));
    }

    #[test]
    fn sn2_reshade_pack_does_not_match_unrelated_ue5() {
        let rules = PackMatch {
            steam_app_ids: vec!["1962700".into()],
            game_ids: vec!["steam-1962700".into()],
            engine_families: vec!["ue5".into()],
            overlay_ids: vec![],
        };
        assert!(!pack_matches(
            &rules,
            Some("steam-2483190"),
            Some("ue5"),
            None
        ));
    }

    #[test]
    fn sn2_reshade_pack_matches_epic_game_id() {
        let rules = PackMatch {
            steam_app_ids: vec!["1962700".into()],
            game_ids: vec!["steam-1962700".into()],
            engine_families: vec!["ue5".into()],
            overlay_ids: vec![],
        };
        assert!(pack_matches(
            &rules,
            Some("epic-Subnautica2"),
            Some("ue5"),
            None
        ));
        assert!(!pack_matches(
            &rules,
            Some("epic-Subnautica2"),
            None,
            None
        ));
        assert!(!pack_matches(
            &rules,
            Some("steam-1962700"),
            Some("ue4"),
            None
        ));
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
