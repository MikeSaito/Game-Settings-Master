use crate::forza::detect::user_config_file;
use crate::gpu::GpuCapabilities;
use crate::models::ConfigDiffEntry;
use regex::Regex;
use roxmltree::Document;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const USER_CONFIG_FILE: &str = "UserConfigSelections";

const PRESERVE_SETTINGS: &[&str] = &[
    "ResolutionWidth",
    "ResolutionHeight",
    "Fullscreen",
    "PresentInterval",
    "MonitorRefreshPeriod",
    "FollowCarsCameraFOV",
    "DriverCameraFOV",
    "HoodCameraFOV",
    "BumperCameraFOV",
    "DLC1",
    "MasterVolume",
    "StreamerMode",
    "TAASharpness",
    "EnablePCHDR",
];

const PRESERVE_OPTIONS: &[&str] = &[
    "EnableHDR",
    "FrameRate",
    "ShowFPS",
    "ResolutionScaling",
    "MasterVolume",
    "AudioQuality",
];

#[derive(Debug, Clone)]
pub struct XmlNode {
    pub tag: String,
    pub attrs: BTreeMap<String, String>,
}

pub fn read_user_config(
    config_dir: &Path,
) -> Result<(BTreeMap<String, XmlNode>, BTreeMap<String, String>), String> {
    let path = user_config_file(config_dir);
    let raw = fs::read_to_string(&path)
        .map_err(|e| crate::i18n::t(&format!("Не удалось прочитать {}: {e}", path.display()), &format!("Failed to read {}: {e}", path.display())))?;
    parse_user_config_xml(&raw)
}

/// Parses a UserConfig snapshot from the profile file (root `<Preset>` or `<UserConfig>`).
pub fn parse_user_config_patch(
    raw: &str,
) -> Result<(BTreeMap<String, XmlNode>, BTreeMap<String, String>), String> {
    let repaired = repair_user_config_raw(raw);
    let doc =
        Document::parse(&repaired).map_err(|e| crate::i18n::t(&format!("Некорректный снимок UserConfig: {e}"), &format!("Invalid UserConfig snapshot: {e}")))?;
    let root = doc.root_element();
    let patch_root = match root.tag_name().name() {
        "Preset" | "UserConfig" => root,
        _ => root
            .children()
            .find(|n| n.is_element() && matches!(n.tag_name().name(), "Preset" | "UserConfig"))
            .ok_or_else(|| crate::i18n::t("В профиле нет корня <Preset> или <UserConfig>", "Profile has no <Preset> or <UserConfig> root"))?,
    };
    parse_settings_and_selections(patch_root)
}

fn parse_user_config_xml(
    raw: &str,
) -> Result<(BTreeMap<String, XmlNode>, BTreeMap<String, String>), String> {
    let repaired = repair_user_config_raw(raw);
    let doc = Document::parse(&repaired)
        .map_err(|e| crate::i18n::t(&format!("Некорректный UserConfigSelections: {e}"), &format!("Invalid UserConfigSelections: {e}")))?;
    let root = doc.root_element();
    if root.tag_name().name() != "UserConfig" {
        return Err(crate::i18n::t("Ожидался корневой элемент UserConfig", "Expected UserConfig root element"));
    }
    parse_settings_and_selections(root)
}

fn parse_settings_and_selections(
    root: roxmltree::Node<'_, '_>,
) -> Result<(BTreeMap<String, XmlNode>, BTreeMap<String, String>), String> {
    let mut settings = BTreeMap::new();
    let mut selections = BTreeMap::new();

    let settings_node = root
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "settings")
        .ok_or_else(|| crate::i18n::t("Нет секции <settings>", "Missing <settings> section"))?;
    for child in settings_node.children().filter(|n| n.is_element()) {
        let tag = child.tag_name().name().to_string();
        let mut attrs = BTreeMap::new();
        for attr in child.attributes() {
            attrs.insert(attr.name().to_string(), attr.value().to_string());
        }
        settings.insert(tag.clone(), XmlNode { tag, attrs });
    }

    if let Some(sel_node) = root
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "selections")
    {
        for child in sel_node
            .children()
            .filter(|n| n.is_element() && n.tag_name().name() == "option")
        {
            let id = child
                .attribute("id")
                .ok_or_else(|| crate::i18n::t("option без id", "option without id"))?
                .to_string();
            let value = child.attribute("value").unwrap_or("").to_string();
            selections.insert(id, value);
        }
    }

    Ok((settings, selections))
}

pub fn repair_user_config_raw(raw: &str) -> String {
    if raw.trim().is_empty() {
        return raw.to_string();
    }
    let mut t = raw.to_string();
    let re1 = Regex::new(r"/>\s*\r?\n\s*(option\s)").unwrap();
    t = re1.replace_all(&t, "/>\n<option ").to_string();
    let re2 = Regex::new(r"/>\s*\r?\n([A-Za-z][A-Za-z0-9]*)").unwrap();
    t = re2.replace_all(&t, "/>\n<$1").to_string();
    let re3 = Regex::new(r"</settings>\s*<selections>").unwrap();
    t = re3.replace_all(&t, "</settings>\n<selections>").to_string();
    t
}

pub fn merge_preset(
    target_settings: &mut BTreeMap<String, XmlNode>,
    target_selections: &mut BTreeMap<String, String>,
    preset_settings: &BTreeMap<String, XmlNode>,
    preset_selections: &BTreeMap<String, String>,
) {
    merge_preset_with_policy(
        target_settings,
        target_selections,
        preset_settings,
        preset_selections,
        None,
    );
}

pub fn merge_preset_with_policy(
    target_settings: &mut BTreeMap<String, XmlNode>,
    target_selections: &mut BTreeMap<String, String>,
    preset_settings: &BTreeMap<String, XmlNode>,
    preset_selections: &BTreeMap<String, String>,
    policy: Option<&crate::remote_presets::PackPolicy>,
) {
    use std::collections::HashSet;

    let preserve_settings: HashSet<&str> =
        if let Some(p) = policy.filter(|p| !p.preserve_settings.is_empty()) {
            p.preserve_settings.iter().map(String::as_str).collect()
        } else {
            PRESERVE_SETTINGS.iter().copied().collect()
        };
    let preserve_selections: HashSet<&str> =
        if let Some(p) = policy.filter(|p| !p.preserve_selections.is_empty()) {
            p.preserve_selections.iter().map(String::as_str).collect()
        } else {
            PRESERVE_OPTIONS.iter().copied().collect()
        };

    for (tag, node) in preset_settings {
        if preserve_settings.contains(tag.as_str()) {
            continue;
        }
        match target_settings.get_mut(tag) {
            Some(existing) => {
                for (k, v) in &node.attrs {
                    existing.attrs.insert(k.clone(), v.clone());
                }
                if !node.attrs.contains_key("isDynamic") {
                    existing.attrs.remove("isDynamic");
                }
            }
            None => {
                target_settings.insert(tag.clone(), node.clone());
            }
        }
    }

    for (id, value) in preset_selections {
        if preserve_selections.contains(id.as_str()) {
            continue;
        }
        target_selections.insert(id.clone(), value.clone());
    }
}

/// Safe GPU clamp: VPS values are not overwritten, only unsupported options are disabled.
pub fn tune_forza_selections(selections: &mut BTreeMap<String, String>, gpu: &GpuCapabilities) {
    if !gpu.supports_dlss {
        for key in [
            "DLSSMode",
            "DLSSGMode",
            "DLAA",
            "NVIDIATech",
            "XeSSMode",
            "XeSSAA",
        ] {
            if selections.contains_key(key) {
                selections.insert(key.into(), "0".into());
            }
        }
    }

    if !gpu.supports_dlss_fg && selections.contains_key("DLSSGMode") {
        selections.insert("DLSSGMode".into(), "0".into());
    }

    if !gpu.supports_ray_tracing {
        for key in ["RTReflectionQuality", "RTGIQuality"] {
            if selections.contains_key(key) {
                selections.insert(key.into(), "0".into());
            }
        }
    }
}

pub fn set_setting_value(settings: &mut BTreeMap<String, XmlNode>, tag: &str, value: &str) {
    let node = settings.entry(tag.to_string()).or_insert_with(|| XmlNode {
        tag: tag.to_string(),
        attrs: BTreeMap::new(),
    });
    node.attrs.insert("value".to_string(), value.to_string());
}

pub fn write_user_config(
    config_dir: &Path,
    settings: &BTreeMap<String, XmlNode>,
    selections: &BTreeMap<String, String>,
) -> Result<(), String> {
    let path = user_config_file(config_dir);
    crate::fs_util::clear_readonly(&path);
    let text = serialize_user_config(settings, selections);
    crate::fs_util::write_file_bytes(path.as_path(), text.as_bytes())?;
    let read_back = crate::fs_util::read_file_bytes(&path)?;
    if read_back != text.as_bytes() {
        return Err(crate::i18n::t(
            &format!(
                "UserConfigSelections не сохранился ({}). Закройте игру и повторите.",
                path.display()
            ),
            &format!(
                "UserConfigSelections was not saved ({}). Close the game and try again.",
                path.display()
            ),
        ));
    }
    Ok(())
}

fn serialize_user_config(
    settings: &BTreeMap<String, XmlNode>,
    selections: &BTreeMap<String, String>,
) -> String {
    let mut out = String::from("<UserConfig>\n<settings>\n");
    for node in settings.values() {
        out.push('<');
        out.push_str(&node.tag);
        for (k, v) in &node.attrs {
            out.push(' ');
            out.push_str(k);
            out.push_str("=\"");
            out.push_str(&escape_xml_attr(v));
            out.push('"');
        }
        out.push_str("/>\n");
    }
    out.push_str("</settings>\n<selections>\n");
    for (id, value) in selections {
        out.push_str("<option id=\"");
        out.push_str(&escape_xml_attr(id));
        out.push_str("\" value=\"");
        out.push_str(&escape_xml_attr(value));
        out.push_str("\"/>\n");
    }
    out.push_str("</selections>\n</UserConfig>\n");
    out
}

fn escape_xml_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

pub fn preview_forza_diff(
    config_dir: &Path,
    preset_settings: &BTreeMap<String, XmlNode>,
    preset_selections: &BTreeMap<String, String>,
) -> Result<Vec<ConfigDiffEntry>, String> {
    let (mut settings, mut selections) = read_user_config(config_dir)?;
    let before_settings = settings.clone();
    let before_selections = selections.clone();
    merge_preset(
        &mut settings,
        &mut selections,
        preset_settings,
        preset_selections,
    );

    let mut diff = Vec::new();

    for (tag, node) in preset_settings {
        if PRESERVE_SETTINGS.contains(&tag.as_str()) {
            continue;
        }
        let before = before_settings.get(tag);
        for (key, new_value) in &node.attrs {
            let old_value = before.and_then(|b| b.attrs.get(key)).cloned();
            if old_value.as_deref() == Some(new_value.as_str()) {
                continue;
            }
            diff.push(ConfigDiffEntry {
                file: USER_CONFIG_FILE.to_string(),
                section: format!("settings/{tag}"),
                key: key.clone(),
                old_value,
                new_value: new_value.clone(),
            });
        }
    }

    for (id, new_value) in preset_selections {
        if PRESERVE_OPTIONS.contains(&id.as_str()) {
            continue;
        }
        let old_value = before_selections.get(id).cloned();
        if old_value.as_deref() == Some(new_value.as_str()) {
            continue;
        }
        diff.push(ConfigDiffEntry {
            file: USER_CONFIG_FILE.to_string(),
            section: "selections".to_string(),
            key: id.clone(),
            old_value,
            new_value: (*new_value).clone(),
        });
    }

    Ok(diff)
}

fn media_file_differs(src: &Path, dest: &Path) -> Result<bool, String> {
    if !dest.is_file() {
        return Ok(true);
    }
    let src_bytes =
        fs::read(src).map_err(|e| crate::i18n::t(&format!("Не удалось прочитать {}: {e}", src.display()), &format!("Failed to read {}: {e}", src.display())))?;
    let dest_bytes =
        fs::read(dest).map_err(|e| crate::i18n::t(&format!("Не удалось прочитать {}: {e}", dest.display()), &format!("Failed to read {}: {e}", dest.display())))?;
    Ok(src_bytes != dest_bytes)
}

fn iter_preset_media_files(media_src: &Path) -> Result<Vec<(std::path::PathBuf, String)>, String> {
    if !media_src.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(media_src)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let rel = entry
            .path()
            .strip_prefix(media_src)
            .map_err(|e| e.to_string())?;
        let rel_s = rel.to_string_lossy().replace('\\', "/");
        if !crate::fs_util::is_safe_manifest_relative_path(&rel_s) {
            return Err(crate::i18n::t(
                &format!("Недопустимый путь media в preset: {rel_s}"),
                &format!("Invalid media path in preset: {rel_s}"),
            ));
        }
        files.push((rel.to_path_buf(), rel_s));
    }
    files.sort_by(|a, b| a.1.cmp(&b.1));
    Ok(files)
}

pub fn preview_media_diff(
    install_dir: &Path,
    media_src: &Path,
) -> Result<Vec<crate::models::ConfigDiffEntry>, String> {
    let media_dest_root = install_dir.join("media");
    let mut diff = Vec::new();

    for (rel, rel_s) in iter_preset_media_files(&media_src)? {
        let src = media_src.join(&rel);
        let dest = media_dest_root.join(&rel);
        crate::fs_util::ensure_path_within_root(&media_dest_root, &dest)
            .map_err(|_| crate::i18n::t(&format!("Недопустимый путь media/{rel_s}"), &format!("Invalid media path {rel_s}")))?;
        if !media_file_differs(&src, &dest)? {
            continue;
        }
        let new_size = src.metadata().map(|m| m.len()).unwrap_or(0);
        let old_value = if dest.is_file() {
            dest.metadata().ok().map(|m| format!("{} B", m.len()))
        } else {
            None
        };
        let new_value = if dest.is_file() {
            format!("{new_size} B")
        } else {
            crate::i18n::t(&format!("{new_size} B (новый)"), &format!("{new_size} B (new)"))
        };
        diff.push(crate::models::ConfigDiffEntry {
            file: format!("media/{rel_s}"),
            section: "media".to_string(),
            key: rel_s,
            old_value,
            new_value,
        });
    }

    Ok(diff)
}

pub fn backup_forza_media(
    install_dir: &Path,
    backup_path: &Path,
    media_src: &Path,
) -> Result<(), String> {
    if !media_src.is_dir() {
        return Ok(());
    }
    let media_dest_root = install_dir.join("media");
    for (rel, _) in iter_preset_media_files(media_src)? {
        let src_file = media_src.join(&rel);
        let dest_file = media_dest_root.join(&rel);
        if !media_file_differs(&src_file, &dest_file)? || !dest_file.is_file() {
            continue;
        }
        let backup_file = backup_path.join("media").join(&rel);
        if let Some(parent) = backup_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| crate::i18n::t(&format!("Не удалось создать backup media: {e}"), &format!("Failed to create media backup: {e}")))?;
        }
        fs::copy(&dest_file, &backup_file)
            .map_err(|e| crate::i18n::t(&format!("Не удалось сохранить backup media: {e}"), &format!("Failed to save media backup: {e}")))?;
    }
    Ok(())
}

pub fn copy_preset_media(install_dir: &Path, media_src: &Path) -> Result<Vec<String>, String> {
    let media_dest = install_dir.join("media");
    let mut changed = Vec::new();

    for (rel, rel_s) in iter_preset_media_files(&media_src)? {
        let src = media_src.join(&rel);
        let dest = media_dest.join(&rel);
        crate::fs_util::ensure_path_within_root(&media_dest, &dest)
            .map_err(|_| crate::i18n::t(&format!("Недопустимый путь media/{rel_s}"), &format!("Invalid media path {rel_s}")))?;
        if !media_file_differs(&src, &dest)? {
            continue;
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| crate::i18n::t(&format!("Не удалось создать {}: {e}", parent.display()), &format!("Failed to create {}: {e}", parent.display())))?;
        }
        let bytes = crate::fs_util::read_file_bytes(&src)
            .map_err(|e| crate::i18n::t(&format!("Не удалось прочитать media {}: {e}", rel.display()), &format!("Failed to read media {}: {e}", rel.display())))?;
        crate::fs_util::write_file_bytes(&dest, &bytes)
            .map_err(|e| crate::i18n::t(&format!("Не удалось скопировать media {}: {e}", rel.display()), &format!("Failed to copy media {}: {e}", rel.display())))?;
        changed.push(format!("media/{rel_s}"));
    }

    Ok(changed)
}

#[derive(Debug, Clone)]
pub struct MediaRollbackEntry {
    pub rel: String,
    pub dst: PathBuf,
    pub previous: Option<Vec<u8>>,
}

pub fn snapshot_media_for_rollback(
    install_dir: &Path,
    media_src: &Path,
) -> Result<Vec<MediaRollbackEntry>, String> {
    let media_dest = install_dir.join("media");
    let mut snapshot = Vec::new();
    for (rel, rel_s) in iter_preset_media_files(media_src)? {
        let dst = media_dest.join(&rel);
        crate::fs_util::ensure_path_within_root(&media_dest, &dst)
            .map_err(|_| crate::i18n::t(&format!("Недопустимый путь media/{rel_s}"), &format!("Invalid media path {rel_s}")))?;
        let previous = if dst.is_file() {
            Some(
                crate::fs_util::read_file_bytes(&dst)
                    .map_err(|e| crate::i18n::t(&format!("Не удалось прочитать media/{rel_s}: {e}"), &format!("Failed to read media/{rel_s}: {e}")))?,
            )
        } else {
            None
        };
        snapshot.push(MediaRollbackEntry {
            rel: rel_s,
            dst,
            previous,
        });
    }
    Ok(snapshot)
}

pub fn rollback_media_from_snapshot(snapshot: &[MediaRollbackEntry]) -> Result<(), String> {
    for entry in snapshot {
        if let Some(bytes) = &entry.previous {
            if let Some(parent) = entry.dst.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    crate::i18n::t(
                        &format!(
                            "Не удалось создать каталог для rollback media/{}: {e}",
                            entry.rel
                        ),
                        &format!(
                            "Failed to create directory for rollback media/{}: {e}",
                            entry.rel
                        ),
                    )
                })?;
            }
            crate::fs_util::write_file_bytes(&entry.dst, bytes).map_err(|e| {
                crate::i18n::t(
                    &format!(
                        "Не удалось откатить media/{} к предыдущей версии: {e}",
                        entry.rel
                    ),
                    &format!(
                        "Failed to roll back media/{} to previous version: {e}",
                        entry.rel
                    ),
                )
            })?;
        } else if entry.dst.exists() {
            crate::fs_util::clear_readonly(&entry.dst);
            fs::remove_file(&entry.dst)
                .map_err(|e| crate::i18n::t(&format!("Не удалось удалить media/{} при rollback: {e}", entry.rel), &format!("Failed to delete media/{} during rollback: {e}", entry.rel)))?;
        }
    }
    Ok(())
}

pub fn restore_forza_media(
    install_dir: &Path,
    backup_path: &Path,
) -> Result<Vec<String>, String> {
    use crate::fs_util::{
        ensure_path_within_root, is_safe_manifest_relative_path, read_file_bytes, write_file_bytes,
    };

    let install_dir = crate::forza::validate_forza_install_dir(install_dir)?;

    let media_backup = backup_path.join("media");
    if !media_backup.is_dir() {
        return Ok(Vec::new());
    }

    let install_media = install_dir.join("media");
    let mut restored = Vec::new();

    for entry in walkdir::WalkDir::new(&media_backup)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(&media_backup)
            .map_err(|e| e.to_string())?;
        let rel_s = rel.to_string_lossy().replace('\\', "/");
        if !is_safe_manifest_relative_path(&rel_s) {
            return Err(crate::i18n::t(
                &format!("Недопустимый путь media в backup: {rel_s}"),
                &format!("Invalid media path in backup: {rel_s}"),
            ));
        }
        let dst = install_media.join(rel);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| crate::i18n::t(&format!("Не удалось создать media/{rel_s}: {e}"), &format!("Failed to create media/{rel_s}: {e}")))?;
        }
        ensure_path_within_root(&install_media, &dst)
            .map_err(|_| crate::i18n::t(&format!("Недопустимый путь восстановления media: {rel_s}"), &format!("Invalid media restore path: {rel_s}")))?;
        let bytes = read_file_bytes(entry.path())
            .map_err(|e| crate::i18n::t(&format!("Не удалось прочитать backup media {rel_s}: {e}"), &format!("Failed to read backup media {rel_s}: {e}")))?;
        write_file_bytes(&dst, &bytes)
            .map_err(|e| crate::i18n::t(&format!("Не удалось восстановить media {rel_s}: {e}"), &format!("Failed to restore media {rel_s}: {e}")))?;
        restored.push(format!("media/{rel_s}"));
    }

    Ok(restored)
}

pub fn backup_forza_config(config_dir: &Path) -> Result<String, String> {
    crate::backup::backup_forza_config_dir(config_dir, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_media_skips_identical_files() {
        let install = tempfile::tempdir().unwrap();
        let media_src = tempfile::tempdir().unwrap();
        let rel = Path::new("Tracks").join("test.xml");
        let src = media_src.path().join(&rel);
        let dest = install.path().join("media").join(&rel);
        fs::create_dir_all(src.parent().unwrap()).unwrap();
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::write(&src, b"<same/>").unwrap();
        fs::write(&dest, b"<same/>").unwrap();

        let diff = preview_media_diff(install.path(), media_src.path()).unwrap();
        assert!(diff.is_empty());

        fs::write(&src, b"<preset/>").unwrap();
        let diff = preview_media_diff(install.path(), media_src.path()).unwrap();
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].file, "media/Tracks/test.xml");
    }

    #[test]
    fn restore_forza_media_roundtrip() {
        let install = tempfile::tempdir().unwrap();
        std::fs::write(install.path().join("forzahorizon6.exe"), b"").unwrap();
        let backup = tempfile::tempdir().unwrap();
        let rel = Path::new("Tracks").join("test.xml");
        let backup_file = backup.path().join("media").join(&rel);
        fs::create_dir_all(backup_file.parent().unwrap()).unwrap();
        fs::write(&backup_file, b"<original/>").unwrap();
        let dest = install.path().join("media").join(&rel);
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::write(&dest, b"<changed/>").unwrap();

        let restored = restore_forza_media(install.path(), backup.path()).unwrap();
        assert!(restored.iter().any(|f| f.contains("Tracks/test.xml")));
        assert_eq!(
            fs::read_to_string(install.path().join("media").join(&rel)).unwrap(),
            "<original/>"
        );
    }

    #[test]
    fn copy_media_skips_identical_files() {
        let install = tempfile::tempdir().unwrap();
        let media_src = tempfile::tempdir().unwrap();
        let rel = Path::new("gs").join("routebudget.xml");
        let src = media_src.path().join(&rel);
        let dest = install.path().join("media").join(&rel);
        fs::create_dir_all(src.parent().unwrap()).unwrap();
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::write(&src, b"same").unwrap();
        fs::write(&dest, b"same").unwrap();

        let changed = copy_preset_media(install.path(), media_src.path()).unwrap();
        assert!(changed.is_empty());
    }

    #[test]
    fn media_snapshot_rolls_back_new_and_existing_files() {
        let install = tempfile::tempdir().unwrap();
        let media_src = tempfile::tempdir().unwrap();
        let rel_existing = Path::new("Tracks").join("existing.xml");
        let rel_new = Path::new("Tracks").join("new.xml");

        let src_existing = media_src.path().join(&rel_existing);
        let src_new = media_src.path().join(&rel_new);
        fs::create_dir_all(src_existing.parent().unwrap()).unwrap();
        fs::write(&src_existing, b"<preset-existing/>").unwrap();
        fs::write(&src_new, b"<preset-new/>").unwrap();

        let dst_existing = install.path().join("media").join(&rel_existing);
        fs::create_dir_all(dst_existing.parent().unwrap()).unwrap();
        fs::write(&dst_existing, b"<old/>").unwrap();

        let snapshot = snapshot_media_for_rollback(install.path(), media_src.path()).unwrap();
        let changed = copy_preset_media(install.path(), media_src.path()).unwrap();
        assert_eq!(changed.len(), 2);

        rollback_media_from_snapshot(&snapshot).unwrap();
        assert_eq!(fs::read_to_string(&dst_existing).unwrap(), "<old/>");
        assert!(!install.path().join("media").join(&rel_new).exists());
    }
}
