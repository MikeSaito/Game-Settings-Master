pub const QUALITY_INDEX_GROUPS: &[&str] = &[
    "ViewDistanceQuality",
    "AntiAliasingQuality",
    "ShadowQuality",
    "GlobalIlluminationQuality",
    "ReflectionQuality",
    "PostProcessQuality",
    "TextureQuality",
    "EffectsQuality",
    "FoliageQuality",
    "ShadingQuality",
    "LandscapeQuality",
    "CloudsQuality",
];

/// Render scale in percent — not a 0–4 index.
pub const RESOLUTION_SCALE_KEY: &str = "sg.ResolutionQuality";

/// Standard UE maximum: 0=Low, 1=Medium, 2=High, 3=Epic, 4=Cinematic.
pub const UE_DEFAULT_SCALABILITY_MAX: u32 = 4;

/// Quality indices (0–4+), not percentages or arbitrary sg.*.
pub fn is_scalability_quality_index(sg_key: &str) -> bool {
    if !sg_key.starts_with("sg.") {
        return false;
    }
    if sg_key == RESOLUTION_SCALE_KEY {
        return false;
    }
    let group = sg_key.strip_prefix("sg.").unwrap_or("");
    group.ends_with("Quality")
}

pub(crate) fn group_to_sg_key(group: &str) -> String {
    format!("sg.{group}")
}
