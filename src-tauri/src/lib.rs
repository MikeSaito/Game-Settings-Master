mod backup;
#[cfg(test)]
mod bindings_export;
mod catalog;
mod changes;
mod commands;
mod core;
mod covers;
mod crash_report;
mod diagnostics;
mod discovery;
mod display;
mod fs_util;
mod gpu;
mod i18n;
mod ini;
mod input;
mod launch;
mod presets;
mod profiles;
mod scalability;

use commands::{
    add_manual_game, apply_custom_cmd, apply_game_override, clear_crash_reports_cmd,
    close_game_cmd, delete_game_override, get_desktop_resolution_cmd, get_game_config,
    get_game_overrides, get_game_parameters_cmd, get_gpu_info_cmd, get_scalability_limits_cmd,
    import_game_cover_cmd, is_game_running_cmd, launch_game_cmd, list_backups_cmd,
    list_crash_reports_cmd, open_config_folder, remove_game_cover_cmd, remove_game_profile,
    reset_config_to_user_cmd, resolve_config_from_path, restore_backup_cmd, save_game_override,
    save_game_profile, scan_games, set_app_background_mode_cmd, set_game_config_dir,
    set_language_cmd, submit_crash_report_cmd,
};

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if let Ok(resource_dir) = app.path().resource_dir() {
                core::resource_paths::init_resource_root(resource_dir);
            }
            i18n::init_from_disk();
            diagnostics::start(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            changes::prepare_changes,
            commands::snapshots::create_snapshot,
            commands::snapshots::rename_snapshot,
            commands::snapshots::compare_snapshots,
            commands::snapshots::prepare_parameter_restore,
            changes::apply_prepared_changes,
            changes::validate_prepared_changes,
            changes::discard_prepared_changes,
            input::get_input_document,
            diagnostics::get_diagnostic_report,
            scan_games,
            discovery::other_stores::get_discovery_warnings,
            get_gpu_info_cmd,
            gpu::list_gpu_adapters,
            gpu::get_game_gpu,
            gpu::set_game_gpu,
            gpu::open_graphics_settings,
            get_desktop_resolution_cmd,
            is_game_running_cmd,
            set_language_cmd,
            set_app_background_mode_cmd,
            close_game_cmd,
            launch_game_cmd,
            launch::executable::list_game_executables,
            launch::executable::set_game_executable,
            get_game_config,
            get_game_parameters_cmd,
            get_scalability_limits_cmd,
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
            submit_crash_report_cmd,
            list_crash_reports_cmd,
            clear_crash_reports_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
