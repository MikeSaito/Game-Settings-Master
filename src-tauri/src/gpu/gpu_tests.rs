use super::priority::pick_primary_gpu;
use super::types::{GpuCapabilities, GpuVendor};

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
    let names = vec![
        "AMD Radeon(TM) Graphics".to_string(),
        "NVIDIA GeForce RTX 4070".to_string(),
    ];
    assert_eq!(pick_primary_gpu(&names), "NVIDIA GeForce RTX 4070");
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
    assert!(
        gpu.supports_dlss,
        "RTX должна включать DLSS, а не встройка AMD"
    );
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
