mod catalog_index;
mod dedupe;
mod humanize;
mod injection;
mod loader;
mod localize;
mod overlay;
mod parameter_build;
mod scalability_tiers;
mod types;
mod unknown;
mod version;

pub use loader::get_game_parameters;

pub(crate) use catalog_index::get_or_build_catalog_index;
pub(crate) use types::CatalogIndex;
pub(crate) use version::{parse_ue_semver, reference_applies_to_version, UeSemver};

#[cfg(test)]
pub(crate) use catalog_index::invalidate_catalog_cache;

#[cfg(test)]
mod loader_tests;
