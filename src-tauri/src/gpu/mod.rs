mod enumerate;
mod nvidia;
mod priority;
pub(crate) mod types;

use std::sync::OnceLock;

pub use types::GpuCapabilities;

use enumerate::enumerate_gpu_names;
use priority::pick_primary_gpu;

static GPU_CACHE: OnceLock<GpuCapabilities> = OnceLock::new();

pub fn detect_gpu() -> GpuCapabilities {
    GPU_CACHE
        .get_or_init(|| {
            let names = enumerate_gpu_names();
            let primary = pick_primary_gpu(&names);
            GpuCapabilities::from_gpu_name(&primary)
        })
        .clone()
}

#[cfg(test)]
#[path = "gpu_tests.rs"]
mod tests;
