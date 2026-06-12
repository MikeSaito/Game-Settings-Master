use crate::models::PresetInfo;
use serde::Deserialize;
use serde::de::Error as _;
use std::path::{Path, PathBuf};

fn resolve_pack_relative_file(
    pack_root: &Path,
    root_segment: &str,
    rel: &str,
) -> Result<PathBuf, String> {
    if !crate::fs_util::is_safe_manifest_relative_path(root_segment) {
        return Err(format!("Недопустимый путь в manifest: {root_segment}"));
    }
    if !crate::fs_util::is_safe_manifest_relative_path(rel) {
        return Err(format!("Недопустимый путь в manifest: {rel}"));
    }
    let path = pack_root.join(root_segment).join(rel);
    if !crate::fs_util::path_within_root(pack_root, &path) {
        return Err(format!("Путь вне пака: {rel}"));
    }
    if !path.is_file() {
        return Err(format!("Файл не найден: {rel}"));
    }
    Ok(path)
}

fn resolve_pack_file_under_root(pack_root: &Path, rel: &str) -> Result<PathBuf, String> {
    if !crate::fs_util::is_safe_manifest_relative_path(rel) {
        return Err(format!("Недопустимый путь в manifest: {rel}"));
    }
    let path = pack_root.join(rel);
    if !crate::fs_util::path_within_root(pack_root, &path) {
        return Err(format!("Путь вне пака: {rel}"));
    }
    if !path.is_file() {
        return Err(format!("Файл не найден: {rel}"));
    }
    Ok(path)
}

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
    #[serde(rename = "reshade_ini")]
    ReShadeIni { presets_root: String },
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
pub struct ReShadeIniPresetEntry {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(alias = "ini")]
    pub ini_file: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum PackPresetEntry {
    Forza(ForzaPresetEntry),
    Json(JsonPresetEntry),
    ReShade(ReShadeIniPresetEntry),
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
                PackPresetEntry::ReShade(p) => PresetInfo {
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
        if !crate::fs_util::is_safe_pack_id(&raw.pack_id) {
            return Err(D::Error::custom(format!(
                "Недопустимый pack_id: {}",
                raw.pack_id
            )));
        }
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
        let path = match resolve_pack_file_under_root(&self.root, parameter_catalog) {
            Ok(p) => p,
            Err(_) => return None,
        };
        Some(path)
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
        if !crate::fs_util::is_safe_manifest_relative_path(media_dir) {
            return None;
        }
        if !crate::fs_util::path_within_root(&self.root, profile_dir) {
            return None;
        }
        let media = profile_dir.join(media_dir);
        if !crate::fs_util::path_within_root(&self.root, &media) {
            return None;
        }
        Some(media)
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

        if !crate::fs_util::is_safe_manifest_relative_path(&entry)
            || !crate::fs_util::is_safe_manifest_relative_path(presets_root)
            || !crate::fs_util::is_safe_manifest_relative_path(user_config_patch)
        {
            return None;
        }
        let dir = self.root.join(presets_root).join(&entry);
        if !crate::fs_util::path_within_root(&self.root, &dir) {
            return None;
        }
        let patch_path = dir.join(user_config_patch);
        if !crate::fs_util::path_within_root(&self.root, &patch_path) {
            return None;
        }
        if patch_path.is_file() {
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

    pub fn load_unity_preset_json(&self, preset_id: &str) -> Option<Result<String, String>> {
        if !crate::fs_util::is_safe_pack_id(preset_id) {
            return Some(Err(format!(
                "Недопустимый идентификатор пресета: {preset_id}"
            )));
        }
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
        Some(
            resolve_pack_relative_file(&self.root, presets_root, &rel)
                .and_then(|path| {
                    std::fs::read_to_string(&path)
                        .map_err(|e| format!("Remote Unity preset '{preset_id}' не найден: {e}"))
                }),
        )
    }

    pub fn load_catalog_json_files(&self) -> Option<Vec<PathBuf>> {
        let PackApply::Catalog { catalog_root } = &self.manifest.apply else {
            return None;
        };
        if !crate::fs_util::is_safe_manifest_relative_path(catalog_root) {
            return Some(Vec::new());
        }
        let dir = self.root.join(catalog_root);
        if !crate::fs_util::path_within_root(&self.root, &dir) || !dir.is_dir() {
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

    pub fn load_reshade_ini_path(&self, preset_id: &str) -> Option<PathBuf> {
        let PackApply::ReShadeIni { presets_root } = &self.manifest.apply else {
            return None;
        };
        let rel = self.manifest.presets.iter().find_map(|p| match p {
            PackPresetEntry::ReShade(r) if r.id == preset_id => Some(r.ini_file.clone()),
            _ => None,
        })?;
        crate::fs_util::resolve_pack_file_within_root(&self.root, presets_root, &rel)
    }

    pub fn load_reshade_ini_preset(&self, preset_id: &str) -> Option<Result<String, String>> {
        let path = self.load_reshade_ini_path(preset_id)?;
        Some(
            std::fs::read_to_string(&path)
                .map_err(|e| format!("Remote ReShade preset '{preset_id}' не найден: {e}")),
        )
    }

}

pub fn pack_matches(
    rules: &PackMatch,
    game_id: Option<&str>,
    engine_family: Option<&str>,
    overlay_id: Option<&str>,
) -> bool {
    let has_game_identity_rules = !rules.game_ids.is_empty() || !rules.steam_app_ids.is_empty();

    if let Some(gid) = game_id {
        if has_game_identity_rules {
            if !matches_game_identity(rules, gid) {
                return false;
            }
            if !rules.engine_families.is_empty() {
                match engine_family {
                    Some(family) if rules.engine_families.iter().any(|f| f == family) => {}
                    _ => return false,
                }
            }
            return true;
        }
    }

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
            let aliases = pack_match_game_id_aliases(gid);
            if rules.game_ids.iter().any(|id| aliases.iter().any(|a| a == id)) {
                matched = true;
            }
        }
    }

    if !rules.steam_app_ids.is_empty() {
        any_rule = true;
        if let Some(gid) = game_id {
            for app_id in pack_match_steam_app_ids(gid) {
                if rules.steam_app_ids.iter().any(|id| id == &app_id) {
                    matched = true;
                    break;
                }
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
        if let Some(gid) = game_id {
            if let Some(oid) = crate::discovery::overlay_preset_for_game(gid) {
                if rules.overlay_ids.iter().any(|id| id == &oid) {
                    matched = true;
                }
            }
        }
    }

    any_rule && matched
}

fn matches_game_identity(rules: &PackMatch, game_id: &str) -> bool {
    if !rules.game_ids.is_empty() {
        let aliases = pack_match_game_id_aliases(game_id);
        if rules.game_ids.iter().any(|id| aliases.iter().any(|a| a == id)) {
            return true;
        }
    }
    if !rules.steam_app_ids.is_empty() {
        for app_id in pack_match_steam_app_ids(game_id) {
            if rules.steam_app_ids.iter().any(|id| id == &app_id) {
                return true;
            }
        }
    }
    false
}

fn pack_match_game_id_aliases(game_id: &str) -> Vec<String> {
    let mut out = vec![game_id.to_string()];
    if let Some(app_id) = crate::discovery::known_app_id_for_game(game_id) {
        let steam = format!("steam-{app_id}");
        if !out.iter().any(|id| id == &steam) {
            out.push(steam);
        }
    }
    out
}

fn pack_match_steam_app_ids(game_id: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(app_id) = extract_steam_app_id(game_id) {
        out.push(app_id.to_string());
    }
    if let Some(app_id) = crate::discovery::known_app_id_for_game(game_id) {
        if !out.iter().any(|id| id == &app_id) {
            out.push(app_id);
        }
    }
    out
}

pub fn extract_steam_app_id(game_id: &str) -> Option<&str> {
    game_id.strip_prefix("steam-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_reshade_ini_pack() {
        let raw = r#"{
            "schema_version": 1,
            "pack_id": "subnautica2-reshade",
            "match": { "game_ids": ["steam-1962700"] },
            "apply": { "kind": "reshade_ini", "presets_root": "presets" },
            "presets": [{
                "id": "sn2-underwater-clarity",
                "name": "Underwater Clarity",
                "description": "test",
                "ini_file": "sn2-underwater-clarity.ini"
            }]
        }"#;
        let manifest: PackManifest = serde_json::from_str(raw).unwrap();
        assert!(matches!(manifest.apply, PackApply::ReShadeIni { .. }));
        assert_eq!(manifest.presets.len(), 1);
        match &manifest.presets[0] {
            PackPresetEntry::ReShade(p) => {
                assert_eq!(p.id, "sn2-underwater-clarity");
                assert_eq!(p.ini_file, "sn2-underwater-clarity.ini");
            }
            other => panic!("expected ReShade preset entry, got {other:?}"),
        }
    }

    #[test]
    fn load_reshade_ini_preset_reads_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let presets = dir.path().join("presets");
        std::fs::create_dir_all(&presets).unwrap();
        std::fs::write(presets.join("test.ini"), "Techniques=Clarity\n").unwrap();

        let manifest: PackManifest = serde_json::from_str(
            r#"{
                "schema_version": 1,
                "pack_id": "test-reshade",
                "match": { "game_ids": ["steam-1"] },
                "apply": { "kind": "reshade_ini", "presets_root": "presets" },
                "presets": [{
                    "id": "test",
                    "name": "Test",
                    "ini_file": "test.ini"
                }]
            }"#,
        )
        .unwrap();

        let pack = ResolvedPack {
            manifest,
            root: dir.path().to_path_buf(),
        };
        let content = pack.load_reshade_ini_preset("test").unwrap().unwrap();
        assert!(content.contains("Techniques=Clarity"));
    }

    #[test]
    fn load_reshade_ini_path_rejects_traversal() {
        let dir = tempfile::TempDir::new().unwrap();
        let presets = dir.path().join("presets");
        std::fs::create_dir_all(&presets).unwrap();
        std::fs::write(presets.join("safe.ini"), "Techniques=\n").unwrap();

        let pack = ResolvedPack {
            manifest: PackManifest {
                schema_version: 1,
                pack_id: "evil".to_string(),
                title: None,
                revision: None,
                updated_at: None,
                match_rules: PackMatch {
                    steam_app_ids: vec![],
                    game_ids: vec!["steam-1".into()],
                    engine_families: vec![],
                    overlay_ids: vec![],
                },
                apply: PackApply::ReShadeIni {
                    presets_root: "presets".to_string(),
                },
                bundle: None,
                presets: vec![PackPresetEntry::ReShade(ReShadeIniPresetEntry {
                    id: "evil".to_string(),
                    name: "Evil".to_string(),
                    description: String::new(),
                    ini_file: "..\\..\\Windows\\win.ini".to_string(),
                })],
                policy: None,
            },
            root: dir.path().to_path_buf(),
        };

        assert!(pack.load_reshade_ini_path("evil").is_none());
    }

    #[test]
    fn manifest_rejects_unsafe_pack_id() {
        let raw = r#"{
            "schema_version": 1,
            "pack_id": "../evil",
            "match": { "game_ids": ["steam-1"] },
            "apply": { "kind": "unity", "presets_root": "presets" },
            "presets": []
        }"#;
        let err = serde_json::from_str::<PackManifest>(raw).unwrap_err().to_string();
        assert!(err.contains("Недопустимый pack_id"));
    }

    #[test]
    fn load_unity_preset_rejects_unsafe_id() {
        let dir = tempfile::TempDir::new().unwrap();
        let manifest: PackManifest = serde_json::from_str(
            r#"{
                "schema_version": 1,
                "pack_id": "unity-pack",
                "match": { "game_ids": ["steam-1"] },
                "apply": { "kind": "unity", "presets_root": "presets" },
                "presets": []
            }"#,
        )
        .unwrap();
        let pack = ResolvedPack {
            manifest,
            root: dir.path().to_path_buf(),
        };
        let err = pack
            .load_unity_preset_json("../evil")
            .expect("unity kind")
            .unwrap_err();
        assert!(err.contains("Недопустимый идентификатор пресета"));
    }
}
