use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Deserialize)]
struct ScalabilityTierRow {
    group: String,
    index: i32,
    cvars: HashMap<String, String>,
    ue_version: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TiersIndex {
    #[serde(default)]
    scalability_tiers: Vec<ScalabilityTierRow>,
}

static TIERS_CACHE: OnceLock<Mutex<Vec<ScalabilityTierRow>>> = OnceLock::new();

fn tiers_cache() -> &'static Mutex<Vec<ScalabilityTierRow>> {
    TIERS_CACHE.get_or_init(|| Mutex::new(load_tiers_from_disk()))
}

fn load_tiers_from_disk() -> Vec<ScalabilityTierRow> {
    let path = crate::resource_paths::catalog_dir().join("ue_reference_index.json");
    let content = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        r#"{"schema_version":2,"entries":[],"scalability_tiers":[]}"#.to_string()
    });
    serde_json::from_str::<TiersIndex>(&content)
        .map(|i| i.scalability_tiers)
        .unwrap_or_default()
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct UeSemver {
    major: u32,
    minor: u32,
    patch: u32,
}

fn parse_ue_semver(raw: &str) -> Option<UeSemver> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut parts = trimmed.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some(UeSemver {
        major,
        minor,
        patch,
    })
}

fn sg_key_to_group(key: &str) -> Option<String> {
    if !key.starts_with("sg.") {
        return None;
    }
    let rest = key.strip_prefix("sg.")?;
    if !rest.to_ascii_lowercase().ends_with("quality") {
        return None;
    }
    Some(rest.to_string())
}

fn canonical_group_name(group: &str) -> String {
    if group.is_empty() {
        return group.to_string();
    }
    let mut chars = group.chars();
    let first = chars.next().unwrap().to_uppercase().to_string();
    first + chars.as_str()
}

fn tier_label(index: i32, lang_en: bool) -> String {
    const LABELS_RU: [&str; 5] = [
        "Низкий",
        "Средний",
        "Высокий",
        "Эпический",
        "Кинематографический",
    ];
    const LABELS_EN: [&str; 5] = ["Low", "Medium", "High", "Epic", "Cinematic"];
    let labels = if lang_en { LABELS_EN } else { LABELS_RU };
    let idx = index.clamp(0, 4) as usize;
    format!("{} ({index})", labels[idx])
}

fn format_cvar_line(cvars: &HashMap<String, String>, max_cvars: usize) -> String {
    let mut pairs: Vec<_> = cvars.iter().collect();
    pairs.sort_by(|a, b| a.0.cmp(b.0));
    pairs
        .into_iter()
        .take(max_cvars)
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(" · ")
}

fn pick_tier_rows(group: &str, game_version: Option<UeSemver>) -> Vec<ScalabilityTierRow> {
    let canonical = canonical_group_name(group);
    let tiers = tiers_cache()
        .lock()
        .ok()
        .map(|g| g.clone())
        .unwrap_or_default();
    let mut by_index: HashMap<i32, ScalabilityTierRow> = HashMap::new();

    for row in tiers
        .iter()
        .filter(|t| t.group.eq_ignore_ascii_case(&canonical))
    {
        if let Some(gv) = game_version {
            if let Some(snap) = parse_ue_semver(&row.ue_version) {
                if snap.major != gv.major {
                    continue;
                }
                if snap > gv {
                    continue;
                }
            }
        }
        by_index
            .entry(row.index)
            .and_modify(|existing| {
                if row.ue_version > existing.ue_version {
                    *existing = row.clone();
                }
            })
            .or_insert_with(|| row.clone());
    }

    if by_index.is_empty() {
        for row in tiers
            .iter()
            .filter(|t| t.group.eq_ignore_ascii_case(&canonical))
        {
            by_index
                .entry(row.index)
                .and_modify(|existing| {
                    if row.ue_version > existing.ue_version {
                        *existing = row.clone();
                    }
                })
                .or_insert_with(|| row.clone());
        }
    }

    let mut indices: Vec<i32> = by_index.keys().copied().collect();
    indices.sort_unstable();
    indices.truncate(5);
    indices
        .into_iter()
        .filter_map(|i| by_index.get(&i).cloned())
        .collect()
}

fn build_hint_lines(rows: &[ScalabilityTierRow], lang_en: bool) -> String {
    rows.iter()
        .take(4)
        .map(|row| {
            let label = tier_label(row.index, lang_en);
            let cvars = format_cvar_line(&row.cvars, 4);
            format!("{label}: {cvars}")
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn fallback_tier_hint_pair(key: &str) -> (Option<String>, Option<String>) {
    if key.eq_ignore_ascii_case("sg.ResolutionQuality") {
        let ru = "Процент render scale (не индекс 0–4): 50% — экономия FPS, 100% — натив, 125%+ — supersampling";
        let en = "Render scale percentage (not a 0–4 index): 50% — save FPS, 100% — native, 125%+ — supersampling";
        return (Some(ru.to_string()), Some(en.to_string()));
    }

    let ru = "Низкий (0) | Средний (1) | Высокий (2) | Эпический (3) | Кинематографический (4)";
    let en = "Low (0) | Medium (1) | High (2) | Epic (3) | Cinematic (4)";
    (Some(ru.to_string()), Some(en.to_string()))
}

/// UE preset tier CVars for sg.*Quality keys (RU + EN strings).
pub fn build_tier_hint_pair(
    key: &str,
    engine_version: Option<&str>,
) -> (Option<String>, Option<String>) {
    let Some(group) = sg_key_to_group(key) else {
        return (None, None);
    };
    let game_ver = engine_version.and_then(parse_ue_semver);
    let rows = pick_tier_rows(&group, game_ver);
    if rows.is_empty() {
        return fallback_tier_hint_pair(key);
    }
    let ru = build_hint_lines(&rows, false);
    let en = build_hint_lines(&rows, true);
    (Some(ru), Some(en))
}

pub fn tier_hint_for_key(key: &str, engine_version: Option<&str>) -> Option<String> {
    let (ru, en) = build_tier_hint_pair(key, engine_version);
    match (ru, en) {
        (Some(ru), Some(en)) => Some(crate::i18n::t(&ru, &en)),
        _ => None,
    }
}

#[cfg(test)]
#[path = "scalability_tiers_tests.rs"]
mod tests;
