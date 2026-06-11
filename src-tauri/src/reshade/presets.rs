use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReShadePresetInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub author: bool,
}

pub fn list_presets() -> Vec<ReShadePresetInfo> {
    vec![
        ReShadePresetInfo {
            id: "performance".to_string(),
            name: "Performance".to_string(),
            description: "Лёгкое повышение резкости без тяжёлых эффектов — минимум нагрузки на GPU.".to_string(),
            author: false,
        },
        ReShadePresetInfo {
            id: "clarity".to_string(),
            name: "Clarity".to_string(),
            description: "Баланс чёткости и контраста для повседневной игры.".to_string(),
            author: false,
        },
        ReShadePresetInfo {
            id: "cinematic".to_string(),
            name: "Cinematic".to_string(),
            description: "Мягкая цветокоррекция и виньетка — чуть кинематографичнее картинка.".to_string(),
            author: false,
        },
    ]
}

pub fn preset_exists(id: &str) -> bool {
    list_presets().iter().any(|p| p.id == id)
}

pub fn parse_techniques(ini: &str) -> Vec<String> {
    parse_techniques_inner(ini)
}

pub fn parse_parameters(ini: &str) -> Vec<PresetParameter> {
    parse_parameters_inner(ini)
}

pub fn reshade_bundle_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("presets")
        .join("reshade")
}

pub fn preset_ini_path(preset_id: &str) -> Result<PathBuf, String> {
    if !preset_exists(preset_id) {
        return Err(format!("Неизвестный пресет ReShade: {preset_id}"));
    }
    let path = reshade_bundle_dir()
        .join("presets")
        .join(preset_id)
        .join("ReShade.ini");
    if !path.is_file() {
        return Err(format!(
            "Файл пресета не найден в бандле: {}",
            path.display()
        ));
    }
    Ok(path)
}

pub fn base_ini_path() -> PathBuf {
    reshade_bundle_dir().join("bin").join("ReShade.ini")
}

pub fn bundled_file(name: &str) -> PathBuf {
    reshade_bundle_dir().join("bin").join(name)
}

pub fn bundled_shaders_dir() -> PathBuf {
    reshade_bundle_dir().join("shaders")
}

pub fn bundled_shaders_effects_dir() -> PathBuf {
    bundled_shaders_dir().join("Shaders")
}

pub const SHADERS_FINGERPRINT_FILE: &str = ".gsm-reshade-shaders-fingerprint";

fn collect_fx_fingerprint(dir: &Path) -> (u64, u64) {
    let mut count = 0u64;
    let mut size = 0u64;
    let Ok(entries) = fs::read_dir(dir) else {
        return (0, 0);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let (c, s) = collect_fx_fingerprint(&path);
            count += c;
            size += s;
        } else if path.extension().is_some_and(|x| x == "fx") && path.is_file() {
            count += 1;
            size += entry.metadata().ok().map(|m| m.len()).unwrap_or(0);
        }
    }
    (count, size)
}

/// Fingerprint of bundled shader pack — used to detect stale copies in game folders.
pub fn shaders_bundle_fingerprint() -> Option<String> {
    let dir = bundled_shaders_effects_dir();
    if !dir.is_dir() {
        return None;
    }
    let (count, size) = collect_fx_fingerprint(&dir);
    if count == 0 {
        return None;
    }
    Some(format!("{count}:{size}"))
}

fn shaders_available_in_bundle_uncached() -> bool {
    bundled_shaders_effects_dir().is_dir()
        && fs::read_dir(bundled_shaders_effects_dir())
            .map(|entries| entries.flatten().any(|e| e.path().extension().is_some_and(|x| x == "fx")))
            .unwrap_or(false)
}

pub fn shaders_available_in_bundle() -> bool {
    shaders_available_in_bundle_uncached()
}

pub fn shaders_present_in_game(target_dir: &Path) -> bool {
    let shaders = target_dir.join("reshade-shaders").join("Shaders");
    shaders.is_dir()
        && fs::read_dir(&shaders)
            .map(|entries| entries.flatten().any(|e| e.path().extension().is_some_and(|x| x == "fx")))
            .unwrap_or(false)
}

#[derive(Debug, Clone, Serialize)]
pub struct PresetParameter {
    pub effect: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PresetDetails {
    pub id: String,
    pub techniques: Vec<String>,
    pub parameters: Vec<PresetParameter>,
    pub shaders_available: bool,
}

pub fn preset_details(preset_id: &str, game_id: Option<&str>) -> Result<PresetDetails, String> {
    if !crate::fs_util::is_safe_pack_id(preset_id) {
        return Err(format!("Недопустимый идентификатор пресета: {preset_id}"));
    }
    if !super::game_presets::preset_exists_for(preset_id, game_id) {
        return Err(format!("Неизвестный пресет ReShade: {preset_id}"));
    }
    let raw = super::game_presets::read_preset_ini_for(preset_id, game_id)?;
    Ok(PresetDetails {
        id: preset_id.to_string(),
        techniques: parse_techniques_inner(&raw),
        parameters: parse_parameters_inner(&raw),
        shaders_available: shaders_available_in_bundle(),
    })
}

fn parse_techniques_inner(ini: &str) -> Vec<String> {
    for line in ini.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(';') || trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("Techniques=") {
            return rest
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    Vec::new()
}

fn parse_parameters_inner(ini: &str) -> Vec<PresetParameter> {
    let mut out = Vec::new();
    let mut section: Option<String> = None;
    for line in ini.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(';') || trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = Some(trimmed.trim_matches(&['[', ']'][..]).to_string());
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            if let Some(effect) = section.clone() {
                if effect.ends_with(".fx") {
                    out.push(PresetParameter {
                        effect: effect.trim_end_matches(".fx").to_string(),
                        key: key.trim().to_string(),
                        value: value.trim().to_string(),
                    });
                }
            }
        }
    }
    out
}

pub fn required_shader_files_for(preset_id: &str, game_id: Option<&str>) -> Vec<String> {
    let ini = super::game_presets::read_preset_ini_for(preset_id, game_id).unwrap_or_default();
    parse_techniques_inner(&ini)
        .into_iter()
        .map(|t| format!("{t}.fx"))
        .collect()
}

pub fn preset_shaders_ready_for(preset_id: &str, game_id: Option<&str>) -> bool {
    let required = required_shader_files_for(preset_id, game_id);
    if required.is_empty() {
        return true;
    }
    if !shaders_available_in_bundle() {
        return false;
    }
    let dir = bundled_shaders_effects_dir();
    required.iter().all(|f| dir.join(f).is_file())
}

/// Безопасный пресет без внешних эффектов — игра стартует даже без shader pack.
pub fn safe_preset_overlay() -> &'static str {
    "; GSM safe fallback — эффекты отключены (нет шейдеров в бандле)\nTechniques=\nTechniqueSorting=\n"
}

pub fn read_base_ini() -> Result<String, String> {
    let path = base_ini_path();
    if path.is_file() {
        fs::read_to_string(&path).map_err(|e| format!("Не удалось прочитать ReShade.ini: {e}"))
    } else {
        Ok(default_base_ini())
    }
}

fn default_base_ini() -> String {
    "[GENERAL]\nEffectSearchPaths=.\\reshade-shaders\\Shaders\nTextureSearchPaths=.\\reshade-shaders\\Textures\nPreprocessorDefinitions=\nPresetPath=.\nPresetTransitionDuration=1000\n".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shaders_available_in_bundle_is_stable() {
        let first = shaders_available_in_bundle();
        let second = shaders_available_in_bundle();
        assert_eq!(first, second);
    }

    #[test]
    fn shaders_bundle_fingerprint_none_without_effects() {
        let fp = shaders_bundle_fingerprint();
        if shaders_available_in_bundle() {
            assert!(fp.is_some());
            assert!(fp.unwrap().contains(':'));
        } else {
            assert!(fp.is_none());
        }
    }

    #[test]
    fn collect_fx_fingerprint_includes_nested_dirs() {
        let dir = tempfile::TempDir::new().unwrap();
        let nested = dir.path().join("sub");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(dir.path().join("a.fx"), b"x").unwrap();
        std::fs::write(nested.join("b.fx"), b"yy").unwrap();
        let (count, size) = super::collect_fx_fingerprint(dir.path());
        assert_eq!(count, 2);
        assert_eq!(size, 3);
    }
}
