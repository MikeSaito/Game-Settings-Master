mod config;
mod io;
mod path_safety;
mod process;

pub use config::ensure_config_writable;
pub use io::{
    clear_readonly, read_file_bytes, read_utf8_text_file, write_file_bytes, write_file_bytes_opts,
};
pub use path_safety::{
    ensure_safe_child_file, is_allowed_config_ini_filename, is_allowed_restore_filename,
    is_safe_backup_id, is_safe_exe_basename, is_safe_ini_key_name, is_safe_ini_section_name,
    is_safe_ini_value, normalize_ini_section_name, path_within_root, safe_child_path,
};
pub use process::{is_exe_running, kill_exe};

#[cfg(test)]
#[path = "fs_util_tests.rs"]
mod tests;
