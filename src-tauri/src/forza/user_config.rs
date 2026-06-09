use crate::forza::detect::user_config_file;
use crate::gpu::GpuCapabilities;
use crate::models::ConfigDiffEntry;
use regex::Regex;
use roxmltree::Document;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

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

pub fn read_user_config(config_dir: &Path) -> Result<(BTreeMap<String, XmlNode>, BTreeMap<String, String>), String> {
    let path = user_config_file(config_dir);
    let raw = fs::read_to_string(&path)
        .map_err(|e| format!("Не удалось прочитать {}: {e}", path.display()))?;
    parse_user_config_xml(&raw)
}

/// Парсит снимок UserConfig из файла профиля (корень `<Preset>` или `<UserConfig>`).
pub fn parse_user_config_patch(raw: &str) -> Result<(BTreeMap<String, XmlNode>, BTreeMap<String, String>), String> {
    let repaired = repair_user_config_raw(raw);
    let doc = Document::parse(&repaired)
        .map_err(|e| format!("Некорректный снимок UserConfig: {e}"))?;
    let root = doc.root_element();
    let patch_root = match root.tag_name().name() {
        "Preset" | "UserConfig" => root,
        _ => root
            .children()
            .find(|n| {
                n.is_element()
                    && matches!(n.tag_name().name(), "Preset" | "UserConfig")
            })
            .ok_or_else(|| {
                "В профиле нет корня <Preset> или <UserConfig>".to_string()
            })?,
    };
    parse_settings_and_selections(patch_root)
}

fn parse_user_config_xml(raw: &str) -> Result<(BTreeMap<String, XmlNode>, BTreeMap<String, String>), String> {
    let repaired = repair_user_config_raw(raw);
    let doc = Document::parse(&repaired)
        .map_err(|e| format!("Некорректный UserConfigSelections: {e}"))?;
    let root = doc.root_element();
    if root.tag_name().name() != "UserConfig" {
        return Err("Ожидался корневой элемент UserConfig".to_string());
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
        .ok_or_else(|| "Нет секции <settings>".to_string())?;
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
        for child in sel_node.children().filter(|n| n.is_element() && n.tag_name().name() == "option") {
            let id = child
                .attribute("id")
                .ok_or_else(|| "option без id".to_string())?
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

/// Безопасный clamp по GPU: значения из VPS не переписываются, только отключается неподдерживаемое.
pub fn tune_forza_selections(
    selections: &mut BTreeMap<String, String>,
    gpu: &GpuCapabilities,
) {
    if !gpu.supports_dlss {
        for key in ["DLSSMode", "DLSSGMode", "DLAA", "NVIDIATech", "XeSSMode", "XeSSAA"] {
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

pub fn set_setting_value(
    settings: &mut BTreeMap<String, XmlNode>,
    tag: &str,
    value: &str,
) {
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
    let text = serialize_user_config(settings, selections);
    fs::write(&path, text).map_err(|e| format!("Не удалось записать {}: {e}", path.display()))
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
    let src_bytes = fs::read(src)
        .map_err(|e| format!("Не удалось прочитать {}: {e}", src.display()))?;
    let dest_bytes = fs::read(dest)
        .map_err(|e| format!("Не удалось прочитать {}: {e}", dest.display()))?;
    Ok(src_bytes != dest_bytes)
}

fn iter_preset_media_files(
    media_src: &Path,
) -> Result<Vec<(std::path::PathBuf, String)>, String> {
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
        if !media_file_differs(&src, &dest)? {
            continue;
        }
        let new_size = src.metadata().map(|m| m.len()).unwrap_or(0);
        let old_value = if dest.is_file() {
            dest.metadata()
                .ok()
                .map(|m| format!("{} B", m.len()))
        } else {
            None
        };
        let new_value = if dest.is_file() {
            format!("{new_size} B")
        } else {
            format!("{new_size} B (новый)")
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
                .map_err(|e| format!("Не удалось создать backup media: {e}"))?;
        }
        fs::copy(&dest_file, &backup_file)
            .map_err(|e| format!("Не удалось сохранить backup media: {e}"))?;
    }
    Ok(())
}

pub fn copy_preset_media(install_dir: &Path, media_src: &Path) -> Result<Vec<String>, String> {
    let media_dest = install_dir.join("media");
    let mut changed = Vec::new();

    for (rel, rel_s) in iter_preset_media_files(&media_src)? {
        let src = media_src.join(&rel);
        let dest = media_dest.join(&rel);
        if !media_file_differs(&src, &dest)? {
            continue;
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Не удалось создать {}: {e}", parent.display()))?;
        }
        fs::copy(&src, &dest)
            .map_err(|e| format!("Не удалось скопировать media {}: {e}", rel.display()))?;
        changed.push(format!("media/{rel_s}"));
    }

    Ok(changed)
}

pub fn backup_forza_config(config_dir: &Path) -> Result<String, String> {
    crate::backup::backup_forza_config_dir(config_dir)
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
}
