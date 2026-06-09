use crate::models::{PresetDefinition, PresetInfo};
use serde::Deserialize;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct PresetCatalog {
    pub schema_version: u32,
    pub catalog_id: String,
    pub version: String,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    pub packs: Vec<CatalogPackRef>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct CatalogPackRef {
    pub id: String,
    pub manifest_url: String,
    #[serde(default)]
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackMatch {
    #[serde(default)]
    pub steam_app_ids: Vec<String>,
    #[serde(default)]
    pub game_ids: Vec<String>,
    #[serde(default)]
    pub engine_families: Vec<String>,
    #[serde(default)]
    pub overlay_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackBundle {
    pub file: String,
    #[serde(default)]
    pub sha256: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind")]
pub enum PackApply {
    #[serde(rename = "forza")]
    Forza {
        #[serde(alias = "profiles_root")]
        presets_root: String,
        #[serde(alias = "preset_xml")]
        user_config_patch: String,
        media_dir: String,
        #[serde(default = "default_parameter_catalog")]
        parameter_catalog: String,
    },
    #[serde(rename = "ue_overlay")]
    UeOverlay {
        overlay_id: String,
        overlay_file: String,
    },
    #[serde(rename = "unity")]
    Unity { presets_root: String },
    #[serde(rename = "ue_json")]
    UeJson {
        presets_root: String,
        #[serde(default = "default_engines_root")]
        engines_root: String,
    },
    #[serde(rename = "catalog")]
    Catalog { catalog_root: String },
}

fn default_engines_root() -> String {
    "engines".to_string()
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PackPolicy {
    #[serde(default)]
    pub preserve_settings: Vec<String>,
    #[serde(default)]
    pub preserve_selections: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForzaPresetEntry {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub profile_folder: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonPresetEntry {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub definition_file: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum PackPresetEntry {
    Forza(ForzaPresetEntry),
    Json(JsonPresetEntry),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PackManifest {
    pub schema_version: u32,
    pub pack_id: String,
    pub title: Option<String>,
    pub revision: Option<String>,
    pub updated_at: Option<String>,
    pub match_rules: PackMatch,
    pub apply: PackApply,
    pub bundle: Option<PackBundle>,
    pub presets: Vec<PackPresetEntry>,
    pub policy: Option<PackPolicy>,
}

fn default_parameter_catalog() -> String {
    "parameter-catalog.json".to_string()
}

impl PackManifest {
    pub fn presets_info(&self) -> Vec<PresetInfo> {
        self.presets
            .iter()
            .map(|entry| match entry {
                PackPresetEntry::Forza(p) => PresetInfo {
                    id: p.id.clone(),
                    name: p.name.clone(),
                    description: p.description.clone(),
                },
                PackPresetEntry::Json(p) => PresetInfo {
                    id: p.id.clone(),
                    name: p.name.clone(),
                    description: p.description.clone(),
                },
            })
            .collect()
    }
}

impl<'de> Deserialize<'de> for PackManifest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            schema_version: u32,
            pack_id: String,
            #[serde(default)]
            title: Option<String>,
            #[serde(default)]
            revision: Option<String>,
            #[serde(default)]
            updated_at: Option<String>,
            #[serde(rename = "match")]
            match_rules: PackMatch,
            apply: PackApply,
            #[serde(default)]
            bundle: Option<PackBundle>,
            #[serde(default)]
            presets: Vec<PackPresetEntry>,
            #[serde(default)]
            policy: Option<PackPolicy>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Ok(PackManifest {
            schema_version: raw.schema_version,
            pack_id: raw.pack_id,
            title: raw.title,
            revision: raw.revision,
            updated_at: raw.updated_at,
            match_rules: raw.match_rules,
            apply: raw.apply,
            bundle: raw.bundle,
            presets: raw.presets,
            policy: raw.policy,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedPack {
    pub manifest: PackManifest,
    pub root: PathBuf,
}

impl ResolvedPack {
    pub fn matches(
        &self,
        game_id: Option<&str>,
        engine_family: Option<&str>,
        overlay_id: Option<&str>,
    ) -> bool {
        pack_matches(
            &self.manifest.match_rules,
            game_id,
            engine_family,
            overlay_id,
        )
    }

    pub fn forza_parameter_catalog_path(&self) -> Option<PathBuf> {
        let PackApply::Forza {
            parameter_catalog, ..
        } = &self.manifest.apply
        else {
            return None;
        };
        let path = self.root.join(parameter_catalog);
        if path.is_file() {
            Some(path)
        } else {
            None
        }
    }

    pub fn forza_pack_has_profiles(&self) -> bool {
        let PackApply::Forza { .. } = &self.manifest.apply else {
            return false;
        };
        self.manifest.presets.iter().any(|p| {
            if let PackPresetEntry::Forza(f) = p {
                self.forza_profile_dir(&f.id).is_some()
            } else {
                false
            }
        })
    }

    pub fn forza_media_src(&self, profile_dir: &std::path::Path) -> Option<PathBuf> {
        let PackApply::Forza { media_dir, .. } = &self.manifest.apply else {
            return None;
        };
        Some(profile_dir.join(media_dir))
    }

    pub fn forza_user_config_patch_file(&self) -> Option<&str> {
        match &self.manifest.apply {
            PackApply::Forza {
                user_config_patch, ..
            } => Some(user_config_patch.as_str()),
            _ => None,
        }
    }

    pub fn forza_profile_dir(&self, preset_id: &str) -> Option<PathBuf> {
        let PackApply::Forza {
            presets_root,
            user_config_patch,
            ..
        } = &self.manifest.apply
        else {
            return None;
        };

        let entry = self.manifest.presets.iter().find_map(|p| match p {
            PackPresetEntry::Forza(f) if f.id == preset_id => Some(f.profile_folder.clone()),
            _ => None,
        })?;

        let dir = self.root.join(presets_root).join(&entry);
        if dir.join(user_config_patch).is_file() {
            Some(dir)
        } else {
            None
        }
    }

    pub fn policy_path(&self) -> PathBuf {
        self.root.join("policy.json")
    }

    pub fn load_policy(&self) -> Option<PackPolicy> {
        if let Some(policy) = &self.manifest.policy {
            return Some(policy.clone());
        }
        let path = self.policy_path();
        if !path.is_file() {
            return None;
        }
        let raw = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&raw).ok()
    }

    pub fn load_ue_json_preset(
        &self,
        preset_id: &str,
        ue4: bool,
    ) -> Option<Result<PresetDefinition, String>> {
        let PackApply::UeJson { presets_root, .. } = &self.manifest.apply else {
            return None;
        };
        let rel = self
            .manifest
            .presets
            .iter()
            .find_map(|p| match p {
                PackPresetEntry::Json(j) if j.id == preset_id => Some(j.definition_file.clone()),
                _ => None,
            })
            .or_else(|| {
                if ue4 {
                    Some(format!("ue4/{preset_id}.json"))
                } else {
                    Some(format!("{preset_id}.json"))
                }
            })?;
        let path = self.root.join(presets_root).join(&rel);
        if !path.is_file() {
            let fallback = self
                .root
                .join(presets_root)
                .join(format!("{preset_id}.json"));
            if fallback.is_file() {
                return Some(load_preset_json(&fallback));
            }
            return Some(Err(format!("Remote UE preset '{preset_id}' не найден")));
        }
        Some(load_preset_json(&path))
    }

    pub fn load_unity_preset_json(&self, preset_id: &str) -> Option<Result<String, String>> {
        let PackApply::Unity { presets_root } = &self.manifest.apply else {
            return None;
        };
        let rel = self
            .manifest
            .presets
            .iter()
            .find_map(|p| match p {
                PackPresetEntry::Json(j) if j.id == preset_id => Some(j.definition_file.clone()),
                _ => None,
            })
            .unwrap_or_else(|| format!("{preset_id}.json"));
        let path = self.root.join(presets_root).join(&rel);
        Some(
            std::fs::read_to_string(&path)
                .map_err(|e| format!("Remote Unity preset '{preset_id}' не найден: {e}")),
        )
    }

    pub fn load_engine_ini_sections(
        &self,
        name: &str,
    ) -> Option<
        Result<
            std::collections::HashMap<String, std::collections::HashMap<String, String>>,
            String,
        >,
    > {
        let engines_root = match &self.manifest.apply {
            PackApply::UeJson { engines_root, .. } => engines_root.clone(),
            _ => return None,
        };
        let path = self.root.join(engines_root).join(format!("{name}.ini"));
        if !path.is_file() {
            return Some(Err(format!("Remote engine ini '{name}' не найден")));
        }
        Some(
            crate::ini::read_ini_file(&path)
                .map(|ini| crate::ini::parser::ini_to_data(&ini))
                .map_err(|e| format!("Не удалось прочитать engine ini: {e}")),
        )
    }

    pub fn load_catalog_json_files(&self) -> Option<Vec<PathBuf>> {
        let PackApply::Catalog { catalog_root } = &self.manifest.apply else {
            return None;
        };
        let dir = self.root.join(catalog_root);
        if !dir.is_dir() {
            return Some(Vec::new());
        }
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    files.push(path);
                }
            }
        }
        files.sort();
        Some(files)
    }

    pub fn load_ue_overlay(&self) -> Option<Result<PresetDefinition, String>> {
        let PackApply::UeOverlay {
            overlay_id: _,
            overlay_file,
        } = &self.manifest.apply
        else {
            return None;
        };

        let path = self.root.join(overlay_file);
        Some(
            std::fs::read_to_string(&path)
                .map_err(|e| format!("Remote overlay не найден: {e}"))
                .and_then(|content| {
                    serde_json::from_str(&content)
                        .map_err(|e| format!("Некорректный remote overlay: {e}"))
                }),
        )
    }
}

pub fn pack_matches(
    rules: &PackMatch,
    game_id: Option<&str>,
    engine_family: Option<&str>,
    overlay_id: Option<&str>,
) -> bool {
    let mut any_rule = false;
    let mut matched = false;

    if !rules.engine_families.is_empty() {
        any_rule = true;
        if let Some(family) = engine_family {
            if rules.engine_families.iter().any(|f| f == family) {
                matched = true;
            }
        }
    }

    if !rules.game_ids.is_empty() {
        any_rule = true;
        if let Some(gid) = game_id {
            if rules.game_ids.iter().any(|id| id == gid) {
                matched = true;
            }
        }
    }

    if !rules.steam_app_ids.is_empty() {
        any_rule = true;
        if let Some(app_id) = game_id.and_then(extract_steam_app_id) {
            if rules.steam_app_ids.iter().any(|id| id == app_id) {
                matched = true;
            }
        }
    }

    if !rules.overlay_ids.is_empty() {
        any_rule = true;
        if let Some(oid) = overlay_id {
            if rules.overlay_ids.iter().any(|id| id == oid) {
                matched = true;
            }
        }
    }

    any_rule && matched
}

pub fn extract_steam_app_id(game_id: &str) -> Option<&str> {
    game_id.strip_prefix("steam-")
}

fn load_preset_json(path: &std::path::Path) -> Result<PresetDefinition, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Не удалось прочитать {}: {e}", path.display()))?;
    serde_json::from_str(&content).map_err(|e| format!("Некорректный JSON пресета: {e}"))
}
