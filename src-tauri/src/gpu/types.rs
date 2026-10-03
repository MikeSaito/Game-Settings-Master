use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct GpuCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dedicated_memory_mb: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared_memory_mb: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ray_tracing_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_warning: Option<String>,
    pub name: String,
    pub vendor: GpuVendor,
    /// DLSS / DLAA — GeForce RTX 20 series and newer (Tensor Cores).
    pub supports_dlss: bool,
    /// DLSS Frame Generation — RTX 40 series and newer.
    pub supports_dlss_fg: bool,
    /// Hardware ray tracing in UE — GeForce RTX 20+.
    pub supports_ray_tracing: bool,
}

impl GpuCapabilities {
    pub fn from_gpu_name(name: &str) -> Self {
        let lower = name.to_lowercase();
        let vendor = super::nvidia::detect_vendor(&lower);

        let rtx_series = super::nvidia::nvidia_rtx_series(&lower);
        let supports_dlss = rtx_series.is_some();
        let supports_dlss_fg = rtx_series.is_some_and(|s| s >= 40);
        let supports_ray_tracing = supports_dlss;

        Self {
            adapter_id: None,
            dedicated_memory_mb: None,
            shared_memory_mb: None,
            ray_tracing_status: Some("unknown".into()),
            selection_warning: None,
            name: name.trim().to_string(),
            vendor,
            supports_dlss,
            supports_dlss_fg,
            supports_ray_tracing,
        }
    }
}
