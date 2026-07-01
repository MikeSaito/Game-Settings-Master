mod backups;
mod config;
mod crash_report;
mod games;
mod helpers;
mod launch;
mod system;

pub use backups::{list_backups_cmd, reset_config_to_user_cmd, restore_backup_cmd};
pub use config::{
    apply_custom_cmd, apply_game_override, delete_game_override, get_game_config,
    get_game_overrides, get_game_parameters_cmd, get_scalability_limits_cmd, save_game_override,
};
pub use crash_report::{clear_crash_reports_cmd, list_crash_reports_cmd, submit_crash_report_cmd};
pub use games::{
    add_manual_game, import_game_cover_cmd, open_config_folder, remove_game_cover_cmd,
    remove_game_profile, resolve_config_from_path, save_game_profile, scan_games,
    set_game_config_dir,
};
pub use launch::launch_game_cmd;
pub use system::{
    close_game_cmd, get_desktop_resolution_cmd, get_gpu_info_cmd, is_game_running_cmd,
    set_app_background_mode_cmd, set_language_cmd,
};
