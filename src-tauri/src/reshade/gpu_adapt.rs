use crate::gpu::{detect_gpu, GpuCapabilities, GpuVendor};

#[derive(Debug, Clone)]
pub struct GpuAdaptResult {
    pub preset_id: String,
    pub reason: Option<String>,
}

/// Polaris RX 4xx/5xx (3-digit model). Does not match RX 5600 — trailing digit check.
fn contains_polaris_rx_model(lower: &str, model: &str) -> bool {
    let Some(idx) = lower.find(model) else {
        return false;
    };
    let after = idx + model.len();
    !lower.as_bytes().get(after).is_some_and(|b| b.is_ascii_digit())
}

fn is_legacy_amd_rx(lower: &str) -> bool {
    if lower.contains("rx 6")
        || lower.contains("rx 7")
        || lower.contains("rx 8")
        || lower.contains("rx 9")
    {
        return false;
    }
    for navi in ["rx 5500", "rx 5600", "rx 5700", "rx 5800", "rx 5900"] {
        if lower.contains(navi) {
            return false;
        }
    }

    const POLARIS: &[&str] = &[
        "rx 450", "rx 455", "rx 460", "rx 465", "rx 470", "rx 475", "rx 480", "rx 485", "rx 490",
        "rx 550", "rx 560", "rx 570", "rx 580", "rx 590",
    ];
    POLARIS.iter().any(|m| contains_polaris_rx_model(lower, m))
}

pub fn is_weak_gpu(gpu: &GpuCapabilities) -> bool {
    if gpu.vendor == GpuVendor::Intel {
        return true;
    }
    let lower = gpu.name.to_lowercase();

    if lower.contains("gtx")
        || lower.contains("geforce gt ")
        || lower.contains("geforce mx")
        || lower.contains("quadro m")
        || lower.contains("nvidia t")
    {
        return true;
    }

    if is_legacy_amd_rx(&lower) {
        return true;
    }
    if lower.contains("radeon r5") || lower.contains("radeon r7") || lower.contains("radeon r9") {
        return true;
    }

    false
}

pub fn adapt_preset_for_gpu(requested: &str) -> GpuAdaptResult {
    let gpu = detect_gpu();
    adapt_preset_with_gpu(requested, &gpu)
}

pub fn adapt_preset_with_gpu(requested: &str, gpu: &GpuCapabilities) -> GpuAdaptResult {
    if !is_weak_gpu(gpu) {
        return GpuAdaptResult {
            preset_id: requested.to_string(),
            reason: None,
        };
    }

    match requested {
        "cinematic" | "clarity" => GpuAdaptResult {
            preset_id: "performance".to_string(),
            reason: Some(format!(
                "Слабая GPU ({}) — выбран Performance вместо {}",
                gpu.name, requested
            )),
        },
        _ => GpuAdaptResult {
            preset_id: requested.to_string(),
            reason: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weak_gpu_downgrades_cinematic() {
        let gpu = GpuCapabilities {
            name: "Intel UHD 630".to_string(),
            vendor: GpuVendor::Intel,
            supports_dlss: false,
            supports_dlss_fg: false,
            supports_ray_tracing: false,
        };
        let r = adapt_preset_with_gpu("cinematic", &gpu);
        assert!(r.reason.is_some());
        assert_eq!(r.preset_id, "performance");
    }

    #[test]
    fn rtx_without_dlss_flag_is_not_weak() {
        let gpu = GpuCapabilities {
            name: "NVIDIA GeForce RTX 3070".to_string(),
            vendor: GpuVendor::Nvidia,
            supports_dlss: false,
            supports_dlss_fg: false,
            supports_ray_tracing: false,
        };
        assert!(!is_weak_gpu(&gpu));
        let r = adapt_preset_with_gpu("cinematic", &gpu);
        assert_eq!(r.preset_id, "cinematic");
        assert!(r.reason.is_none());
    }

    #[test]
    fn gtx_is_weak() {
        let gpu = GpuCapabilities {
            name: "NVIDIA GeForce GTX 1050 Ti".to_string(),
            vendor: GpuVendor::Nvidia,
            supports_dlss: false,
            supports_dlss_fg: false,
            supports_ray_tracing: false,
        };
        assert!(is_weak_gpu(&gpu));
    }

    #[test]
    fn rx_5600_is_not_weak() {
        let gpu = GpuCapabilities {
            name: "AMD Radeon RX 5600 XT".to_string(),
            vendor: GpuVendor::Amd,
            supports_dlss: false,
            supports_dlss_fg: false,
            supports_ray_tracing: false,
        };
        assert!(!is_weak_gpu(&gpu));
    }

    #[test]
    fn rx_580_polaris_is_weak() {
        let gpu = GpuCapabilities {
            name: "AMD Radeon RX 580".to_string(),
            vendor: GpuVendor::Amd,
            supports_dlss: false,
            supports_dlss_fg: false,
            supports_ray_tracing: false,
        };
        assert!(is_weak_gpu(&gpu));
    }
}
