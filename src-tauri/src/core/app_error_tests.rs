use super::{
    is_running_game_error, running_game_ini_blocked, AppError,
};

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
