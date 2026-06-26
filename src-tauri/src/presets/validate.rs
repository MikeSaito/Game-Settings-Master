use std::collections::HashMap;

pub(crate) fn validate_ini_payload(
    file_name: &str,
    sections: &HashMap<String, HashMap<String, String>>,
    removals: &HashMap<String, Vec<String>>,
) -> Result<(), String> {
    for (section, entries) in sections {
        if !crate::fs_util::is_safe_ini_section_name(section) {
            return Err(crate::i18n::t(
                &format!("Недопустимая INI-секция в {file_name}: {section}"),
                &format!("Invalid INI section in {file_name}: {section}"),
            ));
        }
        for (key, value) in entries {
            if !crate::fs_util::is_safe_ini_key_name(key) {
                return Err(crate::i18n::t(
                    &format!("Недопустимый INI-ключ в {file_name}: {key}"),
                    &format!("Invalid INI key in {file_name}: {key}"),
                ));
            }
            if !crate::fs_util::is_safe_ini_value(value) {
                return Err(crate::i18n::t(
                    &format!("Недопустимое INI-значение для {key}"),
                    &format!("Invalid INI value for {key}"),
                ));
            }
        }
    }
    for (section, keys) in removals {
        if !crate::fs_util::is_safe_ini_section_name(section) {
            return Err(crate::i18n::t(
                &format!("Недопустимая INI-секция в removals {file_name}: {section}"),
                &format!("Invalid INI section in removals {file_name}: {section}"),
            ));
        }
        for key in keys {
            if !crate::fs_util::is_safe_ini_key_name(key) {
                return Err(crate::i18n::t(
                    &format!("Недопустимый INI-ключ в removals {file_name}: {key}"),
                    &format!("Invalid INI key in removals {file_name}: {key}"),
                ));
            }
        }
    }
    Ok(())
}
