use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static GPU_CACHE: OnceLock<GpuCapabilities> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct GpuCapabilities {
    pub name: String,
    pub vendor: GpuVendor,
    /// DLSS / DLAA — GeForce RTX 20 series and newer (Tensor Cores).
    pub supports_dlss: bool,
    /// DLSS Frame Generation — RTX 40 series and newer.
    pub supports_dlss_fg: bool,
    /// Hardware ray tracing in UE — GeForce RTX 20+.
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

/// RTX 2060 → 20, RTX 4090 → 40, RTX 5090 → 50. GTX and older Quadro without RTX → None.
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
        // Adapters are numbered 0000, 0001, … but there may be gaps and entries without DriverDesc
        // (filter drivers, software devices). Previously `.ok()?` here caused an early return None
        // on the first missing adapter, discarding an already-found GPU → detect_gpu always
        // returned "Unknown GPU" and GPU adaptation did not work.
        let Ok(sub) = class_key.open_subkey(format!("{i:04}")) else {
            continue;
        };
        let Ok(desc) = sub.get_value::<String, _>("DriverDesc") else {
            continue;
        };
        let lower = desc.to_lowercase();
        if SKIP.iter().any(|needle| lower.contains(needle)) {
            continue;
        }
        names.push(desc);
    }

    Some(names)
}

/// Pick the discrete gaming GPU, NOT the first match. On laptops and AMD/Intel APUs the
/// integrated GPU (`AMD Radeon(TM) Graphics`, `Intel UHD`) often appears before the
/// discrete NVIDIA/Radeon in the registry. The old version took the first entry with
/// `amd`/`radeon`/`nvidia` → picked integrated → vendor=Amd, DLSS/NGX disabled,
/// while the player actually games on discrete NVIDIA.
fn pick_primary_gpu(names: &[String]) -> String {
    names
        .iter()
        .max_by_key(|name| gpu_priority(name))
        .cloned()
        .unwrap_or_else(|| "Unknown GPU".to_string())
}

/// Higher score = higher priority as the "primary gaming" GPU.
/// Discrete GPUs of any vendor always beat integrated ones.
fn gpu_priority(name: &str) -> i32 {
    let lower = name.to_lowercase();
    let is_nvidia =
        lower.contains("nvidia") || lower.contains("geforce") || lower.contains("quadro");
    let is_amd = lower.contains("amd") || lower.contains("radeon");
    let is_intel = lower.contains("intel");

    if is_nvidia {
        // NVIDIA on desktops/laptops is always discrete. RTX ranks higher (DLSS/RT).
        return if nvidia_rtx_series(&lower).is_some() {
            100
        } else {
            95
        };
    }
    if is_amd {
        // AMD APU integrated: "Radeon(TM) Graphics", "Vega ... Graphics", "780M Graphics" —
        // without "RX"/"Pro". Discrete RX/Pro/Frontier outrank integrated.
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
        // Intel Arc is discrete; UHD/Iris/HD Graphics is integrated.
        return if lower.contains("arc") { 85 } else { 30 };
    }
    0
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
    fn prefers_discrete_nvidia_over_amd_igpu() {
        // AMD APU integrated appears first in the registry, discrete NVIDIA second.
        let names = vec![
            "AMD Radeon(TM) Graphics".to_string(),
            "NVIDIA GeForce RTX 4070".to_string(),
        ];
        assert_eq!(pick_primary_gpu(&names), "NVIDIA GeForce RTX 4070");
        // Reverse order — same result.
        let names_rev = vec![
            "NVIDIA GeForce RTX 4070".to_string(),
            "AMD Radeon(TM) Graphics".to_string(),
        ];
        assert_eq!(pick_primary_gpu(&names_rev), "NVIDIA GeForce RTX 4070");
    }

    #[test]
    fn picked_nvidia_enables_dlss_with_amd_igpu_present() {
        let names = vec![
            "AMD Radeon(TM) Graphics".to_string(),
            "NVIDIA GeForce RTX 3060".to_string(),
        ];
        let gpu = GpuCapabilities::from_gpu_name(&pick_primary_gpu(&names));
        assert_eq!(gpu.vendor, GpuVendor::Nvidia);
        assert!(gpu.supports_dlss, "RTX должна включать DLSS, а не встройка AMD");
    }

    #[test]
    fn prefers_discrete_amd_over_amd_igpu() {
        let names = vec![
            "AMD Radeon(TM) Graphics".to_string(),
            "AMD Radeon RX 7800 XT".to_string(),
        ];
        assert_eq!(pick_primary_gpu(&names), "AMD Radeon RX 7800 XT");
    }

    #[test]
    fn prefers_discrete_over_intel_igpu() {
        let names = vec![
            "Intel(R) UHD Graphics 770".to_string(),
            "NVIDIA GeForce RTX 4090".to_string(),
        ];
        assert_eq!(pick_primary_gpu(&names), "NVIDIA GeForce RTX 4090");
    }

    #[test]
    fn single_igpu_is_still_picked() {
        let names = vec!["AMD Radeon(TM) Graphics".to_string()];
        assert_eq!(pick_primary_gpu(&names), "AMD Radeon(TM) Graphics");
    }
}
