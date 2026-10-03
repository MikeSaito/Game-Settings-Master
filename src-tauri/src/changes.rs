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

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct ChangeOperation {
    pub id: String,
    pub group: String,
    pub file: String,
    pub section: String,
    pub key: String,
    pub before: Option<String>,
    pub after: Option<String>,
    pub kind: String,
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

#[derive(Clone)]
struct Plan {
    gpu: crate::gpu::GpuCapabilities,
    public: PreparedChanges,
    request: PrepareRequest,
    profile: GameProfile,
    profile_revision: String,
    active_dir: PathBuf,
    originals: BTreeMap<String, Option<Vec<u8>>>,
    replacements: BTreeMap<String, Option<Vec<u8>>>,
    created: Instant,
}

fn plans() -> &'static Mutex<HashMap<String, Plan>> {
    static PLANS: OnceLock<Mutex<HashMap<String, Plan>>> = OnceLock::new();
    PLANS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(test)]
#[path = "changes_tests.rs"]
mod tests;

fn error(ru: &str, en: &str) -> AppInvokeError {
    AppError::validation(crate::i18n::t(ru, en))
}
pub(crate) fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn read_optional(dir: &Path, file: &str) -> Result<Option<Vec<u8>>, String> {
    let path = crate::fs_util::safe_child_path(dir, file)?;
    if !path.exists() {
        return Ok(None);
    }
    crate::fs_util::ensure_safe_child_file(dir, &path)?;
    if std::fs::metadata(&path).map_err(|e| e.to_string())?.len() > 16 * 1024 * 1024 {
        return Err(crate::i18n::t(
            "INI-файл превышает 16 МБ",
            "INI file exceeds 16 MB",
        ));
    }
    crate::fs_util::read_file_bytes(&path).map(Some)
}

fn decode(bytes: Option<&Vec<u8>>) -> Result<String, String> {
    bytes
        .map(|bytes| crate::ini::encoding::decode_bytes(bytes).map(|(text, _)| text))
        .transpose()
        .map(|text| text.unwrap_or_default())
}

pub(crate) fn encode_like(
    text: &str,
    original: Option<&Vec<u8>>,
    hint: Option<&Vec<u8>>,
) -> Vec<u8> {
    let source = original.or(hint);
    let encoding = source
        .and_then(|bytes| crate::ini::encoding::decode_bytes(bytes).ok())
        .map(|(_, encoding)| encoding)
        .unwrap_or(crate::ini::encoding::IniEncoding::Utf8);
    let mut bytes = crate::ini::encoding::encode_bytes(text, encoding);
    if encoding == crate::ini::encoding::IniEncoding::Utf8
        && source.is_some_and(|bytes| bytes.starts_with(&[0xef, 0xbb, 0xbf]))
    {
        bytes.splice(0..0, [0xef, 0xbb, 0xbf]);
    }
    bytes
}

fn effective_dir(profile: &GameProfile, dir: &str) -> Result<PathBuf, String> {
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

fn semantic_issues(
    changes: &CustomChanges,
    profile: &GameProfile,
    dir: &Path,
    gpu: &crate::gpu::GpuCapabilities,
) -> Vec<ChangeIssue> {
    collect_semantic_issues(
        changes,
        &SemanticValidationContext {
            selected_gpu: Some(gpu),
            engine_family: Some(&profile.engine_family),
            engine_version: profile.engine_version.as_deref(),
            config_path: dir,
            install_dir: Some(&profile.install_dir),
            #[cfg(test)]
            warnings_acknowledged: false,
        },
    )
    .into_iter()
    .map(|issue| ChangeIssue {
        code: issue.code.into(),
        severity: if issue.severity == IssueSeverity::Error {
            "error"
        } else {
            "warning"
        }
        .into(),
        message: crate::i18n::t(&issue.message_ru, &issue.message_en),
    })
    .collect()
}

fn scalar_changes(operations: &[ChangeOperation]) -> CustomChanges {
    let mut changes = CustomChanges::default();
    for op in operations
        .iter()
        .filter(|op| op.kind == "scalar" || op.kind.starts_with("scalar:"))
    {
        if let Some(value) = &op.after {
            changes
                .files
                .entry(op.file.clone())
                .or_default()
                .entry(op.section.clone())
                .or_default()
                .insert(op.key.clone(), value.clone());
        } else {
            changes
                .removals
                .entry(op.file.clone())
                .or_default()
                .entry(op.section.clone())
                .or_default()
                .push(op.key.clone());
        }
    }
    changes
}

#[allow(clippy::too_many_arguments)]
fn add_op(
    operations: &mut Vec<ChangeOperation>,
    file: &str,
    section: &str,
    key: &str,
    before: Option<String>,
    after: Option<String>,
    kind: &str,
    group: &str,
) {
    if before == after {
        return;
    }
    let id = operations.len().to_string();
    operations.push(ChangeOperation {
        id,
        group: group.into(),
        file: file.into(),
        section: section.into(),
        key: key.into(),
        before,
        after,
        kind: kind.into(),
    });
}

fn plan_issues(
    request: &PrepareRequest,
    operations: &[ChangeOperation],
    profile: &GameProfile,
    dir: &Path,
    gpu: &crate::gpu::GpuCapabilities,
) -> Vec<ChangeIssue> {
    if request.backup_id.is_some() || operations.is_empty() {
        return Vec::new();
    }
    let mut issues = semantic_issues(&scalar_changes(operations), profile, dir, gpu);
    let input_updates: Vec<_> = operations
        .iter()
        .filter_map(|operation| {
            let line = operation
                .kind
                .strip_prefix("input:")?
                .parse::<usize>()
                .ok()?;
            Some((line, operation.after.clone()?))
        })
        .collect();
    if !input_updates.is_empty() {
        let check = read_optional(dir, "Input.ini")
            .and_then(|bytes| decode(bytes.as_ref()))
            .and_then(|text| {
                crate::input::validate_array_uniqueness(&Document::parse(&text), &input_updates)
            });
        if let Err(message) = check {
            issues.push(ChangeIssue {
                code: "input_array_collision".into(),
                severity: "error".into(),
                message,
            });
        }
    }
    if operations.iter().any(|op| {
        (op.kind == "scalar" || op.kind.starts_with("scalar:"))
            && op.after.as_ref().is_some_and(|value| {
                let key = op.key.to_ascii_lowercase();
                let value = value.to_ascii_lowercase();
                let enabled = !matches!(
                    value.as_str(),
                    "0" | "false" | "off" | "disabled" | "none" | "u_none"
                );
                let plugin_key =
                    key.contains("dlss") || key.contains("fsr") || key.contains("xess");
                let plugin_choice = key.contains("upscaling")
                    && (value.contains("dlss") || value.contains("fsr") || value.contains("xess"));
                enabled && (plugin_key || plugin_choice || key == "upscalingframegeneration")
            })
    }) {
        issues.push(ChangeIssue { code: "game_plugin_unknown".into(), severity: "warning".into(), message: crate::i18n::t("Наличие GPU не подтверждает поддержку DLSS, FSR, XeSS или генерации кадров этой игрой. Проверьте возможности игрового плагина", "GPU hardware does not confirm this game's support for DLSS, FSR, XeSS or frame generation. Verify the game plugin's capabilities") });
    }
    if let Some(meta) = &request.preset_metadata {
        let mut warning = |code: &str, ru: &str, en: &str| {
            issues.push(ChangeIssue {
                code: code.into(),
                severity: "warning".into(),
                message: crate::i18n::t(ru, en),
            })
        };
        if !meta.source_game_id.is_empty() && meta.source_game_id != profile.id {
            warning(
                "preset_game",
                "Пресет создан для другой игры",
                "Preset was created for another game",
            );
        }
        if meta
            .engine_family
            .as_deref()
            .is_some_and(|engine| engine != profile.engine_family)
            || meta
                .engine_version
                .as_ref()
                .is_some_and(|version| profile.engine_version.as_ref() != Some(version))
        {
            warning(
                "preset_engine",
                "Версия UE отличается или неизвестна",
                "UE version differs or is unknown",
            );
        }
        if meta
            .game_build
            .as_ref()
            .is_some_and(|build| profile.build_id.as_ref() != Some(build))
        {
            warning(
                "preset_build",
                "Сборка игры отличается или неизвестна",
                "Game build differs or is unknown",
            );
        }
        let vendor = match gpu.vendor {
            crate::gpu::types::GpuVendor::Nvidia => "nvidia",
            crate::gpu::types::GpuVendor::Amd => "amd",
            crate::gpu::types::GpuVendor::Intel => "intel",
            crate::gpu::types::GpuVendor::Unknown => "unknown",
        };
        if meta
            .gpu_vendor
            .as_ref()
            .is_some_and(|expected| expected != vendor)
        {
            warning(
                "preset_vendor",
                "Производитель выбранной GPU отличается от требований пресета",
                "Selected GPU vendor differs from preset requirements",
            );
        }
        if let Some(required) = meta.min_dedicated_memory_mb {
            if gpu
                .dedicated_memory_mb
                .is_none_or(|available| available < required)
            {
                warning(
                    "preset_memory",
                    "Недостаточно выделенной видеопамяти или её объём неизвестен",
                    "Dedicated GPU memory is insufficient or unknown",
                );
            }
        }
        if meta.requires_ray_tracing && gpu.ray_tracing_status.as_deref() != Some("supported") {
            warning(
                "preset_ray_tracing",
                "Поддержка трассировки выбранной GPU отсутствует или неизвестна",
                "Selected GPU ray tracing support is absent or unknown",
            );
        }
    }
    issues
}

#[tauri::command]
pub fn prepare_changes(request: PrepareRequest) -> Result<PreparedChanges, AppInvokeError> {
    crate::profiles::validate_override_bounds(&crate::core::models::GameOverride {
        game_id: request.game_id.clone(),
        name: "Preview".into(),
        files: request.changes.files.clone(),
        removals: request.changes.removals.clone(),
        input_updates: Some(request.input_updates.clone()),
        metadata: request.preset_metadata.clone(),
    })?;
    if request.backup_id.is_some() {
        guard_config_dir_for_read(Some(&request.game_id), &request.config_dir)?;
    } else {
        guard_config_dir_for_write(Some(&request.game_id), &request.config_dir)?;
    }
    let profile = find_profile_by_id(&request.game_id)?
        .ok_or_else(|| error("Игра не найдена", "Game not found"))?;
    let gpu = crate::gpu::for_profile(&profile);
    let active_dir = effective_dir(&profile, &request.config_dir)?;
    let hints =
        crate::discovery::platform_hints_for_game(Some(&profile.id), Some(&profile.engine_family));
    let origin = request
        .backup_id
        .as_ref()
        .or(request.restore_origin_backup_id.as_ref());
    let dir = if let Some(id) = origin {
        crate::backup::resolve_backup_config_dir(&active_dir, id, &hints)?
    } else {
        active_dir.clone()
    };
    let exe = resolve_write_exe_name(None, Some(&profile.id))?;
    check_game_stopped(exe.as_deref())?;
    validate_custom_changes_payload(&request.changes, &dir)?;
    if request.input_updates.len() > 512 {
        return Err(error("Слишком много привязок", "Too many input bindings"));
    }
    let mut originals: BTreeMap<String, Option<Vec<u8>>> = BTreeMap::new();
    for file in crate::fs_util::ALLOWED_CONFIG_INI_FILES {
        originals.insert(file.into(), read_optional(&dir, file)?);
    }
    if let Some(revision) = &request.input_revision {
        let actual = digest(
            originals
                .get("Input.ini")
                .and_then(Option::as_ref)
                .map(Vec::as_slice)
                .unwrap_or_default(),
        );
        if &actual != revision {
            return Err(error(
                "Input.ini изменился: обновите конфиг",
                "Input.ini changed: refresh the config",
            ));
        }
    }
    let mut replacements = BTreeMap::new();
    let mut operations = Vec::new();
    if let Some(id) = &request.backup_id {
        if !request.changes.files.is_empty()
            || !request.changes.removals.is_empty()
            || !request.input_updates.is_empty()
        {
            return Err(error(
                "Нельзя смешивать восстановление и правки",
                "Cannot mix restore and edits",
            ));
        }
        let snapshot = crate::backup::paths::resolve_backup_path(&dir, id)?;
        let files: Vec<String> = if request.restore_files.is_empty() {
            crate::fs_util::ALLOWED_CONFIG_INI_FILES
                .iter()
                .map(|name| (*name).into())
                .collect()
        } else {
            request.restore_files.clone()
        };
        for file in files {
            if !crate::fs_util::is_allowed_config_ini_filename(&file) {
                return Err(error("Недопустимый файл", "Invalid file"));
            }
            let before = originals.get(&file).cloned().flatten();
            let after = read_optional(&snapshot, &file)?;
            if after.is_none() && file == "GameUserSettings.ini" {
                continue;
            }
            if before == after {
                continue;
            }
            let before_text = decode(before.as_ref())?;
            let after_text = decode(after.as_ref())?;
            let old = Document::parse(&before_text).occurrence_values();
            let new = Document::parse(&after_text).occurrence_values();
            for (section, key) in old
                .keys()
                .chain(new.keys())
                .collect::<std::collections::BTreeSet<_>>()
            {
                add_op(
                    &mut operations,
                    &file,
                    section,
                    key,
                    old.get(&(section.clone(), key.clone())).cloned(),
                    new.get(&(section.clone(), key.clone())).cloned(),
                    "file",
                    &format!("file:{file}"),
                );
            }
            // Include byte identity even for empty files and comment/encoding changes.
            add_op(
                &mut operations,
                &file,
                "",
                "*",
                before.as_ref().map(|bytes| digest(bytes)),
                after.as_ref().map(|bytes| digest(bytes)),
                "file",
                &format!("file:{file}"),
            );
            replacements.insert(file, after);
        }
    } else {
        let (width, height) = crate::presets::resolve_apply_resolution(&dir);
        let touched: std::collections::BTreeSet<_> = request
            .changes
            .files
            .keys()
            .chain(request.changes.removals.keys())
            .collect();
        for file in touched {
            let before = originals.get(file).cloned().flatten();
            let text = decode(before.as_ref())?;
            let mut document = Document::parse(&text);
            let existing = crate::ini::parser::parse_ini(&text);
            let updates = request
                .changes
                .files
                .get(file)
                .map(|sections| crate::presets::resolve::resolve_sections(sections, width, height))
                .unwrap_or_default();
            let updates = crate::ini::expand_mirror_key_updates(&existing, &updates);
            let removals: HashMap<String, Vec<String>> = request
                .changes
                .removals
                .get(file)
                .map(|sections| {
                    sections
                        .iter()
                        .map(|(section, keys)| {
                            (
                                crate::fs_util::normalize_ini_section_name(section),
                                keys.clone(),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            let old = document.scalar_values();
            let requested_keys: Vec<_> = updates
                .values()
                .flat_map(|entries| entries.keys())
                .chain(removals.values().flatten())
                .collect();
            if requested_keys
                .iter()
                .any(|key| key.starts_with(['+', '-', '!', '.']))
            {
                return Err(error("Операции массивов требуют структурированного редактора или восстановления файла", "Array operations require a structured editor or file restore"));
            }
            if file == "Input.ini"
                && requested_keys.iter().any(|key| {
                    key.trim_start_matches(['+', '-', '!', '.'])
                        .eq_ignore_ascii_case("ActionMappings")
                        || key
                            .trim_start_matches(['+', '-', '!', '.'])
                            .eq_ignore_ascii_case("AxisMappings")
                        || key
                            .trim_start_matches(['+', '-', '!', '.'])
                            .eq_ignore_ascii_case("AxisConfig")
                })
            {
                return Err(error(
                    "Привязки требуют редактора управления",
                    "Bindings require the input editor",
                ));
            }
            document.patch(&updates, &removals);
            let new = document.scalar_values();
            let original_entries = Document::parse(&text).entries();
            let final_entries = document.entries();
            for (section, key) in old
                .keys()
                .chain(new.keys())
                .collect::<std::collections::BTreeSet<_>>()
            {
                let group = if file == "GameUserSettings.ini"
                    && (key.contains("resolutionsize") || key.contains("desiredscreen"))
                {
                    "resolution".into()
                } else {
                    format!("{file}|{key}")
                };
                let previous: Vec<_> = original_entries
                    .iter()
                    .filter(|entry| {
                        entry.section.eq_ignore_ascii_case(section)
                            && entry.key.eq_ignore_ascii_case(key)
                    })
                    .collect();
                let next: Vec<_> = final_entries
                    .iter()
                    .filter(|entry| {
                        entry.section.eq_ignore_ascii_case(section)
                            && entry.key.eq_ignore_ascii_case(key)
                    })
                    .collect();
                if previous.len() > 1
                    && previous
                        .iter()
                        .map(|entry| &entry.value)
                        .collect::<Vec<_>>()
                        != next.iter().map(|entry| &entry.value).collect::<Vec<_>>()
                {
                    let after = new.get(&(section.clone(), key.clone())).cloned();
                    for (occurrence, entry) in previous.iter().enumerate() {
                        add_op(
                            &mut operations,
                            file,
                            &entry.section,
                            &entry.key,
                            Some(entry.value.clone()),
                            after.clone(),
                            &format!("scalar:{occurrence}"),
                            &group,
                        );
                    }
                    continue;
                }
                // Use original spelling so writing a new section does not change conventions.
                let source = document
                    .entries()
                    .into_iter()
                    .chain(Document::parse(&text).entries())
                    .find(|entry| {
                        entry.section.eq_ignore_ascii_case(section)
                            && entry.key.eq_ignore_ascii_case(key)
                    });
                let (section_name, key_name) = source
                    .map(|entry| (entry.section, entry.key))
                    .unwrap_or((section.clone(), key.clone()));
                add_op(
                    &mut operations,
                    file,
                    &section_name,
                    &key_name,
                    old.get(&(section.clone(), key.clone())).cloned(),
                    new.get(&(section.clone(), key.clone())).cloned(),
                    "scalar",
                    &group,
                );
            }
        }
        if !request.input_updates.is_empty() {
            if request.changes.files.contains_key("Input.ini")
                || request.changes.removals.contains_key("Input.ini")
            {
                return Err(error(
                    "Нельзя смешивать два режима Input.ini",
                    "Cannot mix Input.ini edit modes",
                ));
            }
            let text = decode(originals.get("Input.ini").and_then(Option::as_ref))?;
            let document = Document::parse(&text);
            let identified = crate::input::identified_entries(&document);
            let mut seen = HashSet::new();
            for update in &request.input_updates {
                let entry = identified
                    .iter()
                    .find(|(entry, id)| {
                        update
                            .id
                            .as_ref()
                            .map(|expected_id| id == expected_id)
                            .unwrap_or(entry.line == update.line as usize)
                    })
                    .map(|(entry, _)| entry.clone())
                    .ok_or_else(|| {
                        error(
                            "Привязка изменилась: обновите конфиг",
                            "Binding changed: refresh the config",
                        )
                    })?;
                if !seen.insert(entry.line) {
                    return Err(error("Повторная правка записи", "Duplicate entry update"));
                }
                if ((update.id.is_none() || request.input_revision.is_some())
                    && entry.value != update.expected)
                    || !crate::fs_util::is_safe_ini_value(&update.value)
                {
                    return Err(error(
                        "Привязка изменилась или значение недопустимо",
                        "Binding changed or value is invalid",
                    ));
                }
                // Typed input validation is shared with the input reader.
                crate::input::validate_update(&document, &entry, &update.value)?;
                add_op(
                    &mut operations,
                    "Input.ini",
                    &entry.section,
                    &entry.key,
                    Some(entry.value),
                    Some(update.value.clone()),
                    &format!("input:{}", entry.line),
                    &format!("input:{}", entry.line),
                );
            }
        }
    }
    let issues = plan_issues(&request, &operations, &profile, &dir, &gpu);
    let revisions = originals
        .iter()
        .map(|(file, bytes)| (file.clone(), bytes.as_ref().map(|bytes| digest(bytes))))
        .collect();
    let public = PreparedChanges {
        id: uuid::Uuid::new_v4().to_string(),
        game_id: profile.id.clone(),
        config_dir: dir.to_string_lossy().into(),
        operations,
        issues,
        revisions,
    };
    let profile_revision = digest(
        serde_json::to_string(&profile)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    );
    let mut cache = plans().lock().map_err(|e| e.to_string())?;
    cache.retain(|_, plan| plan.created.elapsed() < Duration::from_secs(600));
    if cache.len() >= 16 {
        return Err(error(
            "Слишком много открытых предпросмотров",
            "Too many open previews",
        ));
    }
    cache.insert(
        public.id.clone(),
        Plan {
            gpu,
            public: public.clone(),
            request,
            profile,
            profile_revision,
            active_dir,
            originals,
            replacements,
            created: Instant::now(),
        },
    );
    Ok(public)
}

#[tauri::command]
pub fn discard_prepared_changes(plan_id: String) {
    if let Ok(mut plans) = plans().lock() {
        plans.remove(&plan_id);
    }
}

#[tauri::command]
pub fn validate_prepared_changes(
    plan_id: String,
    selected_ids: Vec<String>,
) -> Result<Vec<ChangeIssue>, AppInvokeError> {
    let cache = plans().lock().map_err(|e| e.to_string())?;
    let plan = cache
        .get(&plan_id)
        .ok_or_else(|| error("Предпросмотр устарел", "Preview expired"))?;
    let selected = selection(plan, &selected_ids)?;
    Ok(plan_issues(
        &plan.request,
        &selected,
        &plan.profile,
        Path::new(&plan.public.config_dir),
        &plan.gpu,
    ))
}

fn selection(plan: &Plan, ids: &[String]) -> Result<Vec<ChangeOperation>, AppInvokeError> {
    let ids: HashSet<_> = ids.iter().collect();
    if ids
        .iter()
        .any(|id| !plan.public.operations.iter().any(|op| &op.id == *id))
    {
        return Err(error("Неизвестная правка", "Unknown operation"));
    }
    let selected: Vec<_> = plan
        .public
        .operations
        .iter()
        .filter(|op| ids.contains(&op.id))
        .cloned()
        .collect();
    for op in &selected {
        if plan
            .public
            .operations
            .iter()
            .any(|other| other.group == op.group && !ids.contains(&other.id))
        {
            return Err(error(
                "Связанные правки выбираются вместе",
                "Linked changes must be selected together",
            ));
        }
    }
    Ok(selected)
}

#[tauri::command]
pub fn apply_prepared_changes(
    plan_id: String,
    selected_ids: Vec<String>,
    warnings_acknowledged: bool,
) -> Result<ApplyResult, AppInvokeError> {
    // Serializes writes and consumes a preview only after a successful commit.
    let mut cache = plans().lock().map_err(|e| e.to_string())?;
    let plan = cache
        .get(&plan_id)
        .cloned()
        .ok_or_else(|| error("Предпросмотр устарел", "Preview expired"))?;
    let selected = selection(&plan, &selected_ids)?;
    if plan.request.backup_id.is_some() {
        guard_config_dir_for_read(Some(&plan.request.game_id), &plan.request.config_dir)?;
    } else {
        guard_config_dir_for_write(Some(&plan.request.game_id), &plan.request.config_dir)?;
    }
    let trusted = find_profile_by_id(&plan.profile.id)?
        .ok_or_else(|| error("Игра не найдена", "Game not found"))?;
    let context = digest(
        serde_json::to_string(&trusted)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    );
    if plan.created.elapsed() >= Duration::from_secs(600)
        || context != plan.profile_revision
        || effective_dir(&trusted, &plan.request.config_dir)? != plan.active_dir
    {
        return Err(error(
            "Контекст изменился: обновите предпросмотр",
            "Context changed: refresh the preview",
        ));
    }
    let dir = Path::new(&plan.public.config_dir);
    let exe = resolve_write_exe_name(None, Some(&trusted.id))?;
    check_game_stopped(exe.as_deref())?;
    for (file, expected) in &plan.originals {
        if &read_optional(dir, file)? != expected {
            return Err(error(
                "Файлы изменились: обновите предпросмотр",
                "Files changed: refresh the preview",
            ));
        }
    }
    let gpu = crate::gpu::for_profile(&trusted);
    if gpu_fingerprint(&gpu) != gpu_fingerprint(&plan.gpu) {
        return Err(error(
            "Возможности GPU изменились: обновите предпросмотр",
            "GPU capabilities changed: refresh the preview",
        ));
    }
    let issues = plan_issues(&plan.request, &selected, &trusted, dir, &gpu);
    if issues.iter().any(|issue| issue.severity == "error") {
        return Err(AppError::validation(
            issues
                .iter()
                .filter(|issue| issue.severity == "error")
                .map(|issue| issue.message.as_str())
                .collect::<Vec<_>>()
                .join("; "),
        ));
    }
    if !warnings_acknowledged && issues.iter().any(|issue| issue.severity == "warning") {
        return Err(error("Подтвердите предупреждения", "Acknowledge warnings"));
    }
    let mut writes = BTreeMap::new();
    for op in &selected {
        if op.kind == "file" {
            writes.insert(op.file.clone(), plan.replacements[&op.file].clone());
        } else {
            let before = plan.originals.get(&op.file).and_then(Option::as_ref);
            let current = writes
                .entry(op.file.clone())
                .or_insert_with(|| before.cloned());
            let text = decode(current.as_ref())?;
            let mut document = Document::parse(&text);
            if let Some(line) = op
                .kind
                .strip_prefix("input:")
                .and_then(|line| line.parse::<usize>().ok())
            {
                let raw = document
                    .lines
                    .get_mut(line)
                    .ok_or_else(|| error("Привязка изменилась", "Binding changed"))?;
                let ending = if raw.ends_with("\r\n") {
                    "\r\n"
                } else if raw.ends_with('\n') {
                    "\n"
                } else {
                    ""
                };
                let prefix = raw
                    .split_once('=')
                    .map(|(prefix, _)| prefix)
                    .unwrap_or(&op.key);
                *raw = format!(
                    "{prefix}={}{ending}",
                    op.after.as_deref().unwrap_or_default()
                );
            } else {
                document.set(&op.section, &op.key, op.after.as_deref());
            }
            *current = Some(encode_like(
                &document.text(),
                before,
                plan.originals
                    .get("GameUserSettings.ini")
                    .and_then(Option::as_ref),
            ));
        }
    }
    writes.retain(|file, bytes| plan.originals.get(file) != Some(bytes));
    if writes.is_empty() {
        cache.remove(&plan_id);
        return Ok(ApplyResult {
            post_apply_warning: None,
            applied_input_lines: Some(Vec::new()),
            backup_id: String::new(),
            changed_files: Vec::new(),
            diff: Vec::new(),
            effective_config_dir: Some(plan.public.config_dir),
        });
    }
    let backup_id = crate::backup::backup_config_dir(dir, None)?;
    crate::backup::metadata::write(dir, &backup_id, None, Some(trusted.id.clone()))?;
    let result = (|| -> Result<(), String> {
        for (file, bytes) in &writes {
            #[cfg(test)]
            tests::check_injected_write_error(file)?;
            let path = crate::fs_util::safe_child_path(dir, file)?;
            if let Some(bytes) = bytes {
                crate::fs_util::write_file_bytes_opts(&path, bytes, true)?;
            } else if path.exists() {
                crate::fs_util::clear_readonly(&path);
                std::fs::remove_file(&path).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    })();
    if let Err(write_error) = result {
        let rollback = writes
            .keys()
            .map(|file| -> Result<(), String> {
                let path = crate::fs_util::safe_child_path(dir, file)?;
                if let Some(bytes) = plan.originals.get(file).and_then(Option::as_ref) {
                    crate::fs_util::write_file_bytes_opts(&path, bytes, true)?;
                } else if path.exists() {
                    crate::fs_util::ensure_safe_child_file(dir, &path)?;
                    crate::fs_util::clear_readonly(&path);
                    std::fs::remove_file(path).map_err(|error| error.to_string())?;
                }
                Ok(())
            })
            .filter_map(Result::err)
            .collect::<Vec<_>>();
        if !rollback.is_empty() {
            let rollback_error = rollback.join("; ");
            return Err(AppError::validation(format!(
                "{write_error}; rollback: {rollback_error}"
            )));
        }
        return Err(AppError::validation(write_error));
    }
    let mut result = ApplyResult {
        post_apply_warning: None,
        applied_input_lines: Some(
            selected
                .iter()
                .filter_map(|operation| {
                    operation
                        .kind
                        .strip_prefix("input:")
                        .and_then(|line| line.parse().ok())
                })
                .collect(),
        ),
        backup_id,
        changed_files: writes.keys().cloned().collect(),
        diff: selected
            .iter()
            .map(|op| ConfigDiffEntry {
                file: op.file.clone(),
                section: op.section.clone(),
                key: op.key.clone(),
                old_value: op.before.clone(),
                new_value: op.after.clone().unwrap_or_default(),
            })
            .collect(),
        effective_config_dir: Some(plan.public.config_dir.clone()),
    };
    cache.remove(&plan_id);
    // Remember the authorized folder even if discovery can no longer find its INIs.
    let mut persisted = trusted.clone();
    persisted.config_dir = Some(plan.public.config_dir.clone());
    let mut warnings = Vec::new();
    if let Err(error) = crate::profiles::save_profile(&persisted) {
        warnings.push(format!("{}: {error}", error_message_profile()));
    }
    if let Err(error) = crate::diagnostics::record(&persisted, dir, &selected, &writes) {
        warnings.push(format!(
            "{}: {error}",
            crate::i18n::t(
                "Не удалось сохранить данные диагностики",
                "Unable to save diagnostic baseline"
            )
        ));
    }
    if !warnings.is_empty() {
        result.post_apply_warning = Some(format!(
            "{} {}",
            crate::i18n::t("Настройки записаны.", "Settings were written."),
            warnings.join("; ")
        ));
    }
    Ok(result)
}

fn error_message_profile() -> String {
    crate::i18n::t(
        "Не удалось сохранить профиль игры",
        "Unable to save game profile",
    )
}
