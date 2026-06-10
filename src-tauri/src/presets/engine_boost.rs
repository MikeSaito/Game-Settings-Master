use crate::gpu::{GpuCapabilities, GpuVendor};
use std::collections::HashMap;

/// Патчит `[SystemSettings]` поверх базового Engine.ini.
///
/// Шкалы не смешивать:
/// - `sg.*Quality` (кроме ResolutionQuality) — индекс 0..4 (Low..Epic)
/// - `sg.ResolutionQuality` — проценты (45–100+)
/// - `r.ShadowQuality` — движковая шкала ~0–5
/// - `r.PostProcessAAQuality` — 0–6
/// - `r.AmbientOcclusionMaxQuality` — 0–100 (%)
/// - `r.ViewDistanceScale` — множитель ~0.25–2.0
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
    sys.insert("r.RayTracing".to_string(), "0".to_string());
    sys.insert("r.Lumen.HardwareRayTracing".to_string(), "0".to_string());

    if gpu.vendor == GpuVendor::Amd || gpu.vendor == GpuVendor::Intel {
        sys.insert("r.NGX.Enable".to_string(), "0".to_string());
    }

    match preset_id {
        "ultra-low" => apply_ultra_low_performance(sys, gpu),
        "low" => apply_low_performance(sys, gpu),
        "medium" => apply_medium_performance(sys, gpu),
        "high" => apply_high_boost(sys, gpu),
        "ultra-high" => apply_ultra_boost(sys, gpu),
        _ => {}
    }
}

fn apply_ultra_low_performance(sys: &mut HashMap<String, String>, gpu: &GpuCapabilities) {
    let pool = performance_pool_mb(gpu, "ultra-low");
    merge(
        sys,
        &[
            ("r.DynamicGlobalIlluminationMethod", "0"),
            ("r.ReflectionMethod", "0"),
            ("r.Lumen.DiffuseIndirect.Allow", "0"),
            ("r.Lumen.Reflections.Allow", "0"),
            ("r.Lumen.HardwareRayTracing", "0"),
            ("r.Lumen.TraceMeshSDFs", "0"),
            ("r.Lumen.ScreenProbeGather.RadianceCache", "0"),
            ("r.ViewDistanceScale", "0.35"),
            ("r.CastShadows", "0"),
            ("r.ShadowQuality", "0"),
            ("r.Shadow.Virtual.Enable", "0"),
            ("r.AsyncCompute", "0"),
            ("r.VolumetricFog", "0"),
            ("r.PostProcessAAQuality", "0"),
            ("sg.DefaultScalabilityLevel", "0"),
            ("foliage.DensityScale", "0.0"),
            ("grass.DensityScale", "0.0"),
            ("r.OneFrameThreadLag", "1"),
        ],
    );
    sys.insert("r.Streaming.PoolSize".to_string(), pool);
    if gpu.vendor == GpuVendor::Amd || gpu.vendor == GpuVendor::Intel {
        sys.insert("r.NGX.Enable".to_string(), "0".to_string());
    }
}

fn apply_low_performance(sys: &mut HashMap<String, String>, gpu: &GpuCapabilities) {
    let pool = performance_pool_mb(gpu, "low");
    merge(
        sys,
        &[
            ("r.DynamicGlobalIlluminationMethod", "0"),
            ("r.ReflectionMethod", "0"),
            ("r.Lumen.DiffuseIndirect.Allow", "0"),
            ("r.Lumen.Reflections.Allow", "0"),
            ("r.Lumen.HardwareRayTracing", "0"),
            ("r.ViewDistanceScale", "0.6"),
            ("r.ShadowQuality", "1"),
            ("r.Shadow.Virtual.Enable", "0"),
            ("r.AsyncCompute", "0"),
            ("r.VolumetricFog", "0"),
            ("sg.DefaultScalabilityLevel", "1"),
            ("foliage.DensityScale", "0.5"),
            ("grass.DensityScale", "0.5"),
            ("r.Shadow.CSM.MaxCascades", "2"),
            ("r.Shadow.MaxResolution", "512"),
        ],
    );
    sys.insert("r.Streaming.PoolSize".to_string(), pool);
}

fn apply_medium_performance(sys: &mut HashMap<String, String>, gpu: &GpuCapabilities) {
    let pool = performance_pool_mb(gpu, "medium");
    merge(
        sys,
        &[
            ("r.DynamicGlobalIlluminationMethod", "1"),
            ("r.ReflectionMethod", "0"),
            ("r.Lumen.DiffuseIndirect.Allow", "1"),
            ("r.Lumen.Reflections.Allow", "0"),
            ("r.ViewDistanceScale", "1.0"),
            ("r.Shadow.Virtual.Enable", "1"),
            ("sg.DefaultScalabilityLevel", "2"),
            ("foliage.DensityScale", "0.75"),
            ("grass.DensityScale", "0.75"),
            ("r.Shadow.CSM.MaxCascades", "3"),
            ("r.Shadow.MaxResolution", "1024"),
        ],
    );
    sys.insert("r.Streaming.PoolSize".to_string(), pool);
}

fn performance_pool_mb(gpu: &GpuCapabilities, tier: &str) -> String {
    let base: u32 = streaming_pool_mb(gpu, "high").parse().unwrap_or(3072);
    let mb = match tier {
        "ultra-low" => (base / 4).max(512),
        "low" => (base / 2).max(1024),
        "medium" => (base * 3 / 4).max(2048),
        _ => base,
    };
    mb.to_string()
}

fn apply_high_boost(sys: &mut HashMap<String, String>, gpu: &GpuCapabilities) {
    let pool = streaming_pool_mb(gpu, "high");
    merge(
        sys,
        &[
            // Lumen (software) — легче чем Ultra
            ("r.DynamicGlobalIlluminationMethod", "1"),
            ("r.ReflectionMethod", "1"),
            ("r.Lumen.DiffuseIndirect.Allow", "1"),
            ("r.Lumen.Reflections.Allow", "1"),
            ("r.Lumen.TraceMeshSDFs", "1"),
            ("r.Lumen.ScreenProbeGather.RadianceCache", "1"),
            ("r.Lumen.ScreenProbeGather.DownsampleFactor", "16"),
            ("r.Nanite", "1"),
            // Дистанция / тени (r.* шкалы, не sg.*)
            ("r.ViewDistanceScale", "1.55"),
            ("r.CastShadows", "1"),
            ("r.ShadowQuality", "3"),
            ("r.Shadow.Virtual.Enable", "1"),
            ("r.Shadow.CSM.MaxCascades", "4"),
            ("r.Shadow.MaxResolution", "2048"),
            ("r.ContactShadows", "1"),
            ("r.DistanceFieldShadowing", "1"),
            // Пост и отражения
            ("r.AmbientOcclusionLevels", "2"),
            ("r.AmbientOcclusionMaxQuality", "60"),
            ("r.SSR.Quality", "2"),
            ("r.ScreenSpaceReflections", "1"),
            ("r.ReflectionEnvironment", "1"),
            ("r.PostProcessAAQuality", "4"),
            ("r.BloomQuality", "3"),
            ("r.LensFlareQuality", "1"),
            ("r.DepthOfFieldQuality", "1"),
            // Текстуры / стриминг
            ("r.TextureStreaming", "1"),
            ("r.Streaming.LimitPoolSizeToVRAM", "1"),
            ("r.Streaming.MipBias", "0"),
            ("r.MaxAnisotropy", "8"),
            ("r.Streaming.FramesForFullUpdate", "4"),
            // Якорь меню: High = индекс 3
            ("sg.DefaultScalabilityLevel", "3"),
            // Отзывчивость
            ("r.OneFrameThreadLag", "0"),
            ("r.Reflex.Enable", "1"),
            ("r.Reflex.Boost", "1"),
            ("r.RHICmdUseThread", "1"),
            ("r.ParallelRendering", "1"),
            ("r.AsyncCompute", "1"),
            ("r.Tonemapper.Sharpen", "0.25"),
            ("foliage.DensityScale", "0.92"),
            ("grass.DensityScale", "0.92"),
        ],
    );
    sys.insert("r.Streaming.PoolSize".to_string(), pool);
}

fn apply_ultra_boost(sys: &mut HashMap<String, String>, gpu: &GpuCapabilities) {
    let pool = streaming_pool_mb(gpu, "ultra-high");
    merge(
        sys,
        &[
            ("r.DynamicGlobalIlluminationMethod", "1"),
            ("r.ReflectionMethod", "1"),
            ("r.Lumen.DiffuseIndirect.Allow", "1"),
            ("r.Lumen.Reflections.Allow", "1"),
            ("r.Lumen.TraceMeshSDFs", "1"),
            ("r.Lumen.ScreenProbeGather.RadianceCache", "1"),
            ("r.Lumen.ScreenProbeGather.DownsampleFactor", "8"),
            ("r.Nanite", "1"),
            ("r.Nanite.MaterialVisibility", "1"),
            ("r.ViewDistanceScale", "1.85"),
            ("r.CastShadows", "1"),
            ("r.ShadowQuality", "5"),
            ("r.Shadow.Virtual.Enable", "1"),
            ("r.Shadow.CSM.MaxCascades", "4"),
            ("r.Shadow.MaxResolution", "4096"),
            ("r.ContactShadows", "1"),
            ("r.DistanceFieldShadowing", "1"),
            ("r.GenerateMeshDistanceFields", "1"),
            ("r.AmbientOcclusionLevels", "3"),
            ("r.AmbientOcclusionMaxQuality", "85"),
            ("r.SSR.Quality", "4"),
            ("r.ScreenSpaceReflections", "1"),
            ("r.ReflectionEnvironment", "1"),
            ("r.VolumetricFog", "1"),
            ("r.VolumetricFog.GridPixelSize", "6"),
            ("r.PostProcessAAQuality", "5"),
            ("r.BloomQuality", "5"),
            ("r.LensFlareQuality", "2"),
            ("r.DepthOfFieldQuality", "1"),
            ("r.TextureStreaming", "1"),
            ("r.Streaming.LimitPoolSizeToVRAM", "1"),
            ("r.Streaming.MipBias", "0"),
            ("r.MaxAnisotropy", "16"),
            ("r.Streaming.FramesForFullUpdate", "6"),
            ("r.Streaming.Boost", "1"),
            // Якорь меню: Epic = индекс 4
            ("sg.DefaultScalabilityLevel", "4"),
            ("r.OneFrameThreadLag", "0"),
            ("r.Reflex.Enable", "1"),
            ("r.Reflex.Boost", "1"),
            ("r.PSOPrecache.Mode", "1"),
            ("r.RHICmdUseThread", "1"),
            ("r.RHICmdUseParallelAlgorithms", "1"),
            ("r.ParallelInitViews", "1"),
            ("r.ParallelRendering", "1"),
            ("r.ParallelMeshDrawCommands", "1"),
            ("r.AsyncCompute", "1"),
            ("r.D3D12.ExecuteContextInParallel", "1"),
            ("r.IO.UseDirectStorage", "1"),
            ("s.AsyncLoadingThreadEnabled", "1"),
            ("s.IoDispatcherBufferSizeMB", "64"),
            ("r.Tonemapper.Sharpen", "0.32"),
            ("r.FilmGrainIntensity", "0.45"),
            ("foliage.DensityScale", "1.0"),
            ("grass.DensityScale", "1.0"),
            ("r.HZBOcclusion", "1"),
            ("r.CustomDepth", "3"),
        ],
    );
    sys.insert("r.Streaming.PoolSize".to_string(), pool);
}

fn merge(sys: &mut HashMap<String, String>, pairs: &[(&str, &str)]) {
    for (k, v) in pairs {
        sys.insert((*k).to_string(), (*v).to_string());
    }
}

/// Пул текстур (МБ) — эвристика по названию GPU, не путать с sg.*.
fn streaming_pool_mb(gpu: &GpuCapabilities, tier: &str) -> String {
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

    let mb = match (tier, vram_tier) {
        ("high", 3) => 5120,
        ("high", 2) => 4096,
        ("high", 1) => 3072,
        ("high", _) => 3584,
        ("ultra-high", 3) => 7000,
        ("ultra-high", 2) => 5500,
        ("ultra-high", 1) => 4096,
        ("ultra-high", _) => 5120,
        _ => 4096,
    };
    mb.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_uses_sg_anchor_three_not_four() {
        let mut sys = HashMap::new();
        let gpu = GpuCapabilities::from_gpu_name("NVIDIA GeForce RTX 4060 Ti");
        apply_high_boost(&mut sys, &gpu);
        assert_eq!(
            sys.get("sg.DefaultScalabilityLevel").map(String::as_str),
            Some("3")
        );
        assert_eq!(sys.get("r.ShadowQuality").map(String::as_str), Some("3"));
        assert_eq!(
            sys.get("r.ViewDistanceScale").map(String::as_str),
            Some("1.55")
        );
    }

    #[test]
    fn ultra_uses_sg_anchor_four() {
        let mut sys = HashMap::new();
        let gpu = GpuCapabilities::from_gpu_name("NVIDIA GeForce RTX 4070");
        apply_ultra_boost(&mut sys, &gpu);
        assert_eq!(
            sys.get("sg.DefaultScalabilityLevel").map(String::as_str),
            Some("4")
        );
        assert_eq!(sys.get("r.ShadowQuality").map(String::as_str), Some("5"));
    }

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
            Some("0.6")
        );
    }

    #[test]
    fn amd_disables_ngx() {
        let mut sys = HashMap::new();
        let gpu = GpuCapabilities::from_gpu_name("AMD Radeon RX 7800 XT");
        tune_engine_system_settings(&mut sys, "high", &gpu);
        assert_eq!(sys.get("r.NGX.Enable").map(String::as_str), Some("0"));
    }
}
