use crate::ini::{parser::ini_to_data, read_ini_file};
use crate::models::GameParameter;
use crate::scalability::{detect_scalability_limits, is_scalability_quality_index};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct ParameterCatalogEntry {
    pub key: String,
    pub category: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub impact: String,
    #[serde(default)]
    pub min: Option<String>,
    #[serde(default)]
    pub max: Option<String>,
    #[serde(default)]
    pub value_hint: Option<String>,
    #[serde(default)]
    pub in_game_label: Option<String>,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default = "default_value_type")]
    pub value_type: String,
    #[serde(default = "default_editable")]
    pub editable: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct KeyHintEntry {
    pub key: String,
    pub category: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub impact: String,
    #[serde(default)]
    pub min: Option<String>,
    #[serde(default)]
    pub max: Option<String>,
    #[serde(default)]
    pub value_hint: Option<String>,
    #[serde(default = "default_value_type")]
    pub value_type: String,
    #[serde(default = "default_editable")]
    pub editable: bool,
}

fn default_value_type() -> String {
    "string".to_string()
}

fn default_editable() -> bool {
    true
}

struct CatalogIndex {
    by_full_id: HashMap<String, ParameterCatalogEntry>,
    by_file_key: HashMap<String, ParameterCatalogEntry>,
    by_key: HashMap<String, ParameterCatalogEntry>,
    key_hints: HashMap<String, KeyHintEntry>,
}

pub fn load_parameter_catalog() -> Vec<ParameterCatalogEntry> {
    load_parameter_catalog_for_family(None)
}

pub fn load_parameter_catalog_for_family(engine_family: Option<&str>) -> Vec<ParameterCatalogEntry> {
    let is_ue4 = engine_family == Some("ue4");
    let mut entries = load_remote_parameter_catalog(is_ue4);
    entries.extend(load_bundled_parameter_catalog(is_ue4));
    entries
}

fn filter_catalog_entries(
    entries: Vec<ParameterCatalogEntry>,
    is_ue4: bool,
) -> Vec<ParameterCatalogEntry> {
    if !is_ue4 {
        return entries;
    }
    entries
        .into_iter()
        .filter(|entry| !is_ue5_only_catalog_key(&entry.key))
        .collect()
}

fn should_load_catalog_file(name: &str, is_ue4: bool) -> bool {
    if name == "parameters.json" || name == "key_hints.json" {
        return false;
    }
    if is_ue4 {
        return matches!(name, "ue4.json" | "display.json" | "subnautica2.json");
    }
    name != "ue4.json"
}

fn load_remote_parameter_catalog(is_ue4: bool) -> Vec<ParameterCatalogEntry> {
    let mut entries = Vec::new();
    for pack in crate::remote_presets::find_catalog_packs() {
        if let Some(files) = pack.load_catalog_json_files() {
            for path in files {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !should_load_catalog_file(name, is_ue4) {
                    continue;
                }
                entries.extend(filter_catalog_entries(parse_catalog_file(&path), is_ue4));
            }
        }
    }
    entries
}

fn load_bundled_parameter_catalog(is_ue4: bool) -> Vec<ParameterCatalogEntry> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("catalog");
    let mut entries = Vec::new();

    let legacy = dir.join("parameters.json");
    if legacy.exists() {
        entries.extend(filter_catalog_entries(parse_catalog_file(&legacy), is_ue4));
    }

    if let Ok(read_dir) = fs::read_dir(&dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !should_load_catalog_file(name, is_ue4) {
                continue;
            }
            entries.extend(filter_catalog_entries(parse_catalog_file(&path), is_ue4));
        }
    }

    entries
}

fn load_author_catalog(
    game_id: Option<&str>,
    ini_has_subnautica: bool,
) -> Vec<ParameterCatalogEntry> {
    if game_id != Some("steam-1962700") && !ini_has_subnautica {
        return Vec::new();
    }
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("catalog")
        .join("subnautica2.json");
    parse_catalog_file(&path)
}

fn config_dir_has_subnautica_sections(config_dir: &Path) -> bool {
    let gus = config_dir.join("GameUserSettings.ini");
    if !gus.exists() {
        return false;
    }
    let Ok(ini) = read_ini_file(&gus) else {
        return false;
    };
    ini_to_data(&ini)
        .keys()
        .any(|section| section.to_lowercase().contains("subnautica"))
}

pub fn parse_catalog_file(path: &Path) -> Vec<ParameterCatalogEntry> {
    let content = fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string());
    serde_json::from_str(&content).unwrap_or_default()
}

fn load_key_hints() -> HashMap<String, KeyHintEntry> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("catalog")
        .join("key_hints.json");
    let content = fs::read_to_string(&path).unwrap_or_else(|_| "[]".to_string());
    let hints: Vec<KeyHintEntry> = serde_json::from_str(&content).unwrap_or_default();
    hints.into_iter().map(|h| (h.key.to_lowercase(), h)).collect()
}

fn build_catalog_index(catalog: Vec<ParameterCatalogEntry>) -> CatalogIndex {
    let mut by_full_id = HashMap::new();
    let mut by_file_key = HashMap::new();
    let mut by_key = HashMap::new();

    for entry in catalog {
        if let (Some(file), Some(section)) = (&entry.file, &entry.section) {
            let full_id = catalog_id(file, section, &entry.key);
            by_full_id.insert(full_id, entry.clone());
            let file_key = format!("{}::{}", file.to_lowercase(), entry.key.to_lowercase());
            by_file_key.entry(file_key).or_insert(entry.clone());
        }
        by_key
            .entry(entry.key.to_lowercase())
            .or_insert(entry);
    }

    CatalogIndex {
        by_full_id,
        by_file_key,
        by_key,
        key_hints: load_key_hints(),
    }
}

fn catalog_id(file: &str, section: &str, key: &str) -> String {
    format!(
        "{}::{}::{}",
        file.to_lowercase(),
        section.to_lowercase(),
        key.to_lowercase()
    )
}

fn lookup_entry<'a>(
    index: &'a CatalogIndex,
    file: &str,
    section: &str,
    key: &str,
) -> Option<CatalogMatch<'a>> {
    let full_id = catalog_id(file, section, key);
    if let Some(entry) = index.by_full_id.get(&full_id) {
        return Some(CatalogMatch::Entry(entry));
    }

    let file_key = format!("{}::{}", file.to_lowercase(), key.to_lowercase());
    if let Some(entry) = index.by_file_key.get(&file_key) {
        return Some(CatalogMatch::Entry(entry));
    }

    if let Some(entry) = index.by_key.get(&key.to_lowercase()) {
        return Some(CatalogMatch::Entry(entry));
    }

    if let Some(hint) = index.key_hints.get(&key.to_lowercase()) {
        return Some(CatalogMatch::Hint(hint));
    }

    None
}

enum CatalogMatch<'a> {
    Entry(&'a ParameterCatalogEntry),
    Hint(&'a KeyHintEntry),
}

const UE5_ONLY_SG_KEYS: &[&str] = &[
    "sg.GlobalIlluminationQuality",
    "sg.ReflectionQuality",
    "sg.ShadingQuality",
    "sg.LandscapeQuality",
    "sg.CloudsQuality",
];

const UE5_ONLY_CVAR_KEYS: &[&str] = &[
    "r.Nanite",
    "r.Lumen.DiffuseIndirect.Allow",
    "r.Lumen.Reflections.Allow",
    "r.Lumen.Reflections.Quality",
    "r.Lumen.ScreenProbeGather.ScreenTraces",
    "r.VolumetricCloud",
];

pub fn get_game_parameters(
    config_dir: &Path,
    game_id: Option<&str>,
    install_dir: Option<&Path>,
    engine_family: Option<&str>,
) -> Result<Vec<GameParameter>, String> {
    if engine_family == Some("unity") {
        return get_unity_parameters(config_dir);
    }

    if engine_family == Some("forza") || crate::forza::is_forza_config_dir(config_dir) {
        return get_forza_parameters(config_dir, install_dir, game_id);
    }

    let is_ue4 = engine_family == Some("ue4");
    let ini_has_subnautica = config_dir_has_subnautica_sections(config_dir);
    let mut catalog = load_parameter_catalog_for_family(engine_family);
    catalog.extend(load_author_catalog(game_id, ini_has_subnautica));
    let index = build_catalog_index(catalog);

    let ini_files = ["GameUserSettings.ini", "Engine.ini", "Game.ini", "Scalability.ini"];
    let mut parameters = Vec::new();
    let mut seen = HashMap::new();
    for file in ini_files {
        let file_path = config_dir.join(file);
        if !file_path.exists() {
            continue;
        }
        let ini = read_ini_file(&file_path)?;
        let data = ini_to_data(&ini);
        for (section, entries) in data {
            for (key, value) in entries {
                let id = catalog_id(file, &section, &key);
                seen.insert(id.clone(), true);
                parameters.push(match lookup_entry(&index, file, &section, &key) {
                    Some(CatalogMatch::Entry(entry)) => {
                        entry_to_parameter(entry, &key, &section, file, &value, true, true)
                    }
                    Some(CatalogMatch::Hint(hint)) => {
                        hint_to_parameter(hint, &key, &section, file, &value)
                    }
                    None => unknown_parameter(&key, &section, file, &value),
                });
            }
        }
    }

    for (full_id, entry) in &index.by_full_id {
        if seen.contains_key(full_id) {
            continue;
        }
        if !should_include_catalog_entry(entry, game_id, ini_has_subnautica, is_ue4) {
            continue;
        }
        let file = entry.file.as_deref().unwrap_or("GameUserSettings.ini");
        if file != "Engine.ini" {
            continue;
        }
        let section = entry.section.as_deref().unwrap_or("");
        let default_value = catalog_default_value(entry);
        parameters.push(entry_to_parameter(
            entry,
            &entry.key,
            section,
            file,
            &default_value,
            true,
            false,
        ));
    }

    parameters.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then(a.file.cmp(&b.file))
            .then(a.key.cmp(&b.key))
    });

    let limits = detect_scalability_limits(install_dir, Some(config_dir));
    for param in &mut parameters {
        if is_scalability_quality_index(&param.key) {
            let max = limits.max_for(&param.key);
            if param.min.is_none() {
                param.min = Some("0".to_string());
            }
            param.max = Some(max.to_string());
            if param.value_hint.is_none() {
                param.value_hint = Some(format!("0 Low → {max} max (Cinematic+)"));
            }
        } else {
            apply_known_range_patterns(param);
            if param.min.is_none() && param.max.is_none() {
                infer_range_from_value(param);
            }
            fill_generic_value_hint(param);
        }
    }

    Ok(parameters)
}

fn entry_to_parameter(
    entry: &ParameterCatalogEntry,
    key: &str,
    section: &str,
    file: &str,
    value: &str,
    known: bool,
    present_in_ini: bool,
) -> GameParameter {
    let default_value = if !present_in_ini && (file == "Engine.ini" || file == "boot.config") {
        Some(catalog_default_value(entry))
    } else {
        None
    };
    GameParameter {
        key: key.to_string(),
        section: section.to_string(),
        file: file.to_string(),
        value: value.to_string(),
        title: entry.title.clone(),
        description: entry.description.clone(),
        impact: entry.impact.clone(),
        category: entry.category.clone(),
        min: entry.min.clone(),
        max: entry.max.clone(),
        value_hint: entry.value_hint.clone(),
        in_game_label: entry.in_game_label.clone(),
        value_type: entry.value_type.clone(),
        editable: entry.editable,
        known,
        present_in_ini,
        default_value,
    }
}

fn catalog_default_value(entry: &ParameterCatalogEntry) -> String {
    if let Some(hint) = &entry.value_hint {
        if let Some(num) = extract_hint_number(hint) {
            return num;
        }
    }
    match entry.value_type.as_str() {
        "bool" => "True".to_string(),
        "int" => {
            if let Some(max) = &entry.max {
                if let Ok(m) = max.parse::<i64>() {
                    if m <= 5 {
                        return max.clone();
                    }
                }
            }
            entry.min.clone().unwrap_or_else(|| "1".to_string())
        }
        "float" => "1.0".to_string(),
        _ => String::new(),
    }
}

fn extract_hint_number(hint: &str) -> Option<String> {
    let token = hint.split(|c: char| c == ',' || c == '—' || c == '-' || c == ' ')
        .find_map(|part| {
            let t = part.trim();
            if t.parse::<f64>().is_ok() || t.parse::<i64>().is_ok() {
                Some(t.to_string())
            } else {
                None
            }
        })?;
    Some(token)
}

fn hint_to_parameter(
    hint: &KeyHintEntry,
    key: &str,
    section: &str,
    file: &str,
    value: &str,
) -> GameParameter {
    GameParameter {
        key: key.to_string(),
        section: section.to_string(),
        file: file.to_string(),
        value: value.to_string(),
        title: hint.title.clone(),
        description: hint.description.clone(),
        impact: hint.impact.clone(),
        category: hint.category.clone(),
        min: hint.min.clone(),
        max: hint.max.clone(),
        value_hint: hint.value_hint.clone(),
        in_game_label: None,
        value_type: hint.value_type.clone(),
        editable: hint.editable,
        known: true,
        present_in_ini: true,
        default_value: None,
    }
}

fn is_opaque_struct_value(value: &str) -> bool {
    let v = value.trim();
    if v.len() > 200 {
        return true;
    }
    if v.starts_with('(') {
        return true;
    }
    let lower = v.to_ascii_lowercase();
    [
        "actionkeylist=",
        "axiskeylist=",
        "sensitivemap=",
        "gamepadkeylist=",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn truncate_preview(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let truncated: String = value.chars().take(max_chars).collect();
    format!("{truncated}…")
}

fn unknown_parameter(key: &str, section: &str, file: &str, value: &str) -> GameParameter {
    if is_opaque_struct_value(value) {
        return GameParameter {
            key: key.to_string(),
            section: section.to_string(),
            file: file.to_string(),
            value: value.to_string(),
            title: key.to_string(),
            description: format!(
                "Сложная структура из {file} (секция [{section}]). Редактирование в приложении недоступно."
            ),
            impact: "Ключ хранит вложенные данные игры (привязки клавиш, профили и т.п.). Меняйте в меню игры или вручную в ini.".to_string(),
            category: infer_category(section, key),
            min: None,
            max: None,
            value_hint: Some(format!(
                "Текущее значение ({})",
                truncate_preview(value, 80)
            )),
            in_game_label: None,
            value_type: "opaque".to_string(),
            editable: false,
            known: false,
            present_in_ini: true,
            default_value: None,
        };
    }

    let value_note = if value.chars().count() > 120 {
        format!(" Текущее: «{}».", truncate_preview(value, 120))
    } else {
        format!(" Текущее значение: «{value}».")
    };

    let mut param = GameParameter {
        key: key.to_string(),
        section: section.to_string(),
        file: file.to_string(),
        value: value.to_string(),
        title: humanize_cvar_key(key),
        description: format!(
            "Параметр «{key}» из {file} (секция [{section}]). В справочнике нет отдельной статьи — ниже подобран ориентировочный диапазон по типу ключа.{value_note}"
        ),
        impact: "Меняйте осторожно: эффект зависит от игры, возможен сброс при обновлении или смене пресета в меню.".to_string(),
        category: infer_category(section, key),
        min: None,
        max: None,
        value_hint: None,
        in_game_label: None,
        value_type: infer_value_type(value),
        editable: true,
        known: false,
        present_in_ini: true,
        default_value: None,
    };
    apply_known_range_patterns(&mut param);
    if param.min.is_none() && param.max.is_none() {
        infer_range_from_value(&mut param);
    }
    fill_generic_value_hint(&mut param);
    param
}

fn humanize_cvar_key(key: &str) -> String {
    let stripped = key
        .strip_prefix("r.")
        .or_else(|| key.strip_prefix("sg."))
        .or_else(|| key.strip_prefix("fx."))
        .unwrap_or(key);
    stripped
        .split('.')
        .map(|part| {
            let lower = part.to_lowercase();
            match lower.as_str() {
                "max" => "макс.".to_string(),
                "min" => "мин.".to_string(),
                "quality" => "качество".to_string(),
                "scale" => "масштаб".to_string(),
                "distance" => "дальность".to_string(),
                "shadow" => "тени".to_string(),
                "streaming" => "стриминг".to_string(),
                other => {
                    let mut chars = other.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

fn apply_known_range_patterns(param: &mut GameParameter) {
    if param.min.is_some() && param.max.is_some() {
        return;
    }
    if param.value_type == "bool" || param.value_type == "enum" || param.value_type == "string" {
        return;
    }

    let key = param.key.to_lowercase();

    if key == "sg.resolutionquality" {
        param.min = Some("25".to_string());
        param.max = Some("200".to_string());
        return;
    }

    if key.starts_with("sg.") && key.ends_with("quality") {
        param.min = Some("0".to_string());
        param.max = Some("4".to_string());
        return;
    }

    if key.contains("poolsize") {
        param.min = Some("128".to_string());
        param.max = Some("32768".to_string());
        return;
    }

    if key.contains("anisotropy") {
        param.min = Some("0".to_string());
        param.max = Some("16".to_string());
        return;
    }

    if key.contains("mipbias") {
        param.min = Some("-3".to_string());
        param.max = Some("15".to_string());
        return;
    }

    if key.contains("framerate") || key.contains("framelimit") || key.ends_with("fps") {
        param.min = Some("0".to_string());
        param.max = Some("360".to_string());
        return;
    }

    if key.contains("fov") || key.contains("fieldofview") {
        param.min = Some("70".to_string());
        param.max = Some("120".to_string());
        return;
    }

    if key.contains("gamma") {
        param.min = Some("1.0".to_string());
        param.max = Some("3.0".to_string());
        return;
    }

    if key.contains("resolutionscale") || key.contains("renderscale") {
        param.min = Some("0.25".to_string());
        param.max = Some("2.0".to_string());
        return;
    }

    if key.contains("resolution") && key.contains("size") {
        if key.ends_with('x') || key.contains("width") || key.contains("sizex") {
            param.min = Some("640".to_string());
            param.max = Some("7680".to_string());
        } else {
            param.min = Some("480".to_string());
            param.max = Some("4320".to_string());
        }
        return;
    }

    if key.contains("shadow") && key.contains("resolution") {
        param.min = Some("256".to_string());
        param.max = Some("8192".to_string());
        return;
    }

    if key.ends_with("scale") || key.contains(".scale") {
        param.min = Some("0.1".to_string());
        param.max = Some("4.0".to_string());
        return;
    }

    if key.ends_with("quality") || key.contains(".quality") {
        param.min = Some("0".to_string());
        param.max = Some(if key.contains("postprocess") || key.contains("aa") {
            "6".to_string()
        } else {
            "5".to_string()
        });
        return;
    }

    if key.starts_with("r.") {
        param.min = Some("0".to_string());
        param.max = Some("4".to_string());
    }
}

fn fill_generic_value_hint(param: &mut GameParameter) {
    if param.value_hint.is_some() {
        return;
    }
    if param.value_type == "bool" {
        param.value_hint = Some("True — вкл, False — выкл".to_string());
        return;
    }
    if param.value_type == "enum" {
        param.value_hint = Some("On — вкл, Off — выкл".to_string());
        return;
    }
    if let (Some(min), Some(max)) = (&param.min, &param.max) {
        param.value_hint = Some(format!("Допустимо: {min} – {max}"));
    }
}

fn infer_category(section: &str, key: &str) -> String {
    let lower = section.to_lowercase();
    if lower.starts_with("/script/") && !lower.contains("engine.gameusersettings") {
        if lower.contains("subnautica") {
            return "AuthorCurated".to_string();
        }
        return "GameSpecific".to_string();
    }
    if key.starts_with("sg.") {
        if key == "sg.ResolutionQuality" {
            return "Scalability".to_string();
        }
        if is_scalability_quality_index(key) {
            return "Scalability".to_string();
        }
        return "Scalability".to_string();
    }
    if key.starts_with("r.") {
        if key.to_lowercase().contains("shadow") {
            return "Shadows".to_string();
        }
        if key.to_lowercase().contains("stream") || key.to_lowercase().contains("anisotropy") {
            return "Textures".to_string();
        }
        if key.to_lowercase().contains("bloom")
            || key.to_lowercase().contains("motion")
            || key.to_lowercase().contains("ssr")
            || key.to_lowercase().contains("post")
            || key.to_lowercase().contains("dof")
            || key.to_lowercase().contains("tonemapper")
            || key.to_lowercase().contains("ambient")
        {
            return "PostProcess".to_string();
        }
        return "Rendering".to_string();
    }
    if key.to_lowercase().contains("audio") {
        return "Audio".to_string();
    }
    "Other".to_string()
}

fn infer_range_from_value(param: &mut GameParameter) {
    if param.key == "sg.ResolutionQuality" {
        param.min = Some("25".to_string());
        param.max = Some("200".to_string());
        param.value_hint = Some("Процент render scale, не индекс 0–4".to_string());
        return;
    }

    if param.value.trim() == "-1" || param.value.trim() == "-1.0" {
        param.value_hint = Some(
            "−1 — автоматически (движок/меню сами выбирают значение). Задайте число вручную, чтобы зафиксировать."
                .to_string(),
        );
        return;
    }

    let Ok(num) = param.value.parse::<f64>() else {
        return;
    };

    if num.fract() != 0.0 {
        let pad = num.abs().max(0.5);
        param.min = Some(format!("{:.4}", (num - pad).max(0.0)));
        param.max = Some(format!("{:.4}", num + pad));
        return;
    }

    let n = num as i64;
    if n <= 4 && !param.key.starts_with("r.") {
        return;
    }

    param.min = Some("0".to_string());
    param.max = Some(n.saturating_mul(2).max(100).to_string());
}

fn should_include_catalog_entry(
    entry: &ParameterCatalogEntry,
    game_id: Option<&str>,
    ini_has_subnautica: bool,
    is_ue4: bool,
) -> bool {
    if entry.category == "AuthorCurated" {
        let section = entry.section.as_deref().unwrap_or("").to_lowercase();
        return section.contains("subnautica")
            && (game_id == Some("steam-1962700") || ini_has_subnautica);
    }
    if is_ue4 && is_ue5_only_catalog_key(&entry.key) {
        return false;
    }
    true
}

fn is_ue5_only_catalog_key(key: &str) -> bool {
    UE5_ONLY_SG_KEYS.contains(&key) || UE5_ONLY_CVAR_KEYS.contains(&key)
}

fn get_unity_parameters(config_dir: &Path) -> Result<Vec<GameParameter>, String> {
    let catalog_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("catalog").join("unity.json");
    let entries = parse_catalog_file(&catalog_path);
    let boot_path = crate::unity::boot_config_path(config_dir);
    let boot_map = if boot_path.exists() {
        let content = std::fs::read_to_string(&boot_path)
            .map_err(|e| format!("Не удалось прочитать boot.config: {e}"))?;
        crate::unity::parse_boot_config(&content)
    } else {
        HashMap::new()
    };

    let mut parameters = Vec::new();
    for entry in entries {
        let present = boot_map.contains_key(&entry.key);
        let value = boot_map
            .get(&entry.key)
            .cloned()
            .unwrap_or_else(|| {
                if present {
                    String::new()
                } else {
                    catalog_default_value(&entry)
                }
            });
        parameters.push(entry_to_parameter(
            &entry,
            &entry.key,
            "",
            entry.file.as_deref().unwrap_or("boot.config"),
            &value,
            true,
            present,
        ));
    }

    for (key, value) in &boot_map {
        if parameters.iter().any(|p| p.key == *key) {
            continue;
        }
        parameters.push(unknown_parameter(key, "", "boot.config", value));
    }

    parameters.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then_with(|| a.key.cmp(&b.key))
    });
    Ok(parameters)
}

pub fn load_forza_parameter_catalog(game_id: Option<&str>) -> Option<Vec<ParameterCatalogEntry>> {
    crate::remote_presets::ensure_synced();
    let pack = crate::remote_presets::find_pack(game_id, Some("forza"), None)?;
    let path = pack.forza_parameter_catalog_path()?;
    Some(parse_catalog_file(&path))
}

fn load_forza_catalog_entries(game_id: Option<&str>) -> Vec<ParameterCatalogEntry> {
    load_forza_parameter_catalog(game_id).unwrap_or_default()
}

fn get_forza_parameters(
    config_dir: &Path,
    install_dir: Option<&Path>,
    game_id: Option<&str>,
) -> Result<Vec<GameParameter>, String> {
    use crate::forza::user_config::read_user_config;

    let entries = load_forza_catalog_entries(game_id);
    let (settings, selections) = read_user_config(config_dir)?;

    let mut parameters = Vec::new();
    for entry in entries {
        let file = entry.file.as_deref().unwrap_or("UserConfigSelections");
        let section = entry.section.as_deref().unwrap_or("selections");

        if file == "media" {
            let rel = if section.is_empty() {
                entry.key.clone()
            } else {
                format!("{section}/{}", entry.key)
            };
            let installed = install_dir
                .map(|dir| dir.join("media").join(&rel).is_file())
                .unwrap_or(false);
            let value = if installed {
                "установлено в игре".to_string()
            } else {
                "копируется пресетом".to_string()
            };
            parameters.push(entry_to_parameter(
                &entry,
                &entry.key,
                section,
                file,
                &value,
                true,
                installed,
            ));
            continue;
        }

        let value = if section == "selections" {
            selections
                .get(&entry.key)
                .cloned()
                .unwrap_or_default()
        } else {
            forza_setting_display_value(settings.get(&entry.key))
        };
        let present = if section == "selections" {
            selections.contains_key(&entry.key)
        } else {
            settings.contains_key(&entry.key)
        };
        parameters.push(entry_to_parameter(
            &entry,
            &entry.key,
            section,
            file,
            &value,
            true,
            present,
        ));
    }

    for (id, value) in &selections {
        if parameters
            .iter()
            .any(|p| p.section == "selections" && p.key == *id)
        {
            continue;
        }
        parameters.push(unknown_parameter(
            id,
            "selections",
            "UserConfigSelections",
            value,
        ));
    }

    for (tag, node) in &settings {
        if parameters
            .iter()
            .any(|p| p.section == "settings" && p.key == *tag)
        {
            continue;
        }
        let value = forza_setting_display_value(Some(node));
        parameters.push(unknown_parameter(
            tag,
            "settings",
            "UserConfigSelections",
            &value,
        ));
    }

    parameters.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then(a.section.cmp(&b.section))
            .then(a.key.cmp(&b.key))
    });
    Ok(parameters)
}

fn forza_setting_display_value(node: Option<&crate::forza::user_config::XmlNode>) -> String {
    let Some(node) = node else {
        return String::new();
    };
    node.attrs
        .get("value")
        .or_else(|| node.attrs.get("TrackCullDistanceReduced"))
        .cloned()
        .unwrap_or_else(|| {
            node.attrs
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(", ")
        })
}

fn infer_value_type(value: &str) -> String {
    if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
        "bool".to_string()
    } else if value.eq_ignore_ascii_case("on") || value.eq_ignore_ascii_case("off") {
        "enum".to_string()
    } else if value.parse::<i64>().is_ok() {
        "int".to_string()
    } else if value.parse::<f64>().is_ok() {
        "float".to_string()
    } else {
        "string".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forza_catalog_parses_with_optional_impact() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("vps")
            .join("source")
            .join("forza-fh6")
            .join("parameter-catalog.json");
        if !path.is_file() {
            return;
        }
        let entries = parse_catalog_file(&path);
        assert!(entries.len() > 30, "expected rich forza catalog");
        assert!(
            entries.iter().any(|e| e.key == "XeSSAA"),
            "XeSSAA must parse"
        );
        assert!(
            entries.iter().any(|e| e.file.as_deref() == Some("media")),
            "media entries expected"
        );
    }

    #[test]
    fn loads_split_catalog() {
        let catalog = load_parameter_catalog();
        assert!(catalog.len() > 50);
        assert!(catalog.iter().any(|e| e.key == "r.Streaming.PoolSize"));
        assert!(catalog.iter().any(|e| e.key == "sg.LandscapeQuality"));
        assert!(
            catalog.iter().any(|e| e.key == "gfx-enable-gfx-jobs"),
            "merged catalog includes unity entries from ue-catalog pack"
        );
    }

    #[test]
    fn unity_catalog_has_boot_params() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("catalog").join("unity.json");
        let entries = parse_catalog_file(&path);
        assert!(entries.len() >= 30);
        assert!(entries.iter().any(|e| e.key == "job-worker-count"));
    }

    #[test]
    fn file_key_fallback_matches_engine_cvar() {
        let catalog = load_parameter_catalog();
        let index = build_catalog_index(catalog);
        let matched = lookup_entry(
            &index,
            "Engine.ini",
            "SystemSettings",
            "r.ViewDistanceScale",
        );
        assert!(matched.is_some());
    }

    #[test]
    fn by_key_matches_cvar_in_different_section() {
        let catalog = load_parameter_catalog();
        let index = build_catalog_index(catalog);
        let matched = lookup_entry(
            &index,
            "Engine.ini",
            "ConsoleVariables",
            "r.ViewDistanceScale",
        );
        assert!(matched.is_some());
    }

    #[test]
    fn unknown_r_cvar_gets_range_pattern() {
        let p = unknown_parameter(
            "r.Lumen.Reflections.Quality",
            "SystemSettings",
            "Engine.ini",
            "2",
        );
        assert!(p.min.is_some() && p.max.is_some());
        assert!(p.value_hint.is_some());
    }
}
