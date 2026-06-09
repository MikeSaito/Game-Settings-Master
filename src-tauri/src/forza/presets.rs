use crate::forza::user_config::{
    backup_forza_media, copy_preset_media, parse_user_config_patch,
    preview_forza_diff, preview_media_diff, read_user_config, tune_forza_selections,
    write_user_config,
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
        return Err("Не удалось загрузить пресеты Forza.".into());
    }
    crate::remote_presets::sync_now(true)?;
    if !crate::remote_presets::forza_pack_ready(game_id) {
        return Err(
            "Пресеты Forza недоступны. Проверьте подключение к интернету и попробуйте позже."
                .into(),
        );
    }
    Ok(())
}

fn resolve_forza_pack(game_id: Option<&str>) -> Result<ResolvedPack, String> {
    ensure_forza_profiles_synced(game_id)?;
    crate::remote_presets::find_forza_pack(game_id)
        .ok_or_else(|| "Пак forza-fh6 не найден в кэше.".to_string())
}

pub fn list_forza_presets(game_id: Option<&str>) -> Result<Vec<PresetInfo>, String> {
    ensure_forza_profiles_synced(game_id)?;
    crate::remote_presets::forza_presets(game_id).ok_or_else(|| {
        "На сервере нет списка пресетов Forza (pack forza-fh6).".to_string()
    })
}

fn preset_profile_dir(
    pack: &ResolvedPack,
    preset_id: &str,
) -> Result<PathBuf, String> {
    pack.forza_profile_dir(preset_id)
        .ok_or_else(|| format!("Профиль пресета '{preset_id}' не найден в кэше сервера"))
}

fn load_profile_user_config_patch(
    pack: &ResolvedPack,
    preset_id: &str,
) -> Result<String, String> {
    let profile_dir = preset_profile_dir(pack, preset_id)?;
    let patch_name = pack
        .forza_user_config_patch_file()
        .unwrap_or("Preset.xml");
    let path = profile_dir.join(patch_name);
    fs::read_to_string(&path).map_err(|e| {
        format!("Не удалось прочитать снимок UserConfig ({patch_name}): {e}")
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
    let install = install_dir.ok_or_else(|| {
        "Укажите папку установки Forza — без неё предпросмотр media-файлов неполный.".to_string()
    })?;
    let profile_dir = preset_profile_dir(&pack, preset_id)?;
    let media_src = pack
        .forza_media_src(&profile_dir)
        .ok_or_else(|| "В manifest пака не задан media_dir.".to_string())?;
    diff.extend(preview_media_diff(install, &media_src)?);
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

    let policy = pack.load_policy();
    let (mut settings, mut selections) = read_user_config(config_dir)?;
    crate::forza::user_config::merge_preset_with_policy(
        &mut settings,
        &mut selections,
        &patch_settings,
        &patch_selections,
        policy.as_ref(),
    );
    write_user_config(config_dir, &settings, &selections)?;

    let profile_dir = preset_profile_dir(&pack, preset_id)?;
    let media_src = pack
        .forza_media_src(&profile_dir)
        .ok_or_else(|| "В manifest пака не задан media_dir.".to_string())?;
    if let Some(backup) = backup_path {
        backup_forza_media(install_dir, backup, &media_src)?;
    }
    let mut changed = vec![crate::forza::detect::user_config_file(config_dir)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()];
    changed.extend(copy_preset_media(install_dir, &media_src)?);

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
        assert_eq!(selections.get("ShadowQuality").map(String::as_str), Some("4"));
        assert_eq!(selections.get("DLSSMode").map(String::as_str), Some("2"));
    }
}
