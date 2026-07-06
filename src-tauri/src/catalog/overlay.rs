use crate::catalog::types::ParameterCatalogEntry;

pub(crate) const GLOBAL_CATALOG_FILES: &[&str] = &[
    "engine.json",
    "scalability.json",
    "display.json",
    "ue4.json",
    "ue_extended.json",
];

pub(crate) fn is_global_catalog_file(name: &str) -> bool {
    GLOBAL_CATALOG_FILES.contains(&name)
}

pub(crate) fn overlay_slug_from_catalog_filename(name: &str) -> Option<String> {
    if is_global_catalog_file(name) {
        return None;
    }
    let stem = name.strip_suffix(".json")?;
    if stem.is_empty() {
        return None;
    }
    Some(stem.to_string())
}

pub(crate) fn entry_applies_to_game(entry: &ParameterCatalogEntry, game_id: Option<&str>) -> bool {
    let Some(slug) = entry.overlay_slug.as_deref() else {
        return true;
    };
    let Some(gid) = game_id.filter(|id| !id.trim().is_empty()) else {
        return false;
    };
    game_id_matches_overlay_slug(gid, slug)
}

pub(crate) fn game_id_matches_overlay_slug(game_id: &str, slug: &str) -> bool {
    let slug_norm = normalize_overlay_key(slug);
    if slug_norm.is_empty() {
        return false;
    }
    let raw = game_id
        .strip_prefix("steam-")
        .or_else(|| game_id.strip_prefix("epic-"))
        .unwrap_or(game_id);
    normalize_overlay_key(raw) == slug_norm
}

fn normalize_overlay_key(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != '_')
        .flat_map(|c| c.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epic_subnautica2_matches_subnautica2_overlay() {
        assert!(game_id_matches_overlay_slug(
            "epic-Subnautica2",
            "subnautica2"
        ));
    }

    #[test]
    fn unrelated_game_does_not_match_subnautica2_overlay() {
        assert!(!game_id_matches_overlay_slug("steam-578080", "subnautica2"));
    }

    #[test]
    fn epic_subnautica_does_not_match_subnautica2_overlay() {
        assert!(!game_id_matches_overlay_slug(
            "epic-Subnautica",
            "subnautica2"
        ));
    }
}
