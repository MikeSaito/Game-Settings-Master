use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "kebab-case")]
pub enum OperationKind {
    Scalar,
    Input,
    File,
    FileText,
    Comparison,
    ComparisonFile,
}
#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct ChangeOperation {
    pub id: String,
    pub group: String,
    pub file: String,
    pub section: String,
    pub key: String,
    pub before: Option<String>,
    pub after: Option<String>,
    pub kind: OperationKind,
    #[serde(default)]
    pub line: Option<u32>,
    #[serde(default)]
    pub occurrence: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct ChangeIssue {
    pub code: String,
    pub severity: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct PreparedChanges {
    pub id: String,
    pub game_id: String,
    pub config_dir: String,
    pub operations: Vec<ChangeOperation>,
    pub issues: Vec<ChangeIssue>,
    pub revisions: BTreeMap<String, Option<String>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, specta::Type)]
pub struct PrepareRequest {
    pub game_id: String,
    pub config_dir: String,
    #[serde(default)]
    pub changes: CustomChanges,
    #[serde(default)]
    pub backup_id: Option<String>,
    #[serde(default)]
    pub restore_files: Vec<String>,
    #[serde(default)]
    pub input_updates: Vec<InputUpdate>,
    #[serde(default)]
    pub input_revision: Option<String>,
    #[serde(default)]
    pub raw_files: Vec<crate::text_editor::RawFileUpdate>,
    #[serde(default)]
    pub restore_origin_backup_id: Option<String>,
    #[serde(default)]
    pub preset_metadata: Option<crate::core::models::PresetMetadata>,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct InputUpdate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub line: u32,
    pub expected: String,
    pub value: String,
}
