mod config_index;
mod dedupe;
mod enrich;
mod epic;
pub mod known_games;
mod manual;
mod mtime_snapshot;
mod registry;
mod scan_all;
mod steam;
mod ue_detect;
mod ue_version;

pub use config_index::{
    build_match_candidates, match_config_from_index, scan_local_appdata_configs,
};
pub use dedupe::{dedupe_games, merge_game_profile, normalize_install_dir};
pub(crate) use dedupe::dedupe_paths;
pub use enrich::{enrich_config_dir, enrich_engine_flags};
pub use known_games::{known_config_dir, load_known_games, platform_hints_for_game};
pub use manual::profile_from_manual_path;
pub use registry::{
    cached_scan_all_games, find_game_by_id, force_refresh_scan_all_games,
    invalidate_game_scan_cache,
};
pub use scan_all::scan_all_games;
pub use ue_detect::{detect_unreal_engine, is_non_game_install, UeDetectResult};
pub use ue_version::enrich_engine_version;

#[cfg(test)]
#[path = "discovery_tests.rs"]
mod tests;
