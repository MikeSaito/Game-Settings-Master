/// Stable marker in the "game is running" error text — independent of UI wording.
pub const RUNNING_GAME_ERROR_MARKER: &str = "GSM_ERR_GAME_RUNNING:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppErrorCode {
    GameRunning,
    InvalidPath,
    GameNotFound,
    PresetNotFound,
    IoError,
    Validation,
    Other,
}

#[derive(Debug, Clone)]
pub struct AppError {
    pub code: AppErrorCode,
    pub message: String,
}

impl AppError {
    pub fn new(code: AppErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn to_invoke_string(self) -> String {
        String::from(self)
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(AppErrorCode::Validation, message)
    }

    pub fn invalid_path(message: impl Into<String>) -> Self {
        Self::new(AppErrorCode::InvalidPath, message)
    }

    pub fn game_not_found(message: impl Into<String>) -> Self {
        Self::new(AppErrorCode::GameNotFound, message)
    }

    pub fn preset_not_found(message: impl Into<String>) -> Self {
        Self::new(AppErrorCode::PresetNotFound, message)
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::new(AppErrorCode::IoError, message)
    }

    pub fn other(message: impl Into<String>) -> Self {
        Self::new(AppErrorCode::Other, message)
    }

    pub fn game_running(message: impl Into<String>) -> Self {
        Self::new(AppErrorCode::GameRunning, message)
    }
}

impl From<AppError> for String {
    fn from(err: AppError) -> String {
        match err.code {
            // Frontend strips the marker; message must stay byte-identical to legacy helpers.
            AppErrorCode::GameRunning => err.message,
            AppErrorCode::GameNotFound | AppErrorCode::InvalidPath | AppErrorCode::PresetNotFound => {
                err.message
            }
            AppErrorCode::IoError | AppErrorCode::Validation | AppErrorCode::Other => err.message,
        }
    }
}

pub fn is_running_game_error(err: &str) -> bool {
    err.starts_with(RUNNING_GAME_ERROR_MARKER)
}

pub fn running_game_ini_blocked(exe: &str) -> String {
    AppError::game_running(crate::i18n::t(
        &format!(
            "{RUNNING_GAME_ERROR_MARKER}Игра «{exe}» запущена. Закройте игру перед применением — иначе ini-файлы заблокированы."
        ),
        &format!(
            "{RUNNING_GAME_ERROR_MARKER}Game «{exe}» is running. Close the game before applying changes — otherwise ini files are locked."
        ),
    ))
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_detects_running_game_errors() {
        assert!(is_running_game_error(&running_game_ini_blocked("Game.exe")));
        assert!(!is_running_game_error("другая ошибка"));
    }

    #[test]
    fn from_app_error_preserves_game_running_marker() {
        let msg = running_game_ini_blocked("Game.exe");
        let err = AppError::game_running(msg.clone());
        let out: String = err.into();
        assert!(is_running_game_error(&out));
        assert_eq!(out, msg);
    }

    #[test]
    fn game_not_found_roundtrips_message() {
        let err = AppError::game_not_found("missing game");
        let out: String = err.into();
        assert_eq!(out, "missing game");
    }

    #[test]
    fn preset_not_found_roundtrips_message() {
        let err = AppError::preset_not_found("no preset");
        let out: String = err.into();
        assert_eq!(out, "no preset");
    }

    #[test]
    fn validation_error_roundtrips_message() {
        let err = AppError::validation("bad value");
        let out: String = err.into();
        assert_eq!(out, "bad value");
    }
}
