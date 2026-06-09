use crate::discovery::UeEngineFamily;
use crate::gpu::{detect_gpu, GpuCapabilities, GpuVendor};
use std::collections::HashMap;

type IniFiles = HashMap<String, HashMap<String, HashMap<String, String>>>;

const DLSS_SCALE: &[(&str, &str, &str)] = &[
    ("Off", "0", "1.0"),
    ("Performance", "1", "0.5"),
    ("Balanced", "2", "0.58"),
    ("Quality", "3", "0.66"),
    ("UltraQuality", "4", "0.77"),
    ("DLAA", "5", "1.0"),
];

pub fn tune_combined_preset(preset_id: &str, files: &mut IniFiles, engine_family: UeEngineFamily) {
    let is_ue4 = engine_family == UeEngineFamily::Ue4;
    tune_engine_for_tier(preset_id, files, is_ue4);
    if !is_ue4 {
        tune_subnautica2_for_tier(preset_id, files);
    }
    let gpu = detect_gpu();
    crate::gpu::adapt_preset_for_gpu(files, &gpu);
    reconcile_upscaling_chain(files, &gpu);
}

/// Индекс sg.*Quality (0..menu_max, обычно 4=Epic). Не путать с r.* и процентами.
fn menu_tier_sg_level(
    preset_id: &str,
    menu_max: u32,
    _is_ue4: bool,
    _game_id: Option<&str>,
) -> u32 {
    match preset_id {
        "ultra-low" => 0,
        "low" => 1.min(menu_max),
        "medium" => 2.min(menu_max),
        "high" => 3.min(menu_max),
        "epic" | "ultra-high" => menu_max,
        _ => menu_max,
    }
}

/// Ограничить sg.* по уровню пресета (не затирать low в нули).
pub fn apply_tier_to_scalability(
    sections: &mut HashMap<String, HashMap<String, String>>,
    limits: &crate::scalability::ScalabilityLimits,
    preset_id: &str,
    engine_family: UeEngineFamily,
    game_id: Option<&str>,
) {
    use crate::scalability::is_scalability_quality_index;

    let is_ue4 = engine_family == UeEngineFamily::Ue4;
    let scalability = sections
        .entry("[ScalabilityGroups]".to_string())
        .or_default();

    for (sg_key, max_level) in &limits.groups {
        if !is_scalability_quality_index(sg_key) {
            continue;
        }
        if is_ue4 && is_ue5_only_sg(sg_key) {
            continue;
        }

        let desired = scalability.get(sg_key).and_then(|s| s.parse::<u32>().ok());

        let final_val = menu_tier_sg_level(preset_id, *max_level, is_ue4, game_id);

        if desired.is_some()
            || matches!(
                preset_id,
                "ultra-low" | "low" | "medium" | "high" | "epic" | "ultra-high"
            )
        {
            scalability.insert(sg_key.clone(), final_val.to_string());
        }
    }

    // Масштаб разрешения — фиксированный процент по пресету.
    if let Some(scale) = resolution_quality_for_tier(preset_id, is_ue4) {
        scalability.insert("sg.ResolutionQuality".to_string(), scale.to_string());
    }

    if !is_ue4 {
        for extra in ["sg.LandscapeQuality", "sg.CloudsQuality"] {
            if !scalability.contains_key(extra) && limits.groups.contains_key(extra) {
                let level = menu_tier_sg_level(preset_id, limits.max_for(extra), false, game_id);
                scalability.insert(extra.to_string(), level.to_string());
            }
        }
    }

    // Убрать UE5-группы из шаблона, если они попали из общего JSON.
    if is_ue4 {
        scalability.retain(|key, _| !is_ue5_only_sg(key));
    }
}

fn is_ue5_only_sg(sg_key: &str) -> bool {
    matches!(
        sg_key,
        "sg.GlobalIlluminationQuality"
            | "sg.ShadingQuality"
            | "sg.LandscapeQuality"
            | "sg.CloudsQuality"
    )
}

fn resolution_quality_for_tier(preset_id: &str, _is_ue4: bool) -> Option<&'static str> {
    match preset_id {
        "ultra-low" => Some("45"),
        "low" => Some("70"),
        "medium" => Some("85"),
        "high" | "epic" | "ultra-high" => Some("100"),
        _ => None,
    }
}

fn tune_engine_for_tier(preset_id: &str, files: &mut IniFiles, is_ue4: bool) {
    if !matches!(preset_id, "high" | "ultra-high") {
        return;
    }
    let Some(engine) = files.get_mut("Engine.ini") else {
        return;
    };
    let sys = engine.entry("[SystemSettings]".to_string()).or_default();
    if is_ue4 {
        sys.insert(
            "r.DefaultFeature.MotionBlur".to_string(),
            "False".to_string(),
        );
        sys.insert("r.MotionBlurQuality".to_string(), "0".to_string());
        return;
    }
    let gpu = detect_gpu();
    super::engine_boost::tune_engine_system_settings(sys, preset_id, &gpu);
}

fn tune_subnautica2_for_tier(preset_id: &str, files: &mut IniFiles) {
    let Some(gus) = files.get_mut("GameUserSettings.ini") else {
        return;
    };

    let s2 = "/Script/subnautica2.s2gameusersettings";
    let local = "/Script/subnautica2.sn2settingslocal";

    if !gus.contains_key(s2) && !gus.keys().any(|k| k.to_lowercase().contains("subnautica2")) {
        return;
    }

    let s2_key = gus
        .keys()
        .find(|k| k.to_lowercase().contains("s2gameusersettings"))
        .cloned()
        .unwrap_or_else(|| s2.to_string());
    let local_key = gus
        .keys()
        .find(|k| k.to_lowercase().contains("sn2settingslocal"))
        .cloned()
        .unwrap_or_else(|| local.to_string());

    let (graphics, dlss_mode, dlss_num, scale, tsr, aa, upscaling, fg) =
        subnautica2_tier(preset_id);

    let mut local_extras: HashMap<String, String> = HashMap::from([
        ("DLSSQualityMode".to_string(), dlss_num.to_string()),
        ("TSRQualityMode".to_string(), tsr.to_string()),
        ("UpscalingMethod".to_string(), upscaling.to_string()),
    ]);

    if dlss_mode == "Off" {
        local_extras.remove("ResolutionScaleDLSS");
    } else {
        local_extras.insert("ResolutionScaleDLSS".to_string(), scale.to_string());
    }

    match preset_id {
        "ultra-low" => {
            local_extras.extend([
                ("ResolutionScaleMin".to_string(), "0.25".to_string()),
                ("ResolutionScaleMax".to_string(), "0.5".to_string()),
                ("EnableMotionBlur".to_string(), "Off".to_string()),
                ("EnableLensFlare".to_string(), "Off".to_string()),
                ("EnableChromaticAberration".to_string(), "Off".to_string()),
                ("EnableUnderwaterBlur".to_string(), "Off".to_string()),
            ]);
        }
        "low" => {
            local_extras.extend([
                ("ResolutionScaleMin".to_string(), "0.5".to_string()),
                ("ResolutionScaleMax".to_string(), "1.0".to_string()),
                ("EnableMotionBlur".to_string(), "Off".to_string()),
                ("EnableLensFlare".to_string(), "Off".to_string()),
                ("EnableChromaticAberration".to_string(), "Off".to_string()),
            ]);
        }
        "medium" => {
            local_extras.extend([
                ("ResolutionScaleMin".to_string(), "0.58".to_string()),
                ("ResolutionScaleMax".to_string(), "1.0".to_string()),
                ("EnableMotionBlur".to_string(), "Off".to_string()),
            ]);
        }
        "high" | "epic" => {
            local_extras.extend([
                ("ResolutionScaleMin".to_string(), "0.66".to_string()),
                ("ResolutionScaleMax".to_string(), "1.0".to_string()),
                ("EnableMotionBlur".to_string(), "Off".to_string()),
            ]);
        }
        "ultra-high" => {
            local_extras.extend([
                ("ResolutionScaleMin".to_string(), "1.0".to_string()),
                ("ResolutionScaleMax".to_string(), "1.0".to_string()),
                ("EnableMotionBlur".to_string(), "Off".to_string()),
                ("EnableLensFlare".to_string(), "Off".to_string()),
                ("EnableChromaticAberration".to_string(), "Off".to_string()),
            ]);
        }
        _ => {}
    }

    gus.entry(s2_key).or_default().extend([
        ("GraphicsLevel".to_string(), graphics.to_string()),
        ("DefaultGraphicsLevel".to_string(), graphics.to_string()),
        ("bHasAppliedUserSetting".to_string(), "True".to_string()),
        ("DLSSMode".to_string(), dlss_mode.to_string()),
        ("AntiAliasingType".to_string(), aa.to_string()),
        ("UpscalingFrameGeneration".to_string(), fg.to_string()),
    ]);
    gus.entry(local_key).or_default().extend(local_extras);
}

/// (GraphicsLevel, DLSSMode, DLSSQualityMode, scale, TSRQuality, AA, UpscalingMethod, FG)
fn subnautica2_tier(
    preset_id: &str,
) -> (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
) {
    match preset_id {
        "ultra-low" => (
            "Low",
            "Performance",
            "1",
            "0.5",
            "0",
            "AAM_FXAA",
            "U_None",
            "0",
        ),
        "low" => ("Low", "Off", "0", "1.0", "1", "AAM_TSR", "U_TSR", "0"),
        "medium" => (
            "Medium", "Balanced", "2", "0.58", "2", "AAM_TSR", "U_TSR", "0",
        ),
        "high" => (
            "High", "Quality", "3", "0.66", "0", "AAM_TSR", "U_DLSS", "0",
        ),
        "epic" => (
            "Epic", "Quality", "3", "0.66", "0", "AAM_TSR", "U_DLSS", "0",
        ),
        "ultra-high" => (
            "Cinematic",
            "Quality",
            "3",
            "0.66",
            "0",
            "AAM_TSR",
            "U_DLSS",
            "1",
        ),
        _ => (
            "High", "Balanced", "2", "0.58", "2", "AAM_TSR", "U_TSR", "0",
        ),
    }
}

fn reconcile_upscaling_chain(files: &mut IniFiles, gpu: &GpuCapabilities) {
    let Some(gus) = files.get_mut("GameUserSettings.ini") else {
        return;
    };

    let section_keys: Vec<String> = gus.keys().cloned().collect();
    let s2_key = section_keys
        .iter()
        .find(|k| {
            gus.get(*k)
                .map(|s| s.contains_key("DLSSMode"))
                .unwrap_or(false)
        })
        .cloned();
    let local_key = section_keys
        .iter()
        .find(|k| {
            gus.get(*k)
                .map(|s| {
                    s.contains_key("UpscalingMethod")
                        || s.contains_key("DLSSQualityMode")
                        || s.contains_key("TSRQualityMode")
                })
                .unwrap_or(false)
        })
        .cloned();

    let Some(s2_key) = s2_key else {
        return;
    };

    let dlss_mode = gus
        .get(&s2_key)
        .and_then(|s| s.get("DLSSMode"))
        .map(|s| s.as_str())
        .unwrap_or("Off");
    let dlss_off = dlss_mode.eq_ignore_ascii_case("off");

    if !gpu.supports_dlss {
        if let Some(s2) = gus.get_mut(&s2_key) {
            s2.insert("DLSSMode".to_string(), "Off".to_string());
            s2.insert("AntiAliasingType".to_string(), "AAM_TSR".to_string());
            if !gpu.supports_dlss_fg {
                s2.insert("UpscalingFrameGeneration".to_string(), "0".to_string());
            }
        }
        if let Some(local_key) = local_key {
            if let Some(local) = gus.get_mut(&local_key) {
                local.insert("DLSSQualityMode".to_string(), "0".to_string());
                local.remove("ResolutionScaleDLSS");
                let supersampling = local
                    .get("ResolutionScaleMin")
                    .and_then(|s| s.parse::<f32>().ok())
                    .is_some_and(|v| v >= 1.0);
                if supersampling {
                    local.insert("UpscalingMethod".to_string(), "U_None".to_string());
                } else {
                    let use_fsr = gpu.vendor == GpuVendor::Amd;
                    local.insert(
                        "UpscalingMethod".to_string(),
                        if use_fsr { "U_FSR" } else { "U_TSR" }.to_string(),
                    );
                    if local
                        .get("TSRQualityMode")
                        .map(|s| s == "0")
                        .unwrap_or(true)
                    {
                        local.insert("TSRQualityMode".to_string(), "2".to_string());
                    }
                }
            }
        }
        return;
    }

    if !dlss_off {
        if let Some((_, num, scale)) = DLSS_SCALE
            .iter()
            .find(|(m, _, _)| m.eq_ignore_ascii_case(dlss_mode))
        {
            if let Some(local_key) = local_key.clone() {
                if let Some(local) = gus.get_mut(&local_key) {
                    local.insert("DLSSQualityMode".to_string(), (*num).to_string());
                    local.insert("ResolutionScaleDLSS".to_string(), (*scale).to_string());
                    local.insert("UpscalingMethod".to_string(), "U_DLSS".to_string());
                    local.insert("TSRQualityMode".to_string(), "0".to_string());
                }
            }
        }
    } else if let Some(local_key) = local_key.clone() {
        if let Some(local) = gus.get_mut(&local_key) {
            local.insert("DLSSQualityMode".to_string(), "0".to_string());
            if local
                .get("UpscalingMethod")
                .map(|s| s.contains("TSR"))
                .unwrap_or(false)
                && local
                    .get("TSRQualityMode")
                    .map(|s| s == "0")
                    .unwrap_or(true)
            {
                local.insert("TSRQualityMode".to_string(), "2".to_string());
            }
        }
    }

    if !gpu.supports_dlss_fg {
        if let Some(s2) = gus.get_mut(&s2_key) {
            s2.insert("UpscalingFrameGeneration".to_string(), "0".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::GpuCapabilities;
    use crate::scalability::detect_scalability_limits;

    #[test]
    fn ultra_high_uses_epic_scalability_index() {
        let mut sections = HashMap::new();
        sections.insert("[ScalabilityGroups]".to_string(), HashMap::new());
        let limits = detect_scalability_limits(None, None);
        apply_tier_to_scalability(
            &mut sections,
            &limits,
            "ultra-high",
            UeEngineFamily::Ue5,
            None,
        );
        let sg = sections.get("[ScalabilityGroups]").unwrap();
        assert_eq!(
            sg.get("sg.ShadowQuality").map(String::as_str),
            Some(limits.global_max.to_string().as_str())
        );
        assert_eq!(
            sg.get("sg.ResolutionQuality").map(String::as_str),
            Some("100")
        );
    }

    #[test]
    fn low_uses_sg_one_not_zero() {
        let mut sections = HashMap::new();
        sections.insert("[ScalabilityGroups]".to_string(), HashMap::new());
        let limits = detect_scalability_limits(None, None);
        apply_tier_to_scalability(&mut sections, &limits, "low", UeEngineFamily::Ue5, None);
        let sg = sections.get("[ScalabilityGroups]").unwrap();
        assert_eq!(sg.get("sg.ShadowQuality").map(String::as_str), Some("1"));
        assert_eq!(
            sg.get("sg.ResolutionQuality").map(String::as_str),
            Some("70")
        );
    }

    #[test]
    fn sn2_ultra_high_uses_dlss_not_supersampling() {
        let mut files = IniFiles::new();
        files.insert(
            "GameUserSettings.ini".to_string(),
            HashMap::from([
                (
                    "/Script/subnautica2.s2gameusersettings".to_string(),
                    HashMap::new(),
                ),
                (
                    "/Script/subnautica2.sn2settingslocal".to_string(),
                    HashMap::new(),
                ),
            ]),
        );
        tune_subnautica2_for_tier("ultra-high", &mut files);
        let gus = &files["GameUserSettings.ini"];
        let s2 = &gus["/Script/subnautica2.s2gameusersettings"];
        let local = &gus["/Script/subnautica2.sn2settingslocal"];
        assert_eq!(
            s2.get("GraphicsLevel").map(String::as_str),
            Some("Cinematic")
        );
        assert_eq!(s2.get("DLSSMode").map(String::as_str), Some("Quality"));
        assert_eq!(
            local.get("UpscalingMethod").map(String::as_str),
            Some("U_DLSS")
        );
        assert_eq!(
            local.get("ResolutionScaleMin").map(String::as_str),
            Some("1.0")
        );
        assert_eq!(
            local.get("ResolutionScaleMax").map(String::as_str),
            Some("1.0")
        );
        assert_eq!(
            s2.get("UpscalingFrameGeneration").map(String::as_str),
            Some("1")
        );
    }

    #[test]
    fn tier_ladder_anchors_on_epic_max() {
        let limits = detect_scalability_limits(None, None);
        let max = limits.global_max;

        let mut epic_sg = HashMap::new();
        epic_sg.insert("[ScalabilityGroups]".to_string(), HashMap::new());
        apply_tier_to_scalability(&mut epic_sg, &limits, "epic", UeEngineFamily::Unknown, None);
        assert_eq!(
            epic_sg["[ScalabilityGroups]"]["sg.ShadowQuality"],
            max.to_string()
        );

        let mut high_sg = HashMap::new();
        high_sg.insert("[ScalabilityGroups]".to_string(), HashMap::new());
        apply_tier_to_scalability(&mut high_sg, &limits, "high", UeEngineFamily::Unknown, None);
        let high_val: u32 = high_sg["[ScalabilityGroups]"]["sg.ShadowQuality"]
            .parse()
            .unwrap();
        assert_eq!(high_val, 3.min(max));

        let mut medium_sg = HashMap::new();
        medium_sg.insert("[ScalabilityGroups]".to_string(), HashMap::new());
        apply_tier_to_scalability(
            &mut medium_sg,
            &limits,
            "medium",
            UeEngineFamily::Unknown,
            None,
        );
        let medium_val: u32 = medium_sg["[ScalabilityGroups]"]["sg.ShadowQuality"]
            .parse()
            .unwrap();
        assert_eq!(medium_val, 2.min(max));
    }

    #[test]
    fn ultra_low_is_potato_below_menu() {
        let mut sections = HashMap::new();
        sections.insert("[ScalabilityGroups]".to_string(), HashMap::new());
        let limits = detect_scalability_limits(None, None);
        apply_tier_to_scalability(
            &mut sections,
            &limits,
            "ultra-low",
            UeEngineFamily::Unknown,
            None,
        );
        let sg = sections.get("[ScalabilityGroups]").unwrap();
        assert_eq!(sg.get("sg.ShadowQuality").map(String::as_str), Some("0"));
        assert_eq!(
            sg.get("sg.ResolutionQuality").map(String::as_str),
            Some("45")
        );
    }

    #[test]
    fn sn2_ultra_low_uses_tsr_not_dlss() {
        let mut files = IniFiles::new();
        files.insert(
            "GameUserSettings.ini".to_string(),
            HashMap::from([
                (
                    "/Script/subnautica2.s2gameusersettings".to_string(),
                    HashMap::new(),
                ),
                (
                    "/Script/subnautica2.sn2settingslocal".to_string(),
                    HashMap::new(),
                ),
            ]),
        );
        tune_subnautica2_for_tier("ultra-low", &mut files);
        let gus = &files["GameUserSettings.ini"];
        let s2 = &gus["/Script/subnautica2.s2gameusersettings"];
        let local = &gus["/Script/subnautica2.sn2settingslocal"];
        assert_eq!(s2.get("DLSSMode").map(String::as_str), Some("Performance"));
        assert_eq!(s2.get("GraphicsLevel").map(String::as_str), Some("Low"));
        assert_eq!(
            local.get("ResolutionScaleMin").map(String::as_str),
            Some("0.25")
        );
        assert_eq!(
            local.get("ResolutionScaleMax").map(String::as_str),
            Some("0.5")
        );
    }

    #[test]
    fn amd_ultra_high_keeps_supersampling_not_fsr() {
        let mut files = IniFiles::new();
        files.insert(
            "GameUserSettings.ini".to_string(),
            HashMap::from([
                (
                    "/Script/subnautica2.s2gameusersettings".to_string(),
                    HashMap::from([("DLSSMode".to_string(), "Off".to_string())]),
                ),
                (
                    "/Script/subnautica2.sn2settingslocal".to_string(),
                    HashMap::from([
                        ("ResolutionScaleMin".to_string(), "1.5".to_string()),
                        ("ResolutionScaleMax".to_string(), "1.5".to_string()),
                        ("UpscalingMethod".to_string(), "U_None".to_string()),
                        ("TSRQualityMode".to_string(), "4".to_string()),
                    ]),
                ),
            ]),
        );
        let gpu = GpuCapabilities::from_gpu_name("AMD Radeon RX 7800 XT");
        reconcile_upscaling_chain(&mut files, &gpu);
        let local = &files["GameUserSettings.ini"]["/Script/subnautica2.sn2settingslocal"];
        assert_eq!(
            local.get("UpscalingMethod").map(String::as_str),
            Some("U_None")
        );
        assert_eq!(
            local.get("ResolutionScaleMin").map(String::as_str),
            Some("1.5")
        );
    }

    #[test]
    fn amd_adapt_swaps_dlss_to_fsr() {
        let mut files = IniFiles::new();
        let mut gus = HashMap::new();
        let mut s2 = HashMap::new();
        s2.insert("DLSSMode".to_string(), "Quality".to_string());
        gus.insert("/Script/subnautica2.s2gameusersettings".to_string(), s2);
        let mut local = HashMap::new();
        local.insert("UpscalingMethod".to_string(), "U_DLSS".to_string());
        gus.insert("/Script/subnautica2.sn2settingslocal".to_string(), local);
        files.insert("GameUserSettings.ini".to_string(), gus);

        let gpu = GpuCapabilities::from_gpu_name("AMD Radeon RX 7800 XT");
        reconcile_upscaling_chain(&mut files, &gpu);

        let local = &files["GameUserSettings.ini"]["/Script/subnautica2.sn2settingslocal"];
        assert_eq!(
            local.get("UpscalingMethod").map(String::as_str),
            Some("U_FSR")
        );
    }
}
