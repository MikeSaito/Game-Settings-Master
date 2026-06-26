use super::nvidia::nvidia_rtx_series;

/// Higher score = higher priority as the "primary gaming" GPU.
/// Discrete GPUs of any vendor always beat integrated ones.
pub(crate) fn gpu_priority(name: &str) -> i32 {
    let lower = name.to_lowercase();
    let is_nvidia =
        lower.contains("nvidia") || lower.contains("geforce") || lower.contains("quadro");
    let is_amd = lower.contains("amd") || lower.contains("radeon");
    let is_intel = lower.contains("intel");

    if is_nvidia {
        return if nvidia_rtx_series(&lower).is_some() {
            100
        } else {
            95
        };
    }
    if is_amd {
        let discrete = lower.contains("rx ")
            || lower.contains("rx")
                && lower
                    .split("rx")
                    .nth(1)
                    .map(|rest| rest.trim_start().starts_with(|c: char| c.is_ascii_digit()))
                    .unwrap_or(false)
            || lower.contains(" pro ")
            || lower.contains("frontier")
            || lower.contains("instinct");
        return if discrete { 90 } else { 40 };
    }
    if is_intel {
        return if lower.contains("arc") { 85 } else { 30 };
    }
    0
}

/// Pick the discrete gaming GPU, NOT the first match.
pub(crate) fn pick_primary_gpu(names: &[String]) -> String {
    names
        .iter()
        .max_by_key(|name| gpu_priority(name))
        .cloned()
        .unwrap_or_else(|| "Unknown GPU".to_string())
}
