mod display;
mod backup;
mod catalog;
mod commands;
mod covers;
mod discovery;
mod gpu;
mod fs_util;
mod ini;
mod launch;
mod models;
mod presets;
mod profiles;
mod scalability;
mod unity;
mod forza;
mod remote_presets;

use commands::{
    add_manual_game, apply_custom_cmd, apply_game_override, apply_game_preset_cmd, apply_preset_cmd,
    delete_game_override, get_game_config, get_game_overrides,
    get_desktop_resolution_cmd, get_game_parameters_cmd, get_gpu_info_cmd, get_scalability_limits_cmd,
    is_game_running_cmd,
    close_game_cmd,
    launch_game_cmd,
    import_game_cover_cmd,
    list_backups_cmd, list_presets_cmd,
    get_preset_server_status_cmd, set_preset_server_url_cmd, sync_presets_cmd,
    open_config_folder, preview_preset_cmd,
    remove_game_cover_cmd, remove_game_profile, resolve_config_from_path,
    reset_config_to_user_cmd, restore_backup_cmd, save_game_override, save_game_profile,
    scan_games, set_game_config_dir,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|_| {
            if remote_presets::effective_base_url().is_some() {
                std::thread::spawn(|| {
                    let _ = remote_presets::sync_now(false);
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_games,
            get_gpu_info_cmd,
            get_desktop_resolution_cmd,
            is_game_running_cmd,
            close_game_cmd,
            launch_game_cmd,
            get_game_config,
            get_game_parameters_cmd,
            get_scalability_limits_cmd,
            list_presets_cmd,
            get_preset_server_status_cmd,
            set_preset_server_url_cmd,
            sync_presets_cmd,
            preview_preset_cmd,
            apply_game_preset_cmd,
            apply_preset_cmd,
            apply_custom_cmd,
            list_backups_cmd,
            restore_backup_cmd,
            reset_config_to_user_cmd,
            add_manual_game,
            resolve_config_from_path,
            set_game_config_dir,
            save_game_profile,
            remove_game_profile,
            save_game_override,
            get_game_overrides,
            delete_game_override,
            apply_game_override,
            import_game_cover_cmd,
            remove_game_cover_cmd,
            open_config_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
