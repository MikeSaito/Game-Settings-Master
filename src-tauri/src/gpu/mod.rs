use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static GPU_CACHE: OnceLock<GpuCapabilities> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuCapabilities {
    pub name: String,
    pub vendor: GpuVendor,
    /// DLSS / DLAA — GeForce RTX 20-й серии и новее (Tensor Cores).
    pub supports_dlss: bool,
    /// DLSS Frame Generation — RTX 40-й серии и новее.
    pub supports_dlss_fg: bool,
    /// Аппаратная трассировка лучей в UE — GeForce RTX 20+.
    pub supports_ray_tracing: bool,
}

impl GpuCapabilities {
    pub fn from_gpu_name(name: &str) -> Self {
        let lower = name.to_lowercase();
        let vendor = detect_vendor(&lower);

        let rtx_series = nvidia_rtx_series(&lower);
        let supports_dlss = rtx_series.is_some();
        let supports_dlss_fg = rtx_series.is_some_and(|s| s >= 40);
        let supports_ray_tracing = supports_dlss;

        Self {
            name: name.trim().to_string(),
            vendor,
            supports_dlss,
            supports_dlss_fg,
            supports_ray_tracing,
        }
    }
}

fn detect_vendor(lower: &str) -> GpuVendor {
    if lower.contains("nvidia") || lower.contains("geforce") || lower.contains("quadro rtx") {
        GpuVendor::Nvidia
    } else if lower.contains("amd") || lower.contains("radeon") {
        GpuVendor::Amd
    } else if lower.contains("intel") {
        GpuVendor::Intel
    } else {
        GpuVendor::Unknown
    }
}

/// RTX 2060 → 20, RTX 4090 → 40, RTX 5090 → 50. GTX и старые Quadro без RTX → None.
fn nvidia_rtx_series(lower: &str) -> Option<u8> {
    if lower.contains("gtx") || lower.contains("gt ") || lower.contains("mx ") {
        return None;
    }

    let patterns = [
        r"rtx\s*(\d{2})\d{2}",
        r"geforce\s+rtx\s*(\d{2})\d{2}",
        r"quadro\s+rtx\s*(\d{2})\d{2}",
    ];

    for pat in patterns {
        let re = regex::Regex::new(pat).ok()?;
        if let Some(cap) = re.captures(lower) {
            if let Some(series) = cap.get(1).and_then(|m| m.as_str().parse::<u8>().ok()) {
                if (20..=90).contains(&series) {
                    return Some(series);
                }
            }
        }
    }

    None
}

pub fn detect_gpu() -> GpuCapabilities {
    GPU_CACHE
        .get_or_init(|| {
            let names = enumerate_gpu_names();
            let primary = pick_primary_gpu(&names);
            GpuCapabilities::from_gpu_name(&primary)
        })
        .clone()
}

fn enumerate_gpu_names() -> Vec<String> {
    #[cfg(windows)]
    {
        if let Some(names) = enumerate_gpu_from_registry() {
            if !names.is_empty() {
                return names;
            }
        }
    }

    vec!["Unknown GPU".to_string()]
}

#[cfg(windows)]
fn enumerate_gpu_from_registry() -> Option<Vec<String>> {
    use winreg::enums::*;
    use winreg::RegKey;

    const SKIP: &[&str] = &[
        "microsoft basic",
        "remote",
        "parsec",
        "virtual",
        "vmware",
        "citrix",
        "meta virtual",
        "spice",
        "qxl",
    ];

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let class_key = hklm
        .open_subkey(
            r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}",
        )
        .ok()?;

    let mut names = Vec::new();
    for i in 0..32 {
        let sub = class_key.open_subkey(format!("{i:04}")).ok()?;
        let desc = sub.get_value::<String, _>("DriverDesc").ok()?;
        let lower = desc.to_lowercase();
        if SKIP.iter().any(|needle| lower.contains(needle)) {
            continue;
        }
        names.push(desc);
    }

    Some(names)
}

fn pick_primary_gpu(names: &[String]) -> String {
    for name in names {
        let lower = name.to_lowercase();
        if lower.contains("nvidia")
            || lower.contains("geforce")
            || lower.contains("radeon")
            || lower.contains("amd")
        {
            return name.clone();
        }
    }
    names
        .first()
        .cloned()
        .unwrap_or_else(|| "Unknown GPU".to_string())
}

pub fn adapt_preset_for_gpu(
    files: &mut std::collections::HashMap<
        String,
        std::collections::HashMap<String, std::collections::HashMap<String, String>>,
    >,
    gpu: &GpuCapabilities,
) {
    let gus = match files.get_mut("GameUserSettings.ini") {
        Some(f) => f,
        None => return,
    };

    for (_section, keys) in gus.iter_mut() {
        if !gpu.supports_dlss {
            if keys.contains_key("DLSSMode") {
                keys.insert("DLSSMode".to_string(), "Off".to_string());
            }
            if keys.contains_key("DLSSQualityMode") {
                keys.insert("DLSSQualityMode".to_string(), "0".to_string());
            }
            keys.remove("ResolutionScaleDLSS");
            if keys
                .get("AntiAliasingType")
                .map(|s| s.contains("DLAA"))
                .unwrap_or(false)
            {
                keys.insert("AntiAliasingType".to_string(), "AAM_TSR".to_string());
            }
            if keys
                .get("UpscalingMethod")
                .map(|s| s.contains("DLSS"))
                .unwrap_or(false)
            {
                keys.insert(
                    "UpscalingMethod".to_string(),
                    if gpu.vendor == GpuVendor::Amd {
                        "U_FSR".to_string()
                    } else {
                        "U_TSR".to_string()
                    },
                );
            }
        }

        if !gpu.supports_dlss_fg {
            if keys.contains_key("UpscalingFrameGeneration") {
                keys.insert("UpscalingFrameGeneration".to_string(), "0".to_string());
            }
        }
    }

    if !gpu.supports_ray_tracing {
        if let Some(engine) = files.get_mut("Engine.ini") {
            for (_section, keys) in engine.iter_mut() {
                for key in ["r.RayTracing", "r.RayTracing.Shadows"] {
                    if keys.contains_key(key) {
                        keys.insert(key.to_string(), "0".to_string());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtx_3060_has_dlss_no_fg() {
        let gpu = GpuCapabilities::from_gpu_name("NVIDIA GeForce RTX 3060");
        assert_eq!(gpu.vendor, GpuVendor::Nvidia);
        assert!(gpu.supports_dlss);
        assert!(!gpu.supports_dlss_fg);
        assert!(gpu.supports_ray_tracing);
    }

    #[test]
    fn rtx_4090_has_fg() {
        let gpu = GpuCapabilities::from_gpu_name("NVIDIA GeForce RTX 4090");
        assert!(gpu.supports_dlss_fg);
    }

    #[test]
    fn rtx_5090_has_fg() {
        let gpu = GpuCapabilities::from_gpu_name("NVIDIA GeForce RTX 5090");
        assert!(gpu.supports_dlss);
        assert!(gpu.supports_dlss_fg);
    }

    #[test]
    fn rtx_2060_has_dlss() {
        let gpu = GpuCapabilities::from_gpu_name("NVIDIA GeForce RTX 2060");
        assert!(gpu.supports_dlss);
        assert!(!gpu.supports_dlss_fg);
    }

    #[test]
    fn gtx_1080_no_dlss() {
        let gpu = GpuCapabilities::from_gpu_name("NVIDIA GeForce GTX 1080");
        assert!(!gpu.supports_dlss);
        assert!(!gpu.supports_dlss_fg);
    }

    #[test]
    fn amd_no_nvidia_features() {
        let gpu = GpuCapabilities::from_gpu_name("AMD Radeon RX 7800 XT");
        assert_eq!(gpu.vendor, GpuVendor::Amd);
        assert!(!gpu.supports_dlss);
        assert!(!gpu.supports_dlss_fg);
    }

    #[test]
    fn adapt_preset_swaps_dlss_for_amd() {
        let mut files = std::collections::HashMap::from([(
            "GameUserSettings.ini".to_string(),
            std::collections::HashMap::from([(
                "[/Script/subnautica2.s2gameusersettings]".to_string(),
                std::collections::HashMap::from([
                    ("DLSSMode".to_string(), "Quality".to_string()),
                    ("AntiAliasingType".to_string(), "AAM_DLAA".to_string()),
                    ("UpscalingMethod".to_string(), "U_DLSS".to_string()),
                    ("UpscalingFrameGeneration".to_string(), "1".to_string()),
                ]),
            )]),
        )]);
        let gpu = GpuCapabilities::from_gpu_name("AMD Radeon RX 7800 XT");
        adapt_preset_for_gpu(&mut files, &gpu);
        let keys = &files["GameUserSettings.ini"]["[/Script/subnautica2.s2gameusersettings]"];
        assert_eq!(keys["DLSSMode"], "Off");
        assert_eq!(keys["AntiAliasingType"], "AAM_TSR");
        assert_eq!(keys["UpscalingMethod"], "U_FSR");
        assert_eq!(keys["UpscalingFrameGeneration"], "0");
    }
}
