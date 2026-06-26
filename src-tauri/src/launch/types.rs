#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct LaunchResult {
    pub launcher: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}
