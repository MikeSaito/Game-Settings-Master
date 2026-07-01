use super::build::build_catalog_index;
use super::load::load_parameter_catalog_for_family;
use std::sync::Arc;
use std::sync::{Mutex, OnceLock};

use crate::catalog::types::CatalogIndex;

static CATALOG_INDEX_CACHE: OnceLock<Mutex<std::collections::HashMap<String, Arc<CatalogIndex>>>> =
    OnceLock::new();

fn catalog_cache() -> &'static Mutex<std::collections::HashMap<String, Arc<CatalogIndex>>> {
    CATALOG_INDEX_CACHE.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

#[cfg(test)]
static CATALOG_BUILD_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
pub(crate) fn catalog_build_count() -> usize {
    CATALOG_BUILD_COUNT.load(std::sync::atomic::Ordering::SeqCst)
}

#[cfg(test)]
pub fn reset_catalog_build_count() {
    CATALOG_BUILD_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(test)]
pub fn invalidate_catalog_cache() {
    if let Ok(mut guard) = catalog_cache().lock() {
        guard.clear();
    }
}

fn catalog_cache_key(engine_family: Option<&str>) -> &'static str {
    if engine_family == Some("ue4") {
        "ue4"
    } else {
        "ue5"
    }
}

pub(crate) fn get_or_build_catalog_index(engine_family: Option<&str>) -> Arc<CatalogIndex> {
    let key = catalog_cache_key(engine_family);

    if let Ok(guard) = catalog_cache().lock() {
        if let Some(index) = guard.get(key) {
            return Arc::clone(index);
        }
    }

    let catalog = load_parameter_catalog_for_family(engine_family);
    let is_ue4 = engine_family == Some("ue4");
    let index = Arc::new(build_catalog_index(catalog, is_ue4));

    if let Ok(mut guard) = catalog_cache().lock() {
        if let Some(existing) = guard.get(key) {
            return Arc::clone(existing);
        }
        guard.insert(key.to_string(), Arc::clone(&index));
        #[cfg(test)]
        CATALOG_BUILD_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        return Arc::clone(guard.get(key).unwrap_or(&index));
    }
    index
}
