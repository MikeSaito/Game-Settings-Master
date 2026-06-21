"""One-off generator for committed UE reference fixtures."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / "ue-reference" / "fixtures"


def write_engine(path: Path, extra_keys: list[tuple[str, str]]) -> None:
    lines = ["[/Script/Engine.Engine]", "bUseFixedFrameRate=False", "", "[SystemSettings]"]
    base = [
        ("r.ViewDistanceScale", "1.0"),
        ("r.MaterialQualityLevel", "1"),
        ("r.DetailMode", "2"),
        ("r.ShadowQuality", "5"),
        ("r.Shadow.MaxResolution", "2048"),
        ("r.MotionBlurQuality", "4"),
        ("r.BloomQuality", "5"),
        ("r.DepthOfFieldQuality", "2"),
        ("r.AmbientOcclusionLevels", "3"),
        ("r.SSR.Quality", "3"),
        ("r.SSR.MaxRoughness", "0.6"),
        ("r.TemporalAAQuality", "2"),
        ("r.Tonemapper.Sharpen", "0"),
        ("r.DefaultFeature.Bloom", "1"),
        ("r.DefaultFeature.AmbientOcclusion", "1"),
        ("r.DefaultFeature.AutoExposure", "1"),
        ("r.DefaultFeature.MotionBlur", "1"),
        ("r.Streaming.PoolSize", "1000"),
        ("r.MaxAnisotropy", "8"),
        ("r.MipMapLODBias", "0"),
        ("r.EarlyZPass", "3"),
        ("r.GBufferFormat", "1"),
        ("r.AllowStaticLighting", "1"),
        ("r.ForwardShading", "0"),
        ("r.VolumetricFog", "1"),
        ("r.Atmosphere", "1"),
        ("r.ReflectionEnvironment", "1"),
        ("r.SeparateTranslucency", "1"),
        ("r.CustomDepth", "3"),
        ("r.WireframeCullThreshold", "5.0"),
        ("r.HZBOcclusion", "1"),
        ("r.DistanceFieldShadowing", "1"),
        ("r.LightFunctionQuality", "2"),
        ("r.SkyQuality", "1"),
        ("r.Fog", "1"),
        ("r.ContactShadows", "1"),
        ("r.SceneColorFringeQuality", "0"),
        ("r.EyeAdaptationQuality", "2"),
        ("r.PostProcessAAQuality", "6"),
        ("r.MobileHDR", "1"),
    ]
    for key, value in base + extra_keys:
        lines.append(f"{key}={value}")
    lines += ["", "[ConsoleVariables]"]
    console = [
        ("r.OneFrameThreadLag", "1"),
        ("r.FinishCurrentFrame", "0"),
        ("r.RHICmdBypass", "0"),
        ("r.AsyncCompute", "1"),
        ("r.D3D12.UseAllowTearing", "1"),
        ("r.Vulkan.UseRealUBs", "1"),
        ("r.SceneDepthHZBAsyncCompute", "1"),
        ("r.Fog.HZBAsyncCompute", "1"),
        ("r.SkyAtmosphereAsyncCompute", "1"),
        ("r.Streaming.UseFixedPoolSize", "0"),
        ("r.Streaming.LimitPoolSizeToVRAM", "0"),
        ("r.Streaming.Boost", "1"),
        ("r.Streaming.MaxEffectiveScreenSize", "0"),
        ("r.TextureStreaming", "1"),
        ("r.ParticleLODBias", "0"),
        ("r.SkeletalMeshLODBias", "0"),
        ("r.StaticMeshLODDistanceScale", "1"),
        ("r.TranslucencyLightingVolumeDim", "64"),
        ("r.LensFlareQuality", "2"),
        ("r.LightShaftQuality", "1"),
        ("r.LightShaftOcclusion", "1"),
        ("r.ScreenSpaceReflectionQuality", "50"),
        ("r.SSAOQuality", "50"),
        ("r.AmbientOcclusionRadiusScale", "1.0"),
        ("r.MotionBlur.Max", "0"),
        ("r.MotionBlur.Amount", "0.5"),
        ("r.BloomIntensity", "1.0"),
        ("r.Shadow.CSM.MaxCascades", "4"),
        ("r.Shadow.CSM.TransitionScale", "1.0"),
        ("r.Shadow.PreShadowResolutionFactor", "1.0"),
        ("r.Shadow.RadiusThreshold", "0.03"),
        ("r.Shadow.DistanceScale", "1.0"),
        ("r.DFShadowQuality", "1"),
        ("r.ClearCoatNormal", "0"),
        ("r.SubsurfaceScattering", "1"),
        ("r.IndirectLightingCache", "1"),
        ("r.AllowOcclusionQueries", "1"),
        ("r.DisableDistortion", "0"),
        ("r.BasePassVelocity", "1"),
        ("r.VelocityOutputPass", "1"),
        ("r.MaxQualityMode", "0"),
    ]
    for key, value in console:
        lines.append(f"{key}={value}")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_scal(path: Path, ue5: bool) -> None:
    lines = ["[ScalabilityGroups]"]
    groups = [
        ("sg.ResolutionQuality", "100"),
        ("sg.ViewDistanceQuality", "3"),
        ("sg.AntiAliasingQuality", "3"),
        ("sg.ShadowQuality", "3"),
        ("sg.GlobalIlluminationQuality", "3"),
        ("sg.ReflectionQuality", "3"),
        ("sg.PostProcessQuality", "3"),
        ("sg.TextureQuality", "3"),
        ("sg.EffectsQuality", "3"),
        ("sg.FoliageQuality", "3"),
        ("sg.ShadingQuality", "3"),
    ]
    if ue5:
        groups += [("sg.LandscapeQuality", "3"), ("sg.CloudsQuality", "3")]
    for key, value in groups:
        lines.append(f"{key}={value}")

    tiers = [
        "ViewDistanceQuality",
        "ShadowQuality",
        "PostProcessQuality",
        "TextureQuality",
        "EffectsQuality",
    ]
    for tier in tiers:
        for i in range(5):
            lines += ["", f"[{tier}@{i}]"]
            if tier == "ShadowQuality":
                lines += [
                    f"r.ShadowQuality={i}",
                    f"r.Shadow.MaxResolution={512 * (i + 1)}",
                ]
            elif tier == "PostProcessQuality":
                lines += [
                    f"r.MotionBlurQuality={i}",
                    f"r.BloomQuality={i + 1}",
                    f"r.DepthOfFieldQuality={i}",
                ]
            elif tier == "TextureQuality":
                lines += [
                    f"r.MaxAnisotropy={4 + i}",
                    f"r.Streaming.PoolSize={700 + i * 100}",
                ]
            else:
                lines += [f"r.ViewDistanceScale={0.4 + i * 0.15:.2f}"]

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


if __name__ == "__main__":
    ue5_extra = [
        ("r.Nanite", "1"),
        ("r.Nanite.MaxPixelsPerEdge", "1"),
        ("r.Lumen.DiffuseIndirect.Allow", "1"),
        ("r.Lumen.Reflections.Allow", "1"),
        ("r.Lumen.Reflections.Quality", "1"),
        ("r.VolumetricCloud", "1"),
        ("r.VirtualShadowMaps", "1"),
        ("r.Shadow.Virtual.Enable", "1"),
        ("r.RayTracing", "0"),
        ("r.RayTracing.Shadows", "0"),
        ("r.DynamicGlobalIlluminationMethod", "1"),
        ("r.ReflectionMethod", "1"),
        ("r.HeterogeneousVolumes", "0"),
        ("r.HairStrands", "0"),
    ]
    write_engine(ROOT / "UE_4.27" / "BaseEngine.ini", [])
    write_engine(ROOT / "UE_5.4" / "BaseEngine.ini", ue5_extra)
    write_scal(ROOT / "UE_4.27" / "BaseScalability.ini", False)
    write_scal(ROOT / "UE_5.4" / "BaseScalability.ini", True)
    print("fixtures ok")
