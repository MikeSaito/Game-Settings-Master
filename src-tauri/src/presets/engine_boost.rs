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
        "ultra-low" => apply_ultra_low_performance(sys),
        "low" => apply_low_performance(sys),
        "ultra-high" => apply_ultra_boost(sys),
        _ => {}
    }
}

fn apply_ultra_low_performance(sys: &mut HashMap<String, String>) {
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
            ("r.Nanite", "0"),
            ("r.Streaming.MipBias", "2"),
            ("foliage.DensityScale", "0.0"),
            ("grass.DensityScale", "0.0"),
        ],
    );
}

fn apply_low_performance(sys: &mut HashMap<String, String>) {
    merge(
        sys,
        &[
            ("r.DynamicGlobalIlluminationMethod", "0"),
            ("r.Lumen.DiffuseIndirect.Allow", "0"),
            ("r.ViewDistanceScale", "0.5"),
            ("r.ShadowQuality", "1"),
            ("r.Shadow.Virtual.Enable", "0"),
            ("foliage.DensityScale", "0.5"),
            ("grass.DensityScale", "0.5"),
        ],
    );
}

fn apply_ultra_boost(sys: &mut HashMap<String, String>) {
    merge(
        sys,
        &[
            ("r.ViewDistanceScale", "1.25"),
            ("r.Lumen.ScreenProbeGather.DownsampleFactor", "16"),
        ],
    );
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
        apply_low_performance(&mut sys);
        assert_eq!(
            sys.get("r.Lumen.DiffuseIndirect.Allow").map(String::as_str),
            Some("0")
        );
        assert_eq!(
            sys.get("r.ViewDistanceScale").map(String::as_str),
            Some("0.5")
        );
        assert!(
            !sys.contains_key("r.Streaming.PoolSize"),
            "UE5 presets must let the engine size the streaming pool automatically"
        );
        assert!(
            !sys.contains_key("r.VolumetricFog"),
            "UE5 presets must preserve game fog/skyline defaults"
        );
    }

    #[test]
    fn ultra_high_minimal_boost_only() {
        let mut sys = HashMap::new();
        apply_ultra_boost(&mut sys);
        assert_eq!(
            sys.get("r.ViewDistanceScale").map(String::as_str),
            Some("1.25")
        );
        assert!(!sys.contains_key("sg.DefaultScalabilityLevel"));
        assert!(!sys.contains_key("r.Streaming.PoolSize"));
    }

    #[test]
    fn amd_disables_ngx_on_performance_tier() {
        let mut sys = HashMap::new();
        let gpu = GpuCapabilities::from_gpu_name("AMD Radeon RX 7800 XT");
        tune_engine_system_settings(&mut sys, "low", &gpu);
        assert_eq!(sys.get("r.NGX.Enable").map(String::as_str), Some("0"));
    }
}
