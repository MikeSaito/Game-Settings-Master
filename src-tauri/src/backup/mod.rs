pub(crate) mod metadata;
mod migrate;
pub(crate) mod paths;
mod reset;
mod restore;
mod snapshot;

pub use reset::reset_config_all_targets;
pub use restore::resolve_backup_config_dir;
pub use snapshot::{backup_config_dir, list_backups_for_platform};

#[cfg(test)]
#[path = "backup_tests.rs"]
mod tests;
