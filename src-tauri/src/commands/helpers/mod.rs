mod custom_changes;
mod exe;
mod guard;
mod paths;
mod profile;
mod trust;

pub(crate) use custom_changes::validate_custom_changes_payload;
pub(crate) use exe::{resolve_trusted_close_exe_name, resolve_write_exe_name};
pub(crate) use guard::{
    ensure_all_targets_writable, guard_config_dir_for_write, guard_write_context,
};
pub(crate) use paths::resolve_ue_config_path;
pub(crate) use profile::{find_profile_by_id, normalize_path_cmp};
pub(crate) use trust::{validate_config_dir_for_game, validate_install_dir_for_game};

#[cfg(test)]
#[path = "ipc_tests.rs"]
mod ipc_tests;
