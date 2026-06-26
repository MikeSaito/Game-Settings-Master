use crate::core::models::IniFile;
use crate::ini::encoding::{detect_encoding, write_text, IniEncoding};
use std::path::Path;

/// When creating a new ini in the config folder, inherits UTF-8/UTF-16 from `encoding_hint`
/// (usually GameUserSettings.ini) so the game does not ignore Engine.ini.
pub fn write_ini_file_with_encoding_hint(
    path: &Path,
    ini: &IniFile,
    encoding_hint: Option<&Path>,
) -> Result<(), String> {
    let encoding = resolve_write_encoding(path, encoding_hint);
    let content = serialize_ini(ini);
    write_text(path, &content, encoding)
}

fn resolve_write_encoding(path: &Path, encoding_hint: Option<&Path>) -> IniEncoding {
    if path.exists() {
        return detect_encoding(path);
    }
    if let Some(hint) = encoding_hint {
        if hint.exists() {
            return detect_encoding(hint);
        }
    }
    IniEncoding::Utf8
}

pub(crate) fn serialize_ini(ini: &IniFile) -> String {
    let mut lines: Vec<String> = Vec::new();

    for (section_name, section) in &ini.sections {
        if section_name.is_empty() {
            for line in &section.preamble {
                lines.push(line.clone());
            }
            for (key, value) in &section.entries {
                lines.push(format!("{key}={value}"));
            }
            continue;
        }

        if !lines.is_empty() && !lines.last().map(|l| l.is_empty()).unwrap_or(false) {
            lines.push(String::new());
        }

        lines.push(format!("[{section_name}]"));
        for line in &section.preamble {
            lines.push(line.clone());
        }
        for (key, value) in &section.entries {
            lines.push(format!("{key}={value}"));
        }
    }

    if lines.is_empty() {
        String::new()
    } else {
        lines.join("\r\n") + "\r\n"
    }
}
