use crate::forza::user_config::{
    backup_forza_media, copy_preset_media, parse_user_config_patch, preview_forza_diff,
    preview_media_diff, read_user_config, rollback_media_from_snapshot, snapshot_media_for_rollback,
    tune_forza_selections, write_user_config,
};
use crate::gpu::detect_gpu;
use crate::models::{ConfigDiffEntry, PresetInfo};
use crate::remote_presets::ResolvedPack;
use std::fs;
use std::path::{Path, PathBuf};

pub fn ensure_forza_profiles_synced(game_id: Option<&str>) -> Result<(), String> {
    if crate::remote_presets::forza_pack_ready(game_id) {
        return Ok(());
    }
    if crate::remote_presets::effective_base_url().is_none() {
        return Err(crate::i18n::t("Не удалось загрузить пресеты Forza.", "Failed to load Forza presets."));
    }
    crate::remote_presets::sync_forza_pack_if_needed(false)?;
    if !crate::remote_presets::forza_pack_ready(game_id) {
        crate::remote_presets::sync_forza_pack_if_needed(true)?;
    }
    if !crate::remote_presets::forza_pack_ready(game_id) {
        return Err(crate::i18n::t(
            "Пресеты Forza недоступны. Проверьте подключение к интернету и попробуйте позже.",
            "Forza presets unavailable. Check your internet connection and try again later.",
        ));
    }
    Ok(())
}

fn resolve_forza_pack(game_id: Option<&str>) -> Result<ResolvedPack, String> {
    ensure_forza_profiles_synced(game_id)?;
    crate::remote_presets::find_forza_pack(game_id)
        .ok_or_else(|| crate::i18n::t("Пак forza-fh6 не найден в кэше.", "Pack forza-fh6 not found in cache."))
}

pub fn list_forza_presets(game_id: Option<&str>) -> Result<Vec<PresetInfo>, String> {
    if let Some(presets) = crate::remote_presets::forza_presets_from_cache(game_id) {
        if !presets.is_empty() {
            return Ok(presets);
        }
    }

    if crate::remote_presets::effective_base_url().is_some()
        && !crate::process_util::is_app_background()
    {
        let _ = crate::remote_presets::sync_forza_pack_if_needed(false);
        if let Some(presets) = crate::remote_presets::forza_presets_from_cache(game_id) {
            if !presets.is_empty() {
                return Ok(presets);
            }
        }
    }

    ensure_forza_profiles_synced(game_id)?;
    crate::remote_presets::forza_presets_from_cache(game_id)
        .ok_or_else(|| crate::i18n::t("На сервере нет списка пресетов Forza (pack forza-fh6).", "Server has no Forza preset list (pack forza-fh6)."))
}

fn preset_profile_dir(pack: &ResolvedPack, preset_id: &str) -> Result<PathBuf, String> {
    if !crate::fs_util::is_safe_pack_id(preset_id) {
        return Err(crate::i18n::t(
            &format!("Недопустимый идентификатор пресета: {preset_id}"),
            &format!("Invalid preset identifier: {preset_id}"),
        ));
    }
    pack.forza_profile_dir(preset_id)
        .ok_or_else(|| {
            crate::i18n::t(
                &format!("Профиль пресета '{preset_id}' не найден в кэше сервера"),
                &format!("Preset profile '{preset_id}' not found in server cache"),
            )
        })
}

fn load_profile_user_config_patch(pack: &ResolvedPack, preset_id: &str) -> Result<String, String> {
    let profile_dir = preset_profile_dir(pack, preset_id)?;
    let patch_name = pack.forza_user_config_patch_file().unwrap_or("Preset.xml");
    let path = profile_dir.join(patch_name);
    fs::read_to_string(&path)
        .map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось прочитать снимок UserConfig ({patch_name}): {e}"),
                &format!("Failed to read UserConfig snapshot ({patch_name}): {e}"),
            )
        })
}

pub fn preview_forza_preset(
    config_dir: &Path,
    install_dir: Option<&Path>,
    preset_id: &str,
    game_id: Option<&str>,
) -> Result<Vec<ConfigDiffEntry>, String> {
    let pack = resolve_forza_pack(game_id)?;
    let raw = load_profile_user_config_patch(&pack, preset_id)?;
    let (patch_settings, mut patch_selections) = parse_user_config_patch(&raw)?;
    let gpu = detect_gpu();
    tune_forza_selections(&mut patch_selections, &gpu);
    let mut diff = preview_forza_diff(config_dir, &patch_settings, &patch_selections)?;
    let install_raw = install_dir.ok_or_else(|| {
        crate::i18n::t("Укажите папку установки Forza — без неё предпросмотр media-файлов неполный.", "Specify Forza install folder — media file preview is incomplete without it.")
    })?;
    let install = crate::forza::validate_forza_install_dir(install_raw)?;
    let profile_dir = preset_profile_dir(&pack, preset_id)?;
    let media_src = pack
        .forza_media_src(&profile_dir)
        .ok_or_else(|| crate::i18n::t("В manifest пака не задан media_dir.", "Pack manifest has no media_dir."))?;
    diff.extend(preview_media_diff(&install, &media_src)?);
    Ok(diff)
}

pub fn apply_forza_preset(
    config_dir: &Path,
    install_dir: &Path,
    preset_id: &str,
    game_id: Option<&str>,
    backup_path: Option<&Path>,
) -> Result<(Vec<String>, Vec<ConfigDiffEntry>), String> {
    let pack = resolve_forza_pack(game_id)?;
    let raw = load_profile_user_config_patch(&pack, preset_id)?;
    let (patch_settings, mut patch_selections) = parse_user_config_patch(&raw)?;
    let gpu = detect_gpu();
    tune_forza_selections(&mut patch_selections, &gpu);

    let diff = preview_forza_diff(config_dir, &patch_settings, &patch_selections)?;
    let user_config_path = crate::forza::detect::user_config_file(config_dir);
    let original_user_config = crate::fs_util::read_file_bytes(&user_config_path)
        .map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось прочитать исходный UserConfigSelections: {e}"),
                &format!("Failed to read original UserConfigSelections: {e}"),
            )
        })?;

    let policy = pack.load_policy();
    let (mut settings, mut selections) = read_user_config(config_dir)?;
    crate::forza::user_config::merge_preset_with_policy(
        &mut settings,
        &mut selections,
        &patch_settings,
        &patch_selections,
        policy.as_ref(),
    );

    let profile_dir = preset_profile_dir(&pack, preset_id)?;
    let media_src = pack
        .forza_media_src(&profile_dir)
        .ok_or_else(|| crate::i18n::t("В manifest пака не задан media_dir.", "Pack manifest has no media_dir."))?;
    let media_snapshot = snapshot_media_for_rollback(install_dir, &media_src)?;

    if let Some(backup) = backup_path {
        backup_forza_media(install_dir, backup, &media_src)?;
    }
    let media_changed = match copy_preset_media(install_dir, &media_src) {
        Ok(changed) => changed,
        Err(e) => {
            if let Err(rb) = rollback_media_from_snapshot(&media_snapshot) {
                return Err(crate::i18n::t(
                    &format!("{e} (откат media: {rb})"),
                    &format!("{e} (media rollback: {rb})"),
                ));
            }
            return Err(e);
        }
    };

    if let Err(e) = write_user_config(config_dir, &settings, &selections) {
        let mut rollback_errors = Vec::new();
        if let Err(rb) = rollback_media_from_snapshot(&media_snapshot) {
            rollback_errors.push(rb);
        }
        if let Err(rb) = crate::fs_util::write_file_bytes(&user_config_path, &original_user_config) {
            rollback_errors.push(rb);
        }
        if rollback_errors.is_empty() {
            return Err(e);
        }
        return Err(crate::i18n::t(&format!("{e} (откат: {})", rollback_errors.join("; ")), &format!("{e} (rollback: {})", rollback_errors.join("; "))));
    }

    let mut changed = vec![user_config_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()];
    changed.extend(media_changed);

    Ok((changed, diff))
}

#[cfg(test)]
mod tests {
    use super::*;

    const HIGH_PATCH: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<Preset id="HIGH">
  <settings>
    <CubemapDrawDistanceScalar value="3.500000"/>
  </settings>
  <selections>
    <option id="ShadowQuality" value="4"/>
    <option id="DLSSMode" value="2"/>
  </selections>
</Preset>"#;

    #[test]
    fn parses_user_config_patch_from_profile_file() {
        let (settings, selections) = parse_user_config_patch(HIGH_PATCH).unwrap();
        assert!(settings.contains_key("CubemapDrawDistanceScalar"));
        assert_eq!(
            selections.get("ShadowQuality").map(String::as_str),
            Some("4")
        );
        assert_eq!(selections.get("DLSSMode").map(String::as_str), Some("2"));
    }
}
