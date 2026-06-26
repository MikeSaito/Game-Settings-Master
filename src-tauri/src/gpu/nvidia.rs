use super::types::GpuVendor;

pub(crate) fn detect_vendor(lower: &str) -> GpuVendor {
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
pub(crate) fn nvidia_rtx_series(lower: &str) -> Option<u8> {
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
