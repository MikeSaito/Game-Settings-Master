use crate::discovery::{find_game_by_id, normalize_install_dir};
use crate::core::models::GameProfile;

pub(crate) fn find_profile_by_id(game_id: &str) -> Result<Option<GameProfile>, String> {
    find_game_by_id(game_id)
}

pub(crate) fn normalize_path_cmp(path: &str) -> String {
    normalize_install_dir(path)
}
