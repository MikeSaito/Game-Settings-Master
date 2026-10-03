use super::GpuCapabilities;

#[cfg(windows)]
pub fn enumerate() -> Vec<GpuCapabilities> {
    use windows::Win32::Graphics::Direct3D::D3D_FEATURE_LEVEL_11_0;
    use windows::Win32::Graphics::Direct3D12::*;
    use windows::Win32::Graphics::Dxgi::*;
    unsafe {
        let Ok(factory) = CreateDXGIFactory1::<IDXGIFactory1>() else {
            return fallback();
        };
        let preferred = windows::core::Interface::cast::<IDXGIFactory6>(&factory).ok();
        let mut adapters = Vec::new();
        for index in 0..32 {
            let adapter = preferred
                .as_ref()
                .and_then(|factory| {
                    factory
                        .EnumAdapterByGpuPreference::<IDXGIAdapter1>(
                            index,
                            DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE,
                        )
                        .ok()
                })
                .or_else(|| factory.EnumAdapters1(index).ok());
            let Some(adapter) = adapter else {
                break;
            };
            let Ok(desc) = adapter.GetDesc1() else {
                continue;
            };
            if desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
                continue;
            }
            let end = desc
                .Description
                .iter()
                .position(|ch| *ch == 0)
                .unwrap_or(desc.Description.len());
            let mut gpu =
                GpuCapabilities::from_gpu_name(&String::from_utf16_lossy(&desc.Description[..end]));
            gpu.vendor = match desc.VendorId {
                0x10de => super::types::GpuVendor::Nvidia,
                0x1002 => super::types::GpuVendor::Amd,
                0x8086 => super::types::GpuVendor::Intel,
                _ => super::types::GpuVendor::Unknown,
            };
            // PCI identity survives restarts; LUID distinguishes matching adapters in this session.
            gpu.adapter_id = Some(format!(
                "{:04x}:{:04x}:{:08x}:{:08x}:{:08x}",
                desc.VendorId,
                desc.DeviceId,
                desc.SubSysId,
                desc.AdapterLuid.HighPart as u32,
                desc.AdapterLuid.LowPart
            ));
            gpu.dedicated_memory_mb =
                Some((desc.DedicatedVideoMemory / (1024 * 1024)).min(u32::MAX as usize) as u32);
            gpu.shared_memory_mb =
                Some((desc.SharedSystemMemory / (1024 * 1024)).min(u32::MAX as usize) as u32);
            gpu.supports_ray_tracing = false;
            let mut device: Option<ID3D12Device> = None;
            if D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_11_0, &mut device).is_ok() {
                if let Some(device) = device {
                    let mut options = D3D12_FEATURE_DATA_D3D12_OPTIONS5::default();
                    if device
                        .CheckFeatureSupport(
                            D3D12_FEATURE_D3D12_OPTIONS5,
                            &mut options as *mut _ as _,
                            std::mem::size_of_val(&options) as u32,
                        )
                        .is_ok()
                    {
                        gpu.supports_ray_tracing =
                            options.RaytracingTier != D3D12_RAYTRACING_TIER_NOT_SUPPORTED;
                        gpu.ray_tracing_status = Some(
                            if gpu.supports_ray_tracing {
                                "supported"
                            } else {
                                "unsupported"
                            }
                            .into(),
                        );
                    }
                }
            }
            adapters.push(gpu);
        }
        if adapters.is_empty() {
            fallback()
        } else {
            adapters
        }
    }
}
#[cfg(not(windows))]
pub fn enumerate() -> Vec<GpuCapabilities> {
    fallback()
}

fn fallback() -> Vec<GpuCapabilities> {
    let names = super::enumerate::enumerate_gpu_names();
    let primary = super::priority::pick_primary_gpu(&names);
    let mut result: Vec<_> = names
        .iter()
        .map(|name| {
            let mut gpu = GpuCapabilities::from_gpu_name(name);
            gpu.adapter_id = Some(format!(
                "fallback:{}",
                crate::changes::digest(name.as_bytes())
            ));
            gpu.supports_ray_tracing = false;
            gpu
        })
        .collect();
    result.sort_by_key(|gpu| gpu.name != primary);
    result
}

pub fn select(adapters: &[GpuCapabilities], saved: Option<&str>) -> GpuCapabilities {
    if let Some(id) = saved {
        if let Some(gpu) = adapters
            .iter()
            .find(|gpu| gpu.adapter_id.as_deref() == Some(id))
        {
            return gpu.clone();
        }
        let identity = id.split(':').take(3).collect::<Vec<_>>().join(":");
        let matches: Vec<_> = adapters
            .iter()
            .filter(|gpu| {
                gpu.adapter_id.as_ref().is_some_and(|candidate| {
                    candidate.split(':').take(3).collect::<Vec<_>>().join(":") == identity
                })
            })
            .collect();
        if matches.len() == 1 {
            return matches[0].clone();
        }
    }
    let mut gpu = adapters
        .first()
        .cloned()
        .unwrap_or_else(|| GpuCapabilities::from_gpu_name("Unknown GPU"));
    if saved.is_some() {
        gpu.selection_warning = Some(crate::i18n::t(
            "Сохранённая GPU недоступна; используется автоматический выбор",
            "Saved GPU is unavailable; using automatic selection",
        ));
    }
    gpu
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selection_rematches_pci_identity_and_reports_missing_gpu() {
        let mut amd = GpuCapabilities::from_gpu_name("AMD Radeon RX 7800 XT");
        amd.adapter_id = Some("1002:747e:00000001:1:2".into());
        amd.supports_ray_tracing = true;
        assert_eq!(
            select(&[amd.clone()], Some("1002:747e:00000001:3:4")).name,
            amd.name
        );
        assert!(select(&[amd], Some("missing")).selection_warning.is_some());
    }
    #[test]
    fn gpu_selection_keeps_memory_and_ray_tracing_separate_from_dlss() {
        let mut intel = GpuCapabilities::from_gpu_name("Intel Arc A770");
        intel.adapter_id = Some("intel".into());
        intel.dedicated_memory_mb = Some(16384);
        intel.shared_memory_mb = Some(8192);
        intel.supports_ray_tracing = true;
        intel.ray_tracing_status = Some("supported".into());
        let mut amd = GpuCapabilities::from_gpu_name("AMD Radeon RX 7800 XT");
        amd.adapter_id = Some("amd".into());
        amd.supports_ray_tracing = true;
        amd.ray_tracing_status = Some("supported".into());
        let selected = select(&[amd, intel], Some("intel"));
        assert!(!selected.supports_dlss);
        assert!(selected.supports_ray_tracing);
        assert_eq!(selected.dedicated_memory_mb, Some(16384));
        assert_eq!(selected.shared_memory_mb, Some(8192));
        assert_eq!(
            select(&[], None).ray_tracing_status.as_deref(),
            Some("unknown")
        );
    }
    #[test]
    #[ignore = "requires a Windows hardware session"]
    fn probe_local_hardware_adapters() {
        let adapters = enumerate();
        assert!(!adapters.is_empty(), "No hardware adapters available");
        for gpu in &adapters {
            println!("{}", serde_json::to_string(gpu).unwrap());
            assert!(matches!(
                gpu.ray_tracing_status.as_deref(),
                Some("supported" | "unsupported" | "unknown")
            ));
            assert!(gpu.adapter_id.is_some());
        }
    }
}
