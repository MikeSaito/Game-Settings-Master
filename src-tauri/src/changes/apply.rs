use super::*;
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
    crate::transaction::ensure_clear(dir)?;
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
    let writes = render::render(&plan, &selected)?;
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
    crate::transaction::execute(
        vec![crate::transaction::Target {
            dir: dir.to_path_buf(),
            writes: writes.clone(),
            expected: Some(plan.originals.clone()),
        }],
        Some(&trusted.id),
        exe.as_deref(),
    )?;
    let mut result = ApplyResult {
        post_apply_warning: None,
        applied_input_lines: Some(
            selected
                .iter()
                .filter_map(|operation| operation.line)
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
