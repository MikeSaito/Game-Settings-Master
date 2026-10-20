//! Prepared changes are server-owned: the UI can select operations, never supply bytes.
use crate::commands::helpers::semantic_validation::{
    collect_semantic_issues, IssueSeverity, SemanticValidationContext,
};
use crate::commands::helpers::{
    find_profile_by_id, guard_config_dir_for_read, guard_config_dir_for_write,
    resolve_write_exe_name, validate_custom_changes_payload,
};
use crate::core::app_error::{AppError, AppInvokeError};
use crate::core::models::{ApplyResult, ConfigDiffEntry, CustomChanges, GameProfile};
use crate::ini::document::Document;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

mod apply;
mod ini;
mod models;
mod prepare;
mod render;
mod store;
mod validation;
pub use apply::*;
use ini::decode;
pub(crate) use ini::{digest, encode_like, read_optional};
pub use models::*;
pub use prepare::*;
use store::{plans, Plan};
use validation::{add_op, plan_issues};
#[cfg(test)]
#[path = "changes_tests.rs"]
pub(crate) mod tests;
fn error(ru: &str, en: &str) -> AppInvokeError {
    AppError::validation(crate::i18n::t(ru, en))
}
pub(crate) fn effective_dir(profile: &GameProfile, dir: &str) -> Result<PathBuf, String> {
    let path = crate::ini::paths::inspect_config_dir(dir)?;
    let hints =
        crate::discovery::platform_hints_for_game(Some(&profile.id), Some(&profile.engine_family));
    Ok(crate::ini::platform::reconcile_config_dir(&path, &hints))
}

fn gpu_fingerprint(gpu: &crate::gpu::GpuCapabilities) -> String {
    digest(
        serde_json::to_string(&(
            &gpu.adapter_id,
            &gpu.vendor,
            gpu.dedicated_memory_mb,
            gpu.shared_memory_mb,
            &gpu.ray_tracing_status,
            gpu.supports_dlss,
            gpu.supports_dlss_fg,
            gpu.supports_ray_tracing,
        ))
        .unwrap_or_default()
        .as_bytes(),
    )
}

pub(crate) fn check_game_stopped(exe: Option<&str>) -> Result<(), AppInvokeError> {
    if let Some(exe) = exe {
        if crate::fs_util::is_exe_running_uncached(exe) {
            return Err(crate::core::app_error::running_game_ini_blocked(exe));
        }
    }
    Ok(())
}
