use crate::gpu::{GpuCapabilities, GpuVendor};
use std::collections::HashMap;

/// Минимальные патчи `[SystemSettings]` только для ultra-low / low.
/// Medium+ управляется sg.* — r.* здесь не задаём, чтобы не конфликтовать с меню.
pub fn tune_engine_system_settings(
    sys: &mut HashMap<String, String>,
    preset_id: &str,
    gpu: &GpuCapabilities,
) {
    sys.insert(
        "r.DefaultFeature.MotionBlur".to_string(),
        "False".to_string(),
    );
    sys.insert("r.MotionBlurQuality".to_string(), "0".to_string());
    sys.insert("r.MotionBlur.Scale".to_string(), "0".to_string());
    sys.insert("r.DepthOfFieldQuality".to_string(), "0".to_string());
    sys.insert("r.RayTracing".to_string(), "0".to_string());
    sys.insert("r.Lumen.HardwareRayTracing".to_string(), "0".to_string());

    if gpu.vendor == GpuVendor::Amd || gpu.vendor == GpuVendor::Intel {
        sys.insert("r.NGX.Enable".to_string(), "0".to_string());
    }

    match preset_id {
        "ultra-low" => apply_ultra_low_performance(sys, gpu),
        "low" => apply_low_performance(sys, gpu),
        "ultra-high" => apply_ultra_boost(sys, gpu),
        _ => {}
    }
}

fn apply_ultra_low_performance(sys: &mut HashMap<String, String>, gpu: &GpuCapabilities) {
    merge(
        sys,
        &[
            ("r.DynamicGlobalIlluminationMethod", "0"),
            ("r.ReflectionMethod", "0"),
            ("r.Lumen.DiffuseIndirect.Allow", "0"),
            ("r.Lumen.Reflections.Allow", "0"),
            ("r.ViewDistanceScale", "0.25"),
            ("r.CastShadows", "0"),
            ("r.ShadowQuality", "0"),
            ("r.Shadow.Virtual.Enable", "0"),
            ("r.VolumetricFog", "0"),
            ("r.Nanite", "0"),
            ("r.Streaming.MipBias", "2"),
            ("foliage.DensityScale", "0.0"),
            ("grass.DensityScale", "0.0"),
        ],
    );
    sys.insert(
        "r.Streaming.PoolSize".to_string(),
        performance_pool_mb(gpu, "ultra-low"),
    );
}

fn apply_low_performance(sys: &mut HashMap<String, String>, gpu: &GpuCapabilities) {
    merge(
        sys,
        &[
            ("r.DynamicGlobalIlluminationMethod", "0"),
            ("r.Lumen.DiffuseIndirect.Allow", "0"),
            ("r.ViewDistanceScale", "0.5"),
            ("r.ShadowQuality", "1"),
            ("r.Shadow.Virtual.Enable", "0"),
            ("r.VolumetricFog", "0"),
            ("foliage.DensityScale", "0.5"),
            ("grass.DensityScale", "0.5"),
        ],
    );
    sys.insert(
        "r.Streaming.PoolSize".to_string(),
        performance_pool_mb(gpu, "low"),
    );
}

fn apply_ultra_boost(sys: &mut HashMap<String, String>, gpu: &GpuCapabilities) {
    merge(
        sys,
        &[
            ("r.ViewDistanceScale", "1.25"),
            ("r.Lumen.ScreenProbeGather.DownsampleFactor", "16"),
        ],
    );
    sys.insert(
        "r.Streaming.PoolSize".to_string(),
        streaming_pool_mb(gpu),
    );
}

fn streaming_pool_mb(gpu: &GpuCapabilities) -> String {
    let name = gpu.name.to_lowercase();
    let vram_tier = if name.contains("4090")
        || name.contains("4080")
        || name.contains("7900 xtx")
        || name.contains("5090")
        || name.contains("5080")
    {
        3
    } else if name.contains("4070")
        || name.contains("4060 ti")
        || name.contains("3070")
        || name.contains("7800")
        || name.contains("6900")
    {
        2
    } else if name.contains("4060")
        || name.contains("3060")
        || name.contains("2080")
        || name.contains("7700")
        || name.contains("6700")
    {
        1
    } else {
        2
    };
    let mb = match vram_tier {
        3 => 7000,
        2 => 5500,
        1 => 4096,
        _ => 5120,
    };
    mb.to_string()
}

fn performance_pool_mb(_gpu: &GpuCapabilities, tier: &str) -> String {
    let base: u32 = 4096;
    let mb = match tier {
        "ultra-low" => (base / 4).max(512),
        "low" => (base / 2).max(1024),
        _ => base,
    };
    mb.to_string()
}

fn merge(sys: &mut HashMap<String, String>, pairs: &[(&str, &str)]) {
    for (k, v) in pairs {
        sys.insert((*k).to_string(), (*v).to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_performance_disables_lumen() {
        let mut sys = HashMap::new();
        let gpu = GpuCapabilities::from_gpu_name("NVIDIA GeForce RTX 4060 Ti");
        apply_low_performance(&mut sys, &gpu);
        assert_eq!(
            sys.get("r.Lumen.DiffuseIndirect.Allow").map(String::as_str),
            Some("0")
        );
        assert_eq!(
            sys.get("r.ViewDistanceScale").map(String::as_str),
            Some("0.5")
        );
    }

    #[test]
    fn ultra_high_minimal_boost_only() {
        let mut sys = HashMap::new();
        let gpu = GpuCapabilities::from_gpu_name("NVIDIA GeForce RTX 4070");
        apply_ultra_boost(&mut sys, &gpu);
        assert_eq!(
            sys.get("r.ViewDistanceScale").map(String::as_str),
            Some("1.25")
        );
        assert!(!sys.contains_key("sg.DefaultScalabilityLevel"));
    }

    #[test]
    fn amd_disables_ngx_on_performance_tier() {
        let mut sys = HashMap::new();
        let gpu = GpuCapabilities::from_gpu_name("AMD Radeon RX 7800 XT");
        tune_engine_system_settings(&mut sys, "low", &gpu);
        assert_eq!(sys.get("r.NGX.Enable").map(String::as_str), Some("0"));
    }
}
