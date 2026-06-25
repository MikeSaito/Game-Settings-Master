use serde::Deserialize;
use std::collections::HashMap;

pub fn default_value_type() -> String {
    "string".to_string()
}

pub fn default_editable() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParameterCatalogEntry {
    pub key: String,
    pub category: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub impact: String,
    #[serde(default)]
    pub min: Option<String>,
    #[serde(default)]
    pub max: Option<String>,
    #[serde(default)]
    pub value_hint: Option<String>,
    #[serde(default)]
    pub title_en: Option<String>,
    #[serde(default)]
    pub description_en: Option<String>,
    #[serde(default)]
    pub impact_en: Option<String>,
    #[serde(default)]
    pub value_hint_en: Option<String>,
    #[serde(default)]
    pub in_game_label: Option<String>,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default = "default_value_type")]
    pub value_type: String,
    #[serde(default = "default_editable")]
    pub editable: bool,
    #[serde(default)]
    pub ui_control: Option<String>,
    #[serde(default)]
    pub step: Option<String>,
    #[serde(default)]
    pub options: Option<Vec<crate::core::models::ParameterOption>>,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub recommended: Option<String>,
    #[serde(default)]
    pub catalog_recommended: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct KeyHintEntry {
    pub key: String,
    pub category: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub impact: String,
    #[serde(default)]
    pub min: Option<String>,
    #[serde(default)]
    pub max: Option<String>,
    #[serde(default)]
    pub value_hint: Option<String>,
    #[serde(default)]
    pub title_en: Option<String>,
    #[serde(default)]
    pub description_en: Option<String>,
    #[serde(default)]
    pub impact_en: Option<String>,
    #[serde(default)]
    pub value_hint_en: Option<String>,
    #[serde(default = "default_value_type")]
    pub value_type: String,
    #[serde(default = "default_editable")]
    pub editable: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ReferenceEntry {
    pub key: String,
    pub file: String,
    pub section: String,
    pub value_type: String,
    #[serde(default)]
    pub defaults_by_version: HashMap<String, String>,
    #[serde(default)]
    pub versions_present: Vec<String>,
    #[serde(default)]
    pub introduced_in: Option<String>,
    #[serde(default)]
    pub removed_in: Option<String>,
    pub ue4: bool,
    pub ue5: bool,
    pub category_guess: String,
    #[serde(default = "default_editable")]
    pub editable: bool,
    #[serde(default)]
    #[allow(dead_code)]
    pub source: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub title_en: Option<String>,
    #[serde(default)]
    pub description_en: Option<String>,
    #[serde(default)]
    pub impact: Option<String>,
    #[serde(default)]
    pub impact_en: Option<String>,
    #[serde(default)]
    pub min: Option<String>,
    #[serde(default)]
    pub max: Option<String>,
    #[serde(default)]
    pub value_hint: Option<String>,
    #[serde(default)]
    pub value_hint_en: Option<String>,
    #[serde(default)]
    pub options: Option<Vec<crate::core::models::ParameterOption>>,
    #[serde(default)]
    pub catalog_recommended: bool,
    #[serde(default)]
    pub description_quality: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct UeReferenceIndex {
    #[serde(default)]
    #[allow(dead_code)]
    schema_version: u32,
    entries: Vec<ReferenceEntry>,
}

pub(crate) struct CatalogIndex {
    pub by_full_id: HashMap<String, ParameterCatalogEntry>,
    pub by_file_key: HashMap<String, ParameterCatalogEntry>,
    pub by_key: HashMap<String, ParameterCatalogEntry>,
    pub key_hints: HashMap<String, KeyHintEntry>,
    pub reference_by_key: HashMap<String, ReferenceEntry>,
}

pub(crate) enum CatalogMatch<'a> {
    Entry(&'a ParameterCatalogEntry),
    Hint(&'a KeyHintEntry),
    Reference(&'a ReferenceEntry),
}

impl UeReferenceIndex {
    fn empty() -> Self {
        Self {
            schema_version: 2,
            entries: vec![],
        }
    }
}

pub(crate) fn parse_reference_index_json(content: &str) -> HashMap<String, ReferenceEntry> {
    let index: UeReferenceIndex =
        serde_json::from_str(content).unwrap_or_else(|_| UeReferenceIndex::empty());
    index
        .entries
        .into_iter()
        .map(|e| (e.key.to_lowercase(), e))
        .collect()
}
