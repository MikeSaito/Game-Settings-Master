/// Стабильный маркер в тексте ошибки «игра запущена» — не зависит от формулировки UI.
pub const RUNNING_GAME_ERROR_MARKER: &str = "GSM_ERR_GAME_RUNNING:";

pub fn is_running_game_error(err: &str) -> bool {
    err.starts_with(RUNNING_GAME_ERROR_MARKER)
}

pub fn running_game_ini_blocked(exe: &str) -> String {
    format!(
        "{RUNNING_GAME_ERROR_MARKER}Игра «{exe}» запущена. Закройте игру перед применением — иначе ini-файлы заблокированы."
    )
}

pub fn running_game_reshade_blocked(exe: &str) -> String {
    format!(
        "{RUNNING_GAME_ERROR_MARKER}Игра «{exe}» запущена. Закройте игру перед изменением ReShade — proxy DLL и ReShade.ini заблокированы процессом."
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
