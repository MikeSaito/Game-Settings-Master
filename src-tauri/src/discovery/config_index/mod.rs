mod matcher;
mod scan;
mod types;

pub use matcher::{build_match_candidates, match_config_from_index, normalize_key};
pub use scan::scan_local_appdata_configs;

#[cfg(test)]
#[path = "config_index_tests.rs"]
mod tests;
