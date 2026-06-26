use super::{
    is_running_game_error, running_game_ini_blocked, AppError, AppErrorCode, AppInvokeError,
};

#[test]
fn marker_detects_running_game_errors() {
    let legacy = format!("{}blocked", super::RUNNING_GAME_ERROR_MARKER);
    assert!(is_running_game_error(&legacy));
    assert!(!is_running_game_error("другая ошибка"));
}

#[test]
fn running_game_error_has_structured_code() {
    let err = running_game_ini_blocked("Game.exe");
    assert_eq!(err.code, AppErrorCode::GameRunning);
    assert!(err.message.contains("Game.exe"));
    assert!(!err.message.contains(super::RUNNING_GAME_ERROR_MARKER));
}

#[test]
fn game_not_found_roundtrips_message() {
    let err = AppError::game_not_found("missing game");
    assert_eq!(err.code, AppErrorCode::GameNotFound);
    assert_eq!(err.message, "missing game");
}

#[test]
fn preset_not_found_roundtrips_message() {
    let err = AppError::preset_not_found("no preset");
    assert_eq!(err.code, AppErrorCode::PresetNotFound);
    assert_eq!(err.message, "no preset");
}

#[test]
fn validation_error_roundtrips_message() {
    let err = AppError::validation("bad value");
    assert_eq!(err.code, AppErrorCode::Validation);
    assert_eq!(err.message, "bad value");
}

#[test]
fn legacy_string_converts_to_game_running() {
    let legacy = format!("{}Игра запущена", super::RUNNING_GAME_ERROR_MARKER);
    let err: AppInvokeError = legacy.into();
    assert_eq!(err.code, AppErrorCode::GameRunning);
    assert_eq!(err.message, "Игра запущена");
}
