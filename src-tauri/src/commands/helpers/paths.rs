use crate::discovery::platform_hints_for_game;
use crate::ini::platform::reconcile_config_dir;
use std::path::PathBuf;

pub(crate) fn resolve_ue_config_path(
    path: PathBuf,
    game_id: Option<&str>,
    engine_family: Option<&str>,
) -> PathBuf {
    let hints = platform_hints_for_game(game_id, engine_family);
    reconcile_config_dir(&path, &hints)
}
