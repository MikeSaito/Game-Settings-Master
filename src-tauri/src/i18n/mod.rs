use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Ru,
    En,
}

impl Lang {
    fn from_code(code: &str) -> Lang {
        let lower = code.trim().to_ascii_lowercase();
        if lower.starts_with("ru") {
            Lang::Ru
        } else {
            Lang::En
        }
    }

    fn code(self) -> &'static str {
        match self {
            Lang::Ru => "ru",
            Lang::En => "en",
        }
    }
}

const LANG_RU: u8 = 0;
const LANG_EN: u8 = 1;

static CURRENT_LANG: AtomicU8 = AtomicU8::new(LANG_EN);

const SETTINGS_FILE: &str = "app_settings.json";

#[derive(Debug, Default, Serialize, Deserialize)]
struct AppSettings {
    #[serde(default)]
    language: Option<String>,
}

fn settings_path() -> Result<PathBuf, String> {
    Ok(crate::profiles::app_data_dir()?.join(SETTINGS_FILE))
}

pub fn current_lang() -> Lang {
    match CURRENT_LANG.load(Ordering::Relaxed) {
        LANG_EN => Lang::En,
        _ => Lang::Ru,
    }
}

fn store_lang(lang: Lang) {
    let value = match lang {
        Lang::Ru => LANG_RU,
        Lang::En => LANG_EN,
    };
    CURRENT_LANG.store(value, Ordering::Relaxed);
}

/// Pick the localized string for the active language.
pub fn t(ru: &str, en: &str) -> String {
    match current_lang() {
        Lang::Ru => ru.to_string(),
        Lang::En => en.to_string(),
    }
}

/// Read saved language or fall back to the OS locale (English for non-Russian systems).
pub fn init_from_disk() {
    let Ok(path) = settings_path() else {
        store_lang(detect_system_lang());
        return;
    };
    let Ok(raw) = fs::read_to_string(&path) else {
        store_lang(detect_system_lang());
        return;
    };
    if let Ok(settings) = serde_json::from_str::<AppSettings>(&raw) {
        if let Some(code) = settings.language {
            store_lang(Lang::from_code(&code));
            return;
        }
    }
    store_lang(detect_system_lang());
}

fn detect_system_lang() -> Lang {
    sys_locale::get_locale()
        .map(|locale| Lang::from_code(&locale))
        .unwrap_or(Lang::En)
}

/// Set and persist the active language.
pub fn set_language(code: &str) -> Result<(), String> {
    let lang = Lang::from_code(code);
    store_lang(lang);
    let path = settings_path()?;
    let settings = AppSettings {
        language: Some(lang.code().to_string()),
    };
    let raw = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&path, raw).map_err(|e| e.to_string())?;
    Ok(())
}
