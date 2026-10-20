use super::*;
pub(super) fn semantic_issues(
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
        .filter(|op| op.kind == OperationKind::Scalar)
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
pub(super) fn add_op(
    operations: &mut Vec<ChangeOperation>,
    file: &str,
    section: &str,
    key: &str,
    before: Option<String>,
    after: Option<String>,
    kind: OperationKind,
    position: Option<u32>,
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
        kind,
        line: if kind == OperationKind::Input {
            position
        } else {
            None
        },
        occurrence: if kind == OperationKind::Scalar {
            position
        } else {
            None
        },
    });
}

pub(super) fn plan_issues(
    request: &PrepareRequest,
    operations: &[ChangeOperation],
    profile: &GameProfile,
    dir: &Path,
    gpu: &crate::gpu::GpuCapabilities,
) -> Vec<ChangeIssue> {
    if request.backup_id.is_some() || operations.is_empty() {
        return Vec::new();
    }
    let mut scalar = scalar_changes(operations);
    for op in operations
        .iter()
        .filter(|op| op.kind == OperationKind::FileText)
    {
        let old = Document::parse(op.before.as_deref().unwrap_or_default()).scalar_values();
        for entry in Document::parse(op.after.as_deref().unwrap_or_default()).entries() {
            if entry.key.starts_with(['+', '-', '!', '.']) {
                continue;
            }
            let identity = (
                entry.section.to_ascii_lowercase(),
                entry.key.to_ascii_lowercase(),
            );
            if old.get(&identity) != Some(&entry.value) {
                scalar
                    .files
                    .entry(op.file.clone())
                    .or_default()
                    .entry(entry.section)
                    .or_default()
                    .insert(entry.key, entry.value);
            }
        }
    }
    let mut issues = semantic_issues(&scalar, profile, dir, gpu);
    if operations
        .iter()
        .any(|op| op.kind == OperationKind::FileText)
    {
        issues.push(ChangeIssue {
            code: "raw_ini_edit".into(), severity: "warning".into(),
            message: crate::i18n::t(
                "Файл будет записан целиком. Проверьте синтаксис и порядок записей: не все ручные изменения можно проверить автоматически.",
                "The entire file will be written. Check syntax and entry order: manual edits cannot all be validated automatically.",
            ),
        });
    }
    let input_updates: Vec<_> = operations
        .iter()
        .filter_map(|operation| {
            let line = operation.line? as usize;
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
        (op.kind == OperationKind::Scalar)
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
