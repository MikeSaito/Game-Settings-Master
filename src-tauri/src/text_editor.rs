//! Whole-file INI editing. Source revisions are checked before preparing a write.
use crate::changes::{digest, effective_dir, read_optional};
use crate::commands::helpers::{find_profile_by_id, guard_config_dir_for_read};
use crate::core::app_error::{AppError, AppInvokeError};
use serde::{Deserialize, Serialize};

pub(crate) const MAX_TEXT_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct IniTextDocument {
    pub file: String,
    pub config_dir: String,
    pub text: String,
    pub revision: Option<String>,
    pub encoding: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct RawFileUpdate {
    pub file: String,
    pub config_dir: String,
    pub text: String,
    pub revision: Option<String>,
}

#[tauri::command]
pub fn get_ini_text_document(
    game_id: String,
    config_dir: String,
    file: String,
) -> Result<IniTextDocument, AppInvokeError> {
    if !crate::fs_util::is_allowed_config_ini_filename(&file) {
        return Err(AppError::validation(crate::i18n::t(
            "Недопустимый INI-файл",
            "Invalid INI filename",
        )));
    }
    guard_config_dir_for_read(Some(&game_id), &config_dir)?;
    let profile = find_profile_by_id(&game_id)?
        .ok_or_else(|| AppError::validation(crate::i18n::t("Игра не найдена", "Game not found")))?;
    let dir = effective_dir(&profile, &config_dir)?;
    let bytes = read_optional(&dir, &file)?;
    if bytes
        .as_ref()
        .is_some_and(|bytes| bytes.len() > MAX_TEXT_BYTES)
    {
        return Err(AppError::validation(crate::i18n::t(
            "Текстовый редактор поддерживает файлы до 1 МБ",
            "The text editor supports files up to 1 MB",
        )));
    }
    let (text, encoding) = match bytes.as_ref() {
        Some(bytes) => {
            let (text, encoding) = crate::ini::encoding::decode_bytes(bytes)?;
            let name = match encoding {
                crate::ini::encoding::IniEncoding::Utf16Le => "UTF-16 LE",
                _ if bytes.starts_with(&[0xef, 0xbb, 0xbf]) => "UTF-8 BOM",
                _ => "UTF-8",
            };
            (text, name.into())
        }
        None => (String::new(), "UTF-8".into()),
    };
    Ok(IniTextDocument {
        file,
        config_dir: dir.to_string_lossy().into_owned(),
        text,
        revision: bytes.as_ref().map(|bytes| digest(bytes)),
        encoding,
    })
}
