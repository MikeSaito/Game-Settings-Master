mod overrides;
mod persist;
mod storage;
mod trust;

pub use overrides::{
    delete_override, get_overrides_for_game, save_override, validate_override_bounds,
};
pub use persist::{load_saved_profiles, prune_stale_saved_profiles, remove_profile, save_profile};
pub use storage::app_data_dir;
pub use trust::{
    ensure_known_game_id, ensure_trusted_ipc_profile, is_stale_saved_profile,
    resolve_trusted_profile,
};

#[cfg(test)]
#[path = "profiles_tests.rs"]
mod tests;
