mod build;
mod cache;
mod load;
mod lookup;

pub(crate) use build::catalog_id;
pub(crate) use cache::get_or_build_catalog_index;
pub(crate) use lookup::{lookup_entry, should_include_catalog_entry};

#[cfg(test)]
pub(crate) use build::build_catalog_index;
#[cfg(test)]
pub(crate) use cache::{catalog_build_count, invalidate_catalog_cache};
#[cfg(test)]
pub(crate) use load::load_parameter_catalog_for_family;
