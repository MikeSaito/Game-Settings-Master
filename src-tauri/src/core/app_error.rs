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
            AppErrorCode::GameNotFound
            | AppErrorCode::InvalidPath
            | AppErrorCode::PresetNotFound => err.message,
            AppErrorCode::IoError | AppErrorCode::Validation | AppErrorCode::Other => err.message,
        }
    }
}

#[cfg(test)]
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
#[path = "app_error_tests.rs"]
mod tests;
