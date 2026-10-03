use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct SnapshotMetadata {
    pub name: Option<String>,
    pub game_id: Option<String>,
    pub source_dir: String,
    pub created_at: String,
}
pub(crate) fn read(dir: &Path, id: &str) -> Option<SnapshotMetadata> {
    if !crate::fs_util::is_safe_backup_id(id) {
        return None;
    }
    let path = super::paths::backup_store_dir(dir).join(format!("{id}.json"));
    let bytes = crate::fs_util::read_file_bytes(&path).ok()?;
    if bytes.len() > 8192 {
        return None;
    }
    serde_json::from_slice(&bytes).ok()
}
pub(crate) fn write(
    dir: &Path,
    id: &str,
    name: Option<String>,
    game_id: Option<String>,
) -> Result<(), String> {
    super::paths::resolve_backup_path(dir, id)?;
    if name
        .as_ref()
        .is_some_and(|name| name.trim().is_empty() || name.len() > 120)
    {
        return Err(crate::i18n::t(
            "Недопустимое имя снимка",
            "Invalid snapshot name",
        ));
    }
    let created_at = read(dir, id)
        .map(|metadata| metadata.created_at)
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let metadata = SnapshotMetadata {
        name,
        game_id,
        source_dir: dir.to_string_lossy().into(),
        created_at,
    };
    let path = super::paths::backup_store_dir(dir).join(format!("{id}.json"));
    crate::profiles::write_json_atomic(
        &path,
        &serde_json::to_string(&metadata).map_err(|e| e.to_string())?,
    )
}
