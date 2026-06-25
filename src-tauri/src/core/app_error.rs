/// Legacy marker kept for parsing old invoke error strings during transition.
pub const RUNNING_GAME_ERROR_MARKER: &str = "GSM_ERR_GAME_RUNNING:";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum AppErrorCode {
    GameRunning,
    InvalidPath,
    GameNotFound,
    PresetNotFound,
    IoError,
    Validation,
    Other,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct AppInvokeError {
    pub code: AppErrorCode,
    pub message: String,
}

/// Internal alias — command handlers and helpers build structured invoke errors.
pub type AppError = AppInvokeError;

impl AppInvokeError {
    pub fn new(code: AppErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
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

pub fn is_running_game_error(err: &str) -> bool {
    err.starts_with(RUNNING_GAME_ERROR_MARKER)
}

impl From<String> for AppInvokeError {
    fn from(message: String) -> Self {
        if is_running_game_error(&message) {
            Self::game_running(message.replace(RUNNING_GAME_ERROR_MARKER, ""))
        } else {
            Self::other(message)
        }
    }
}

impl From<&str> for AppInvokeError {
    fn from(message: &str) -> Self {
        message.to_string().into()
    }
}

#[cfg(test)]
#[path = "app_error_tests.rs"]
mod tests;

pub fn running_game_ini_blocked(exe: &str) -> AppInvokeError {
    AppInvokeError::game_running(crate::i18n::t(
        &format!(
            "Игра «{exe}» запущена. Закройте игру перед применением — иначе ini-файлы заблокированы."
        ),
        &format!(
            "Game «{exe}» is running. Close the game before applying changes — otherwise ini files are locked."
        ),
    ))
}
