const UE5_ONLY_SG_KEYS: &[&str] = &[
    "sg.GlobalIlluminationQuality",
    "sg.ReflectionQuality",
    "sg.ShadingQuality",
    "sg.LandscapeQuality",
    "sg.CloudsQuality",
];

const UE5_ONLY_CVAR_KEYS: &[&str] = &[
    "r.Nanite",
    "r.Lumen.DiffuseIndirect.Allow",
    "r.Lumen.Reflections.Allow",
    "r.Lumen.Reflections.Quality",
    "r.Lumen.ScreenProbeGather.ScreenTraces",
    "r.VolumetricCloud",
];

const HIDDEN_UE_MANUAL_KEYS: &[&str] = &[
    "DLSSQualityMode",
    "ResolutionScaleDLSS",
    "BenchmarkResolutionX",
    "BenchmarkResolutionY",
    "bUseDesiredScreenHeight",
    "bUseDesiredScreenWidth",
    "DesiredScreenHeight",
    "DesiredScreenWidth",
    "InstallGUID",
    "LastCPUBenchmarkResult",
    "LastCPUBenchmarkSteps",
    "LastGPUBenchmarkMultiplier",
    "LastGPUBenchmarkResult",
    "LastGPUBenchmarkSteps",
    "LastRecommendedScreenHeight",
    "LastRecommendedScreenWidth",
    "RunNumber",
    "Version",
    "WindowPosX",
    "WindowPosY",
    "r.AsyncCompute",
    "r.D3D12.ExecuteContextInParallel",
    "r.D3D12.UseAllowTearing",
    "r.FinishCurrentFrame",
    "r.Fog.HZBAsyncCompute",
    "r.IO.UseDirectStorage",
    "r.OneFrameThreadLag",
    "r.RHICmdBypass",
    "r.RHICmdUseParallelAlgorithms",
    "r.RHICmdUseThread",
    "r.SceneDepthHZBAsyncCompute",
    "r.SkyAtmosphereAsyncCompute",
    "r.Streaming.LimitPoolSizeToVRAM",
    "r.Streaming.PoolSize",
    "r.Streaming.UseFixedPoolSize",
];

pub(crate) fn is_standard_ue_cvar_key(key: &str) -> bool {
    key.starts_with("r.")
        || key.starts_with("sg.")
        || key.starts_with("fx.")
        || key.starts_with("t.")
}

pub(crate) fn is_ue5_only_catalog_key(key: &str) -> bool {
    UE5_ONLY_SG_KEYS.contains(&key) || UE5_ONLY_CVAR_KEYS.contains(&key)
}

pub(crate) fn is_hidden_ue_manual_key(key: &str) -> bool {
    HIDDEN_UE_MANUAL_KEYS
        .iter()
        .any(|hidden| key.eq_ignore_ascii_case(hidden))
}
