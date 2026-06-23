use crate::scalability::is_scalability_quality_index;

pub(crate) fn is_game_rendering_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    [
        "dlss",
        "xess",
        "fsr",
        "tsr",
        "raytracing",
        "ray_tracing",
        "lumen",
        "nanite",
        "upscal",
        "framegeneration",
        "frame_generation",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

pub(crate) fn infer_category(section: &str, key: &str) -> String {
    let lower = section.to_lowercase();
    if is_game_rendering_key(key) {
        return "Rendering".to_string();
    }
    if lower.starts_with("/script/") && !lower.contains("engine.gameusersettings") {
        return "GameSpecific".to_string();
    }
    if key.starts_with("sg.") {
        if key == "sg.ResolutionQuality" {
            return "Scalability".to_string();
        }
        if is_scalability_quality_index(key) {
            return "Scalability".to_string();
        }
        return "Scalability".to_string();
    }
    if key.starts_with("r.") {
        if key.to_lowercase().contains("shadow") {
            return "Shadows".to_string();
        }
        if key.to_lowercase().contains("stream") || key.to_lowercase().contains("anisotropy") {
            return "Textures".to_string();
        }
        if key.to_lowercase().contains("bloom")
            || key.to_lowercase().contains("motion")
            || key.to_lowercase().contains("ssr")
            || key.to_lowercase().contains("post")
            || key.to_lowercase().contains("dof")
            || key.to_lowercase().contains("tonemapper")
            || key.to_lowercase().contains("ambient")
        {
            return "PostProcess".to_string();
        }
        return "Rendering".to_string();
    }
    if key.to_lowercase().contains("audio") {
        return "Audio".to_string();
    }
    "Other".to_string()
}
