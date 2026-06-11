mod app_error;
mod backup;
mod catalog;
mod commands;
mod covers;
mod discovery;
mod display;
mod forza;
mod fs_util;
mod gpu;
mod ini;
mod launch;
mod models;
mod reshade;
mod presets;
mod process_util;
mod profiles;
mod remote_presets;
mod scalability;
mod unity;

use commands::{
    add_manual_game, apply_custom_cmd, apply_game_override, apply_game_preset_cmd,
    apply_preset_cmd, close_game_cmd, delete_game_override, get_desktop_resolution_cmd,
    get_game_config, get_game_overrides, get_game_parameters_cmd, get_gpu_info_cmd,
    get_preset_server_status_cmd, get_scalability_limits_cmd, import_game_cover_cmd,
    ensure_reshade_installed_cmd, get_reshade_preset_details_cmd, get_reshade_settings_cmd,
    get_reshade_status_cmd, get_reshade_workspace_cmd,
    install_reshade_cmd, is_game_running_cmd, list_reshade_presets_for_game_cmd,
    open_game_folder_cmd, update_reshade_preset_parameters_cmd,
    launch_game_cmd, list_backups_cmd, list_presets_cmd, open_config_folder, preview_preset_cmd,
    remove_game_cover_cmd, remove_game_profile, remove_reshade_cmd, reset_config_to_user_cmd,
    resolve_config_from_path, restore_backup_cmd, save_game_override, save_game_profile,
    scan_games, set_app_background_mode_cmd, set_game_config_dir, set_preset_server_url_cmd,
    set_reshade_per_game_cmd, set_reshade_settings_cmd, should_prompt_reshade_api_cmd,
    sync_presets_cmd, update_reshade_preset_cmd,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|_| {
            if remote_presets::effective_base_url().is_some() {
                std::thread::spawn(|| {
                    #[cfg(windows)]
                    {
                        use windows_sys::Win32::System::Threading::{
                            GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_LOWEST,
                        };
                        unsafe {
                            SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_LOWEST);
                        }
                    }
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
            set_app_background_mode_cmd,
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
            get_reshade_settings_cmd,
            set_reshade_settings_cmd,
            get_reshade_status_cmd,
            get_reshade_workspace_cmd,
            list_reshade_presets_for_game_cmd,
            install_reshade_cmd,
            remove_reshade_cmd,
            update_reshade_preset_cmd,
            update_reshade_preset_parameters_cmd,
            set_reshade_per_game_cmd,
            should_prompt_reshade_api_cmd,
            ensure_reshade_installed_cmd,
            get_reshade_preset_details_cmd,
            open_game_folder_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
