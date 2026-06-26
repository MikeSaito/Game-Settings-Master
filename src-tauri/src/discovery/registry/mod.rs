mod cache;
mod lookup;

pub use cache::{cached_scan_all_games, force_refresh_scan_all_games, invalidate_game_scan_cache};
pub use lookup::find_game_by_id;

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "registry_mtime_tests.rs"]
mod mtime_tests;
