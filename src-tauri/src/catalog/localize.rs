use super::humanize::humanize_cvar_key;

const STUB_DESCRIPTION_MARKERS: &[&str] = &[
    "see Unreal documentation",
    "Common in Engine.ini",
    "Change with care",
    "Часто встречается в Engine.ini",
    "UE CVar (",
    "Стандартный UE CVar",
];

pub(crate) fn is_stub_description(text: &str) -> bool {
    let normalized = text.trim();
    if normalized.is_empty() {
        return true;
    }
    STUB_DESCRIPTION_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
}

pub(crate) fn pick_localized(ru: &str, en: &Option<String>) -> String {
    let en_str = en.as_deref().filter(|s| !s.trim().is_empty());
    let ru_stub = is_stub_description(ru);
    let en_stub = en_str.map(is_stub_description).unwrap_or(true);

    match crate::i18n::current_lang() {
        crate::i18n::Lang::En => {
            if let Some(e) = en_str {
                if !en_stub {
                    return e.to_string();
                }
            }
            if !ru_stub {
                return ru.to_string();
            }
            en_str.unwrap_or(ru).to_string()
        }
        _ => {
            if !ru_stub {
                return ru.to_string();
            }
            if let Some(e) = en_str {
                if !en_stub {
                    return e.to_string();
                }
            }
            ru.to_string()
        }
    }
}

fn is_poor_title(title: &str, key: &str) -> bool {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return true;
    }
    if trimmed.eq_ignore_ascii_case(key) {
        return true;
    }
    if let Some(last) = key.rsplit('.').next() {
        if trimmed.eq_ignore_ascii_case(last) {
            return true;
        }
    }
    false
}

fn looks_english_only(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }
    let has_latin = trimmed.chars().any(|c| c.is_ascii_alphabetic());
    let has_cyrillic = trimmed
        .chars()
        .any(|c| matches!(c, '\u{0400}'..='\u{04FF}'));
    has_latin && !has_cyrillic
}

pub(crate) fn pick_title(ru: &str, en: &Option<String>, key: &str) -> String {
    let title = pick_localized(ru, en);
    let needs_humanize = is_poor_title(&title, key)
        || (crate::i18n::current_lang() == crate::i18n::Lang::Ru && looks_english_only(&title));
    if needs_humanize {
        humanize_cvar_key(key)
    } else {
        title
    }
}

pub(crate) fn infer_description_quality(description: &str) -> Option<String> {
    if is_stub_description(description) {
        return Some("auto".to_string());
    }
    if description.starts_with("CVar \"") {
        return Some("semi".to_string());
    }
    Some("human".to_string())
}
