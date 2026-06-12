/// Stable marker in the "game is running" error text — independent of UI wording.
pub const RUNNING_GAME_ERROR_MARKER: &str = "GSM_ERR_GAME_RUNNING:";

pub fn is_running_game_error(err: &str) -> bool {
    err.starts_with(RUNNING_GAME_ERROR_MARKER)
}

pub fn running_game_ini_blocked(exe: &str) -> String {
    crate::i18n::t(
        &format!(
            "{RUNNING_GAME_ERROR_MARKER}Игра «{exe}» запущена. Закройте игру перед применением — иначе ini-файлы заблокированы."
        ),
        &format!(
            "{RUNNING_GAME_ERROR_MARKER}Game «{exe}» is running. Close the game before applying changes — otherwise ini files are locked."
        ),
    )
}

pub fn running_game_reshade_blocked(exe: &str) -> String {
    crate::i18n::t(
        &format!(
            "{RUNNING_GAME_ERROR_MARKER}Игра «{exe}» запущена. Закройте игру перед изменением ReShade — proxy DLL и ReShade.ini заблокированы процессом."
        ),
        &format!(
            "{RUNNING_GAME_ERROR_MARKER}Game «{exe}» is running. Close the game before changing ReShade — proxy DLL and ReShade.ini are locked by the process."
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_detects_running_game_errors() {
        assert!(is_running_game_error(&running_game_ini_blocked("Game.exe")));
        assert!(!is_running_game_error("другая ошибка"));
    }
}
