mod config_dir;
mod covers;
mod manual;
mod profile;
mod scan;

pub use config_dir::set_game_config_dir;
pub use covers::{import_game_cover_cmd, open_config_folder, remove_game_cover_cmd};
pub use manual::{add_manual_game, resolve_config_from_path};
pub use profile::{remove_game_profile, save_game_profile};
pub use scan::scan_games;

#[cfg(test)]
#[path = "games_tests.rs"]
mod tests;
