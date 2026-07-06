use crate::catalog::{
    get_or_build_catalog_index, parse_ue_semver, reference_applies_to_version, CatalogIndex,
    UeSemver,
};
use crate::core::app_error::{AppError, AppInvokeError};
use crate::core::models::CustomChanges;
use crate::gpu::{detect_gpu, GpuCapabilities};
use crate::ini::parser::read_ini_file;
use crate::scalability::{
    detect_scalability_limits, is_scalability_quality_index, ScalabilityLimits,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;

const GUS_INI: &str = "GameUserSettings.ini";
const ENGINE_INI_FILES: &[&str] = &["Engine.ini", "Scalability.ini", "Game.ini"];
const RT_CVAR_KEYS: &[&str] = &[
    "r.raytracing",
    "r.raytracing.enabled",
    "r.lumen.hardwareraytracing",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum IssueSeverity {
    Error,
    Warning,
}

struct SemanticIssue {
    code: &'static str,
    severity: IssueSeverity,
    message_ru: String,
    message_en: String,
}

pub struct SemanticValidationContext<'a> {
    pub engine_family: Option<&'a str>,
    pub engine_version: Option<&'a str>,
    pub config_path: &'a Path,
    pub install_dir: Option<&'a str>,
    pub warnings_acknowledged: bool,
}

pub fn validate_custom_changes_semantics(
    changes: &CustomChanges,
    ctx: SemanticValidationContext<'_>,
) -> Result<(), AppInvokeError> {
    let issues = collect_semantic_issues(changes, &ctx);
    gate_apply(issues, ctx.warnings_acknowledged)
}

fn collect_semantic_issues(
    changes: &CustomChanges,
    ctx: &SemanticValidationContext<'_>,
) -> Vec<SemanticIssue> {
    let is_ue4 = ctx.engine_family == Some("ue4");
    let game_version = ctx.engine_version.and_then(parse_ue_semver);
    let index = get_or_build_catalog_index(ctx.engine_family);
    let limits = detect_scalability_limits(ctx.install_dir.map(Path::new), Some(ctx.config_path));
    let gpu = detect_gpu();
    let pending_values = collect_pending_values(&changes.files);

    let mut issues = Vec::new();
    issues.extend(check_version_and_sg_limits(
        &changes.files,
        &index,
        game_version,
        is_ue4,
        &limits,
    ));
    issues.extend(check_combo_rules(&changes.files, &pending_values, &gpu));
    issues.extend(check_sg_r_conflicts(ctx.config_path, changes));
    issues.extend(check_shipped_gus_removals(
        ctx.config_path,
        &changes.removals,
    ));
    dedupe_issues(issues)
}

fn gate_apply(
    issues: Vec<SemanticIssue>,
    warnings_acknowledged: bool,
) -> Result<(), AppInvokeError> {
    let has_errors = issues
        .iter()
        .any(|issue| issue.severity == IssueSeverity::Error);
    if has_errors {
        let message = issues
            .iter()
            .filter(|issue| issue.severity == IssueSeverity::Error)
            .map(|issue| crate::i18n::t(&issue.message_ru, &issue.message_en))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(AppError::validation(message));
    }

    let has_warnings = issues
        .iter()
        .any(|issue| issue.severity == IssueSeverity::Warning);
    if has_warnings && !warnings_acknowledged {
        return Err(AppError::validation(crate::i18n::t(
            "Подтвердите предупреждения валидации перед apply",
            "Acknowledge validation warnings before apply",
        )));
    }

    Ok(())
}

fn check_version_and_sg_limits(
    files: &HashMap<String, HashMap<String, HashMap<String, String>>>,
    index: &CatalogIndex,
    game_version: Option<UeSemver>,
    is_ue4: bool,
    limits: &ScalabilityLimits,
) -> Vec<SemanticIssue> {
    let mut issues = Vec::new();
    for (file, sections) in files {
        for entries in sections.values() {
            for (key, value) in entries {
                if !key_applies_to_game(index, key, file, game_version, is_ue4) {
                    let version_label = game_version
                        .map(|v| format!("{}.{}.{}", v.major, v.minor, v.patch))
                        .or_else(|| {
                            if is_ue4 {
                                Some("UE4".to_string())
                            } else {
                                Some("UE5".to_string())
                            }
                        })
                        .unwrap_or_else(|| "?".to_string());
                    issues.push(SemanticIssue {
                        code: "version_mismatch",
                        severity: IssueSeverity::Error,
                        message_ru: format!("{key} недоступен в UE {version_label}"),
                        message_en: format!("{key} is not available in UE {version_label}"),
                    });
                    continue;
                }

                if file.eq_ignore_ascii_case("GameUserSettings.ini")
                    && is_scalability_quality_index(key)
                {
                    if let Some(issue) = check_sg_limit(key, value, limits) {
                        issues.push(issue);
                    }
                }

                if let Some(reference) = index.reference_by_key.get(&key.to_ascii_lowercase()) {
                    if let Some(issue) = check_value_range(
                        key,
                        value,
                        reference.min.as_deref(),
                        reference.max.as_deref(),
                    ) {
                        issues.push(issue);
                    }
                }
            }
        }
    }
    issues
}

fn check_value_range(
    key: &str,
    value: &str,
    min: Option<&str>,
    max: Option<&str>,
) -> Option<SemanticIssue> {
    let parsed = value.trim().parse::<f64>().ok()?;
    if !parsed.is_finite() {
        return None;
    }
    if let Some(min) = min.and_then(|raw| raw.trim().parse::<f64>().ok()) {
        if parsed < min {
            return Some(SemanticIssue {
                code: "value_out_of_range",
                severity: IssueSeverity::Warning,
                message_ru: format!("{key}={value} ниже минимума ({min})"),
                message_en: format!("{key}={value} is below minimum ({min})"),
            });
        }
    }
    if let Some(max) = max.and_then(|raw| raw.trim().parse::<f64>().ok()) {
        if parsed > max {
            return Some(SemanticIssue {
                code: "value_out_of_range",
                severity: IssueSeverity::Warning,
                message_ru: format!("{key}={value} выше максимума ({max})"),
                message_en: format!("{key}={value} is above maximum ({max})"),
            });
        }
    }
    None
}

fn read_ini_keys_on_disk(path: &Path) -> HashSet<String> {
    let mut keys = HashSet::new();
    if !path.is_file() {
        return keys;
    }
    let Ok(ini) = read_ini_file(path) else {
        return keys;
    };
    for section in ini.sections.values() {
        for key in section.entries.keys() {
            keys.insert(key.to_ascii_lowercase());
        }
    }
    keys
}

fn check_shipped_gus_removals(
    config_path: &Path,
    removals: &HashMap<String, HashMap<String, Vec<String>>>,
) -> Vec<SemanticIssue> {
    if removals.is_empty() {
        return Vec::new();
    }
    let on_disk = read_ini_keys_on_disk(&config_path.join(GUS_INI));
    if on_disk.is_empty() {
        return Vec::new();
    }

    let mut issues = Vec::new();
    for (file, sections) in removals {
        if !file.eq_ignore_ascii_case(GUS_INI) {
            continue;
        }
        for keys in sections.values() {
            for key in keys {
                if !on_disk.contains(&key.to_ascii_lowercase()) {
                    continue;
                }
                issues.push(SemanticIssue {
                    code: "shipped_key_removal",
                    severity: IssueSeverity::Error,
                    message_ru: format!("Нельзя удалить ключ из ini: {key}"),
                    message_en: format!("Cannot remove shipped ini key: {key}"),
                });
            }
        }
    }
    issues
}

fn is_sg_quality_key(key: &str) -> bool {
    if is_scalability_quality_index(key) {
        return true;
    }
    let lower = key.to_ascii_lowercase();
    lower.starts_with("sg.")
        && lower != "sg.resolutionquality"
        && lower
            .strip_prefix("sg.")
            .is_some_and(|group| group.ends_with("quality"))
}

fn sg_quality_to_r_prefix(sg_key: &str) -> Option<String> {
    let lower = sg_key.to_ascii_lowercase();
    let rest = lower.strip_prefix("sg.")?;
    if !rest.ends_with("quality") {
        return None;
    }
    let group = rest.strip_suffix("quality")?;
    Some(format!("r.{group}"))
}

fn matches_sg_r_prefix_family(key: &str, prefix: &str) -> bool {
    let key_lower = key.to_ascii_lowercase();
    let prefix_lower = prefix.to_ascii_lowercase();
    if !key_lower.starts_with(&prefix_lower) {
        return false;
    }
    let rest = &key_lower[prefix_lower.len()..];
    rest.is_empty() || rest.starts_with("quality") || rest.starts_with('.')
}

type EffectiveIniKeys = HashMap<String, HashMap<String, String>>;

fn load_effective_ini_keys(config_path: &Path, changes: &CustomChanges) -> EffectiveIniKeys {
    let mut state: EffectiveIniKeys = HashMap::new();

    let files_to_read: Vec<&str> = std::iter::once(GUS_INI)
        .chain(ENGINE_INI_FILES.iter().copied())
        .collect();

    for file in files_to_read {
        let path = config_path.join(file);
        if !path.is_file() {
            continue;
        }
        let Ok(ini) = read_ini_file(&path) else {
            continue;
        };
        let file_map = state.entry(file.to_string()).or_default();
        for section in ini.sections.values() {
            for (key, value) in &section.entries {
                if value.trim().is_empty() {
                    continue;
                }
                file_map.insert(key.to_ascii_lowercase(), value.clone());
            }
        }
    }

    for (file, sections) in &changes.removals {
        let Some(file_map) = state.get_mut(file) else {
            continue;
        };
        for keys in sections.values() {
            for key in keys {
                file_map.remove(&key.to_ascii_lowercase());
            }
        }
    }

    for (file, sections) in &changes.files {
        let file_map = state.entry(file.clone()).or_default();
        for entries in sections.values() {
            for (key, value) in entries {
                let normalized = key.to_ascii_lowercase();
                if value.trim().is_empty() {
                    file_map.remove(&normalized);
                } else {
                    file_map.insert(normalized, value.clone());
                }
            }
        }
    }

    state
}

fn check_sg_r_conflicts(config_path: &Path, changes: &CustomChanges) -> Vec<SemanticIssue> {
    let state = load_effective_ini_keys(config_path, changes);
    let Some(gus_keys) = state.get(GUS_INI) else {
        return Vec::new();
    };

    let mut engine_r_keys = HashSet::new();
    for file in ENGINE_INI_FILES {
        let Some(file_map) = state.get(*file) else {
            continue;
        };
        for key in file_map.keys() {
            if key.starts_with("r.") {
                engine_r_keys.insert(key.clone());
            }
        }
    }
    if engine_r_keys.is_empty() {
        return Vec::new();
    }

    let mut issues = Vec::new();
    let mut seen_sg = HashSet::new();

    for (sg_key, sg_value) in gus_keys {
        if !is_sg_quality_key(sg_key) || sg_value.trim().is_empty() {
            continue;
        }
        if !seen_sg.insert(sg_key.clone()) {
            continue;
        }
        let Some(prefix) = sg_quality_to_r_prefix(sg_key) else {
            continue;
        };

        let conflicting: Vec<String> = engine_r_keys
            .iter()
            .filter(|r_key| matches_sg_r_prefix_family(r_key, &prefix))
            .cloned()
            .collect();
        if conflicting.is_empty() {
            continue;
        }

        issues.push(SemanticIssue {
            code: "sg_r_conflict",
            severity: IssueSeverity::Warning,
            message_ru: format!(
                "sg/r конфликт: {sg_key} пересекается с r.* ({})",
                conflicting.join(", ")
            ),
            message_en: format!(
                "sg/r conflict: {sg_key} overlaps manual r.* overrides ({})",
                conflicting.join(", ")
            ),
        });
    }

    issues
}

fn key_applies_to_game(
    index: &CatalogIndex,
    key: &str,
    file: &str,
    game_version: Option<UeSemver>,
    is_ue4: bool,
) -> bool {
    let Some(reference) = index.reference_by_key.get(&key.to_lowercase()) else {
        return true;
    };
    if !(reference.file.eq_ignore_ascii_case(file)
        || file.eq_ignore_ascii_case("Engine.ini")
        || (file.eq_ignore_ascii_case("GameUserSettings.ini") && key.starts_with("sg.")))
    {
        return true;
    }
    reference_applies_to_version(reference, game_version, is_ue4)
}

fn check_sg_limit(key: &str, value: &str, limits: &ScalabilityLimits) -> Option<SemanticIssue> {
    let max = limits.max_for(key);
    let parsed = value.trim().parse::<f64>().ok()?;
    if !parsed.is_finite() {
        return None;
    }
    if parsed > f64::from(max) {
        Some(SemanticIssue {
            code: "sg_exceeds_limit",
            severity: IssueSeverity::Error,
            message_ru: format!("{key}={value} превышает лимит sg (макс. {max})"),
            message_en: format!("{key}={value} exceeds sg limit (max {max})"),
        })
    } else {
        None
    }
}

fn collect_pending_values(
    files: &HashMap<String, HashMap<String, HashMap<String, String>>>,
) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for sections in files.values() {
        for entries in sections.values() {
            for (key, value) in entries {
                map.insert(key.to_lowercase(), value.clone());
            }
        }
    }
    map
}

fn keys_in_file(
    files: &HashMap<String, HashMap<String, HashMap<String, String>>>,
    file: &str,
) -> HashSet<String> {
    let mut keys = HashSet::new();
    let Some(sections) = files.get(file) else {
        return keys;
    };
    for entries in sections.values() {
        for key in entries.keys() {
            keys.insert(key.to_lowercase());
        }
    }
    keys
}

fn check_combo_rules(
    files: &HashMap<String, HashMap<String, HashMap<String, String>>>,
    pending_values: &HashMap<String, String>,
    gpu: &GpuCapabilities,
) -> Vec<SemanticIssue> {
    let mut issues = Vec::new();
    let engine_keys = keys_in_file(files, "Engine.ini");
    let scalability_keys = keys_in_file(files, "Scalability.ini");

    for key in &engine_keys {
        if scalability_keys.contains(key) {
            issues.push(SemanticIssue {
                code: "combo_engine_scalability_dup",
                severity: IssueSeverity::Warning,
                message_ru: format!(
                    "{key} в Engine.ini и Scalability.ini: победит последняя запись"
                ),
                message_en: format!(
                    "{key} in both Engine.ini and Scalability.ini: last write wins"
                ),
            });
        }
    }

    let rt_on = RT_CVAR_KEYS.iter().any(|key| {
        pending_values
            .get(&key.to_lowercase())
            .map(|value| is_truthy_cvar(value))
            .unwrap_or(false)
    });

    if rt_on {
        if let Some(shadow_val) = pending_values.get("sg.shadowquality") {
            if is_low_quality(shadow_val, 4) {
                issues.push(SemanticIssue {
                    code: "combo_rt_shadows",
                    severity: IssueSeverity::Warning,
                    message_ru: format!(
                        "Ray Tracing включён при низком sg.ShadowQuality ({shadow_val})"
                    ),
                    message_en: format!(
                        "Ray Tracing enabled with low sg.ShadowQuality ({shadow_val})"
                    ),
                });
            }
        }

        if !gpu.supports_ray_tracing {
            issues.push(SemanticIssue {
                code: "combo_rt_no_hw",
                severity: IssueSeverity::Warning,
                message_ru: "Ray Tracing включён, но GPU не поддерживает RT в UE".to_string(),
                message_en: "Ray Tracing enabled but GPU does not support RT in UE".to_string(),
            });
        }
    }

    if let (Some(texture_val), Some(pool_val)) = (
        pending_values.get("sg.texturequality"),
        pending_values.get("r.streaming.poolsize"),
    ) {
        if is_low_quality(texture_val, 4) {
            if let Ok(pool) = pool_val.trim().parse::<f64>() {
                if pool.is_finite() && pool > 3000.0 {
                    issues.push(SemanticIssue {
                        code: "combo_streaming_texture",
                        severity: IssueSeverity::Warning,
                        message_ru: format!(
                            "Большой r.Streaming.PoolSize ({pool_val}) при низком sg.TextureQuality ({texture_val})"
                        ),
                        message_en: format!(
                            "Large r.Streaming.PoolSize ({pool_val}) with low sg.TextureQuality ({texture_val})"
                        ),
                    });
                }
            }
        }
    }

    issues
}

fn is_truthy_cvar(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn is_low_quality(value: &str, max: u32) -> bool {
    let Ok(n) = value.trim().parse::<f64>() else {
        return false;
    };
    if !n.is_finite() {
        return false;
    }
    let threshold = (max / 2).saturating_sub(1) as f64;
    n <= threshold
}

fn dedupe_issues(mut issues: Vec<SemanticIssue>) -> Vec<SemanticIssue> {
    let mut seen = HashSet::new();
    issues.retain(|issue| seen.insert((issue.code, issue.message_en.clone())));
    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::invalidate_catalog_cache;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn ctx<'a>(
        config_path: &'a Path,
        engine_family: Option<&'a str>,
        engine_version: Option<&'a str>,
    ) -> SemanticValidationContext<'a> {
        SemanticValidationContext {
            engine_family,
            engine_version,
            config_path,
            install_dir: None,
            warnings_acknowledged: false,
        }
    }

    #[test]
    fn blocks_sg_above_limit() {
        invalidate_catalog_cache();
        let dir = TempDir::new().unwrap();
        let mut sections = HashMap::new();
        sections.insert(
            "ScalabilityGroups".to_string(),
            HashMap::from([("sg.ViewDistanceQuality".to_string(), "9".to_string())]),
        );
        let changes = CustomChanges {
            files: HashMap::from([("GameUserSettings.ini".to_string(), sections)]),
            removals: HashMap::new(),
        };
        let result =
            validate_custom_changes_semantics(&changes, ctx(dir.path(), Some("ue5"), Some("5.4")));
        assert!(result.is_err());
    }

    #[test]
    fn blocks_ue5_only_key_on_ue4() {
        invalidate_catalog_cache();
        let dir = TempDir::new().unwrap();
        let mut sections = HashMap::new();
        sections.insert(
            "SystemSettings".to_string(),
            HashMap::from([("r.Nanite".to_string(), "1".to_string())]),
        );
        let changes = CustomChanges {
            files: HashMap::from([("Engine.ini".to_string(), sections)]),
            removals: HashMap::new(),
        };
        let result =
            validate_custom_changes_semantics(&changes, ctx(dir.path(), Some("ue4"), Some("4.27")));
        assert!(result.is_err());
    }

    #[test]
    fn allows_warnings_when_acknowledged() {
        invalidate_catalog_cache();
        let dir = TempDir::new().unwrap();
        let mut engine = HashMap::new();
        engine.insert(
            "SystemSettings".to_string(),
            HashMap::from([("r.ShadowQuality".to_string(), "2".to_string())]),
        );
        let mut scalability = HashMap::new();
        scalability.insert(
            "ScalabilityGroups".to_string(),
            HashMap::from([("r.ShadowQuality".to_string(), "2".to_string())]),
        );
        let changes = CustomChanges {
            files: HashMap::from([
                ("Engine.ini".to_string(), engine),
                ("Scalability.ini".to_string(), scalability),
            ]),
            removals: HashMap::new(),
        };
        let mut validation_ctx = ctx(dir.path(), Some("ue5"), Some("5.4"));
        validation_ctx.warnings_acknowledged = true;
        assert!(validate_custom_changes_semantics(&changes, validation_ctx).is_ok());
    }

    #[test]
    fn warns_on_sg_r_conflict_from_disk_ini() {
        invalidate_catalog_cache();
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[ScalabilityGroups]\nsg.ShadowQuality=2\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("Engine.ini"),
            "[SystemSettings]\nr.ShadowQuality=5\n",
        )
        .unwrap();
        let changes = CustomChanges {
            files: HashMap::new(),
            removals: HashMap::new(),
        };
        let result =
            validate_custom_changes_semantics(&changes, ctx(dir.path(), Some("ue5"), Some("5.4")));
        assert!(result.is_err());
    }

    #[test]
    fn allows_sg_r_conflict_when_acknowledged() {
        invalidate_catalog_cache();
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[ScalabilityGroups]\nsg.ShadowQuality=2\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("Engine.ini"),
            "[SystemSettings]\nr.ShadowQuality=5\n",
        )
        .unwrap();
        let changes = CustomChanges {
            files: HashMap::new(),
            removals: HashMap::new(),
        };
        let mut validation_ctx = ctx(dir.path(), Some("ue5"), Some("5.4"));
        validation_ctx.warnings_acknowledged = true;
        assert!(validate_custom_changes_semantics(&changes, validation_ctx).is_ok());
    }

    #[test]
    fn sg_r_conflict_cleared_when_r_removed_in_payload() {
        invalidate_catalog_cache();
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[ScalabilityGroups]\nsg.ShadowQuality=2\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("Engine.ini"),
            "[SystemSettings]\nr.ShadowQuality=5\n",
        )
        .unwrap();
        let changes = CustomChanges {
            files: HashMap::new(),
            removals: HashMap::from([(
                "Engine.ini".to_string(),
                HashMap::from([(
                    "SystemSettings".to_string(),
                    vec!["r.ShadowQuality".to_string()],
                )]),
            )]),
        };
        assert!(validate_custom_changes_semantics(
            &changes,
            ctx(dir.path(), Some("ue5"), Some("5.4")),
        )
        .is_ok());
    }

    #[test]
    fn blocks_shipped_gus_key_removal() {
        invalidate_catalog_cache();
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[ScalabilityGroups]\nsg.ShadowQuality=2\n",
        )
        .unwrap();
        let changes = CustomChanges {
            files: HashMap::new(),
            removals: HashMap::from([(
                "GameUserSettings.ini".to_string(),
                HashMap::from([(
                    "ScalabilityGroups".to_string(),
                    vec!["sg.ShadowQuality".to_string()],
                )]),
            )]),
        };
        let result =
            validate_custom_changes_semantics(&changes, ctx(dir.path(), Some("ue5"), Some("5.4")));
        assert!(result.is_err());
    }
}
