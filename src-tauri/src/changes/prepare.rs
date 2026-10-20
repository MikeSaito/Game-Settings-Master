use super::*;
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
    if request.raw_files.len() > crate::fs_util::ALLOWED_CONFIG_INI_FILES.len() {
        return Err(error("Слишком много файлов", "Too many files"));
    }
    if !request.raw_files.is_empty()
        && (request.backup_id.is_some()
            || request.restore_origin_backup_id.is_some()
            || !request.restore_files.is_empty()
            || !request.changes.files.is_empty()
            || !request.changes.removals.is_empty()
            || !request.input_updates.is_empty()
            || request.input_revision.is_some()
            || request.preset_metadata.is_some())
    {
        return Err(error(
            "Нельзя смешивать текстовые и структурированные правки",
            "Cannot mix text and structured edits",
        ));
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
    if !request.raw_files.is_empty() {
        let mut seen = HashSet::new();
        for update in &request.raw_files {
            if !crate::fs_util::is_allowed_config_ini_filename(&update.file)
                || !seen.insert(update.file.to_ascii_lowercase())
            {
                return Err(error(
                    "Недопустимый или повторный файл",
                    "Invalid or duplicate file",
                ));
            }
            if update.text.len() > crate::text_editor::MAX_TEXT_BYTES || update.text.contains('\0')
            {
                return Err(error(
                    "Недопустимый текст или размер файла превышает 1 МБ",
                    "Invalid text or file exceeds 1 MB",
                ));
            }
            if Path::new(&update.config_dir) != dir {
                return Err(error(
                    "Папка конфигов изменилась: перечитайте файл",
                    "Config folder changed: reload the file",
                ));
            }
            let before = originals.get(&update.file).cloned().flatten();
            if before.as_ref().map(|bytes| digest(bytes)) != update.revision {
                return Err(error(
                    "Файл изменился: перечитайте файл перед сохранением",
                    "File changed: reload it before saving",
                ));
            }
            let after = encode_like(&update.text, before.as_ref(), None);
            if after.len() > crate::text_editor::MAX_TEXT_BYTES {
                return Err(error("Файл превышает 1 МБ", "File exceeds 1 MB"));
            }
            if before.as_ref() == Some(&after) || (before.is_none() && update.text.is_empty()) {
                continue;
            }
            add_op(
                &mut operations,
                &update.file,
                "",
                "*",
                before
                    .as_ref()
                    .map(|_| decode(before.as_ref()))
                    .transpose()?,
                Some(update.text.clone()),
                OperationKind::FileText,
                None,
                &format!("file:{}", update.file),
            );
            replacements.insert(update.file.clone(), Some(after));
        }
    } else if let Some(id) = &request.backup_id {
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
                    OperationKind::File,
                    None,
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
                OperationKind::File,
                None,
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
                            OperationKind::Scalar,
                            Some(occurrence as u32),
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
                    OperationKind::Scalar,
                    None,
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
                    OperationKind::Input,
                    Some(entry.line as u32),
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
