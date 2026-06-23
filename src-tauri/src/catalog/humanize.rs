use crate::core::models::GameParameter;
use crate::scalability::is_scalability_quality_index;

pub(crate) const UE5_ONLY_SG_KEYS: &[&str] = &[
    "sg.GlobalIlluminationQuality",
    "sg.ReflectionQuality",
    "sg.ShadingQuality",
    "sg.LandscapeQuality",
    "sg.CloudsQuality",
];

pub(crate) const UE5_ONLY_CVAR_KEYS: &[&str] = &[
    "r.Nanite",
    "r.Lumen.DiffuseIndirect.Allow",
    "r.Lumen.Reflections.Allow",
    "r.Lumen.Reflections.Quality",
    "r.Lumen.ScreenProbeGather.ScreenTraces",
    "r.VolumetricCloud",
];

pub(crate) const HIDDEN_UE_MANUAL_KEYS: &[&str] = &[
    "DLSSQualityMode",
    "ResolutionScaleDLSS",
    "BenchmarkResolutionX",
    "BenchmarkResolutionY",
    "bUseDesiredScreenHeight",
    "bUseDesiredScreenWidth",
    "DesiredScreenHeight",
    "DesiredScreenWidth",
    "InstallGUID",
    "LastCPUBenchmarkResult",
    "LastCPUBenchmarkSteps",
    "LastGPUBenchmarkMultiplier",
    "LastGPUBenchmarkResult",
    "LastGPUBenchmarkSteps",
    "LastRecommendedScreenHeight",
    "LastRecommendedScreenWidth",
    "RunNumber",
    "Version",
    "WindowPosX",
    "WindowPosY",
    "r.AsyncCompute",
    "r.D3D12.ExecuteContextInParallel",
    "r.D3D12.UseAllowTearing",
    "r.FinishCurrentFrame",
    "r.Fog.HZBAsyncCompute",
    "r.IO.UseDirectStorage",
    "r.OneFrameThreadLag",
    "r.RHICmdBypass",
    "r.RHICmdUseParallelAlgorithms",
    "r.RHICmdUseThread",
    "r.SceneDepthHZBAsyncCompute",
    "r.SkyAtmosphereAsyncCompute",
    "r.Streaming.LimitPoolSizeToVRAM",
    "r.Streaming.PoolSize",
    "r.Streaming.UseFixedPoolSize",
];
pub(crate) fn is_opaque_struct_value(value: &str) -> bool {
    let v = value.trim();
    if v.len() > 200 {
        return true;
    }
    if v.starts_with('(') {
        return true;
    }
    let lower = v.to_ascii_lowercase();
    [
        "actionkeylist=",
        "axiskeylist=",
        "sensitivemap=",
        "gamepadkeylist=",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

pub(crate) fn truncate_preview(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let truncated: String = value.chars().take(max_chars).collect();
    format!("{truncated}…")
}
pub(crate) fn is_standard_ue_cvar_key(key: &str) -> bool {
    key.starts_with("r.")
        || key.starts_with("sg.")
        || key.starts_with("fx.")
        || key.starts_with("t.")
}

fn split_identifier_part(part: &str) -> Vec<String> {
    let normalized = part.replace(['_', '-'], " ");
    if normalized.trim().is_empty() {
        return Vec::new();
    }
    normalized
        .split(|c: char| c == '.' || c.is_whitespace())
        .flat_map(split_identifier_segment)
        .collect()
}

fn split_identifier_segment(segment: &str) -> Vec<String> {
    let chars: Vec<char> = segment.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }

    let mut tokens = Vec::new();
    let mut start = 0;
    for i in 1..chars.len() {
        let prev = chars[i - 1];
        let current = chars[i];
        let next = chars.get(i + 1).copied();
        let boundary = (current.is_ascii_uppercase()
            && (prev.is_ascii_lowercase() || prev.is_ascii_digit()))
            || (current.is_ascii_uppercase()
                && prev.is_ascii_uppercase()
                && next.is_some_and(|c| c.is_ascii_lowercase()))
            || (current.is_ascii_digit() && !prev.is_ascii_digit())
            || (!current.is_ascii_digit() && prev.is_ascii_digit());

        if boundary {
            tokens.push(chars[start..i].iter().collect());
            start = i;
        }
    }
    tokens.push(chars[start..].iter().collect());
    tokens
}

fn humanize_token(token: &str) -> String {
    let lower = token.to_lowercase();
    match lower.as_str() {
        "max" => crate::i18n::t("макс.", "max"),
        "min" => crate::i18n::t("мин.", "min"),
        "quality" => crate::i18n::t("качество", "quality"),
        "scale" => crate::i18n::t("масштаб", "scale"),
        "distance" => crate::i18n::t("дальность", "distance"),
        "shadow" | "shadows" => crate::i18n::t("тени", "shadows"),
        "streaming" => crate::i18n::t("потоковая загрузка", "streaming"),
        "resolution" => crate::i18n::t("разрешение", "resolution"),
        "texture" | "textures" => crate::i18n::t("текстуры", "textures"),
        "detail" => crate::i18n::t("детализация", "detail"),
        "lod" => crate::i18n::t("LOD", "LOD"),
        "fog" => crate::i18n::t("туман", "fog"),
        "motion" => crate::i18n::t("движение", "motion"),
        "blur" => crate::i18n::t("размытие", "blur"),
        "reflection" | "reflections" => crate::i18n::t("отражения", "reflections"),
        "ambient" => crate::i18n::t("окружение", "ambient"),
        "occlusion" => crate::i18n::t("затенение", "occlusion"),
        "postprocess" | "post" => crate::i18n::t("постобработка", "post"),
        "process" => crate::i18n::t("обработка", "process"),
        "light" | "lighting" | "lights" => crate::i18n::t("освещение", "lighting"),
        "view" => crate::i18n::t("обзор", "view"),
        "field" => crate::i18n::t("поле", "field"),
        "depth" => crate::i18n::t("глубина", "depth"),
        "anisotropy" => crate::i18n::t("анизотропия", "anisotropy"),
        "filter" | "filtering" => crate::i18n::t("фильтрация", "filtering"),
        "anti" => crate::i18n::t("сглаживание", "anti"),
        "aa" => crate::i18n::t("AA", "AA"),
        "aliasing" => crate::i18n::t("сглаживание", "aliasing"),
        "taa" => crate::i18n::t("TAA", "TAA"),
        "temporal" => crate::i18n::t("временное", "temporal"),
        "upscale" | "upscaling" | "upsampling" => crate::i18n::t("апскейлинг", "upscaling"),
        "generation" => crate::i18n::t("генерация", "generation"),
        "frame" => crate::i18n::t("кадр", "frame"),
        "fps" => crate::i18n::t("FPS", "FPS"),
        "rate" => crate::i18n::t("частота", "rate"),
        "limit" => crate::i18n::t("лимит", "limit"),
        "fullscreen" => crate::i18n::t("полный экран", "fullscreen"),
        "window" => crate::i18n::t("окно", "window"),
        "mode" => crate::i18n::t("режим", "mode"),
        "audio" => crate::i18n::t("аудио", "audio"),
        "volume" => crate::i18n::t("громкость", "volume"),
        "render" | "rendering" => crate::i18n::t("рендеринг", "rendering"),
        "world" => crate::i18n::t("мир", "world"),
        "foliage" => crate::i18n::t("растительность", "foliage"),
        "grass" => crate::i18n::t("трава", "grass"),
        "hair" => crate::i18n::t("волосы", "hair"),
        "skin" => crate::i18n::t("кожа", "skin"),
        "water" => crate::i18n::t("вода", "water"),
        "sky" => crate::i18n::t("небо", "sky"),
        "cloud" | "clouds" => crate::i18n::t("облака", "clouds"),
        "volumetric" => crate::i18n::t("объёмный", "volumetric"),
        "global" => crate::i18n::t("глобальный", "global"),
        "illumination" => crate::i18n::t("освещение", "illumination"),
        "exposure" => crate::i18n::t("экспозиция", "exposure"),
        "brightness" => crate::i18n::t("яркость", "brightness"),
        "contrast" => crate::i18n::t("контраст", "contrast"),
        "gamma" => crate::i18n::t("гамма", "gamma"),
        "sharpen" | "sharpness" => crate::i18n::t("резкость", "sharpness"),
        "distancefield" => crate::i18n::t("поле дистанции", "distance field"),
        "distancefields" => crate::i18n::t("поля дистанции", "distance fields"),
        "nanite" => crate::i18n::t("Nanite", "Nanite"),
        "lumen" => crate::i18n::t("Lumen", "Lumen"),
        "raytracing" => crate::i18n::t("трассировка лучей", "ray tracing"),
        "raytraced" => crate::i18n::t("с трассировкой лучей", "ray traced"),
        "pathtracing" => crate::i18n::t("path tracing", "path tracing"),
        "virtual" => crate::i18n::t("виртуальный", "virtual"),
        "vsm" => crate::i18n::t("VSM", "VSM"),
        "gi" => crate::i18n::t("GI", "GI"),
        "dlss" => crate::i18n::t("DLSS", "DLSS"),
        "fsr" => crate::i18n::t("FSR", "FSR"),
        "tsr" => crate::i18n::t("TSR", "TSR"),
        "ssr" => crate::i18n::t("SSR", "SSR"),
        "ssao" => crate::i18n::t("SSAO", "SSAO"),
        "rhi" => crate::i18n::t("RHI", "RHI"),
        "hdr" => crate::i18n::t("HDR", "HDR"),
        "hmd" => crate::i18n::t("HMD", "HMD"),
        "cache" => crate::i18n::t("кэш", "cache"),
        "pool" => crate::i18n::t("пул", "pool"),
        "size" => crate::i18n::t("размер", "size"),
        "count" => crate::i18n::t("количество", "count"),
        "enable" | "enabled" => crate::i18n::t("включено", "enabled"),
        "disable" | "disabled" => crate::i18n::t("выключено", "disabled"),
        "use" => crate::i18n::t("использовать", "use"),
        "allow" => crate::i18n::t("разрешить", "allow"),
        other => {
            if other.len() <= 4 && other.chars().all(|c| !c.is_lowercase()) {
                return other.to_string();
            }
            let mut chars = other.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }
    }
}

pub(crate) fn humanize_cvar_key(key: &str) -> String {
    let stripped = key
        .strip_prefix("r.")
        .or_else(|| key.strip_prefix("sg."))
        .or_else(|| key.strip_prefix("fx."))
        .or_else(|| key.strip_prefix("t."))
        .or_else(|| key.strip_prefix("p."))
        .unwrap_or(key);
    stripped
        .split('.')
        .flat_map(split_identifier_part)
        .map(|token| humanize_token(&token))
        .collect::<Vec<_>>()
        .join(" · ")
}

pub(crate) fn apply_known_range_patterns(param: &mut GameParameter) {
    if param.min.is_some() && param.max.is_some() {
        return;
    }
    if param.value_type == "bool" || param.value_type == "enum" || param.value_type == "string" {
        return;
    }

    let key = param.key.to_lowercase();

    if key == "sg.resolutionquality" {
        param.min = Some("25".to_string());
        param.max = Some("200".to_string());
        return;
    }

    if key.starts_with("sg.") && key.ends_with("quality") {
        param.min = Some("0".to_string());
        param.max = Some("4".to_string());
        return;
    }

    if key.contains("poolsize") {
        param.min = Some("128".to_string());
        param.max = Some("32768".to_string());
        return;
    }

    if key.contains("anisotropy") {
        param.min = Some("0".to_string());
        param.max = Some("16".to_string());
        return;
    }

    if key.contains("mipbias") {
        param.min = Some("-3".to_string());
        param.max = Some("15".to_string());
        return;
    }

    if key.contains("framerate") || key.contains("framelimit") || key.ends_with("fps") {
        param.min = Some("0".to_string());
        param.max = Some("360".to_string());
        return;
    }

    if key.contains("fov") || key.contains("fieldofview") {
        param.min = Some("70".to_string());
        param.max = Some("120".to_string());
        return;
    }

    if key.contains("gamma") {
        param.min = Some("1.0".to_string());
        param.max = Some("3.0".to_string());
        return;
    }

    if key.contains("resolutionscale") || key.contains("renderscale") {
        param.min = Some("0.25".to_string());
        param.max = Some("2.0".to_string());
        return;
    }

    if key.contains("resolution") && key.contains("size") {
        if key.ends_with('x') || key.contains("width") || key.contains("sizex") {
            param.min = Some("640".to_string());
            param.max = Some("7680".to_string());
        } else {
            param.min = Some("480".to_string());
            param.max = Some("4320".to_string());
        }
        return;
    }

    if key.contains("shadow") && key.contains("resolution") {
        param.min = Some("256".to_string());
        param.max = Some("8192".to_string());
        return;
    }

    if key.ends_with("scale") || key.contains(".scale") {
        param.min = Some("0.1".to_string());
        param.max = Some("4.0".to_string());
        return;
    }

    if key.ends_with("quality") || key.contains(".quality") {
        param.min = Some("0".to_string());
        param.max = Some(if key.contains("postprocess") || key.contains("aa") {
            "6".to_string()
        } else {
            "5".to_string()
        });
        return;
    }

    if key.starts_with("r.") {
        param.min = Some("0".to_string());
        param.max = Some("4".to_string());
    }
}

pub(crate) fn fill_generic_value_hint(param: &mut GameParameter) {
    if param.value_hint.is_some() {
        return;
    }
    if param.value_type == "bool" {
        param.value_hint = Some(crate::i18n::t(
            "True — вкл, False — выкл",
            "True — on, False — off",
        ));
        return;
    }
    if param.value_type == "enum" {
        param.value_hint = Some(crate::i18n::t("On — вкл, Off — выкл", "On — on, Off — off"));
        return;
    }
    if let (Some(min), Some(max)) = (&param.min, &param.max) {
        param.value_hint = Some(crate::i18n::t(
            &format!("Допустимо: {min} – {max}"),
            &format!("Allowed: {min} – {max}"),
        ));
    }
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

pub(crate) fn infer_range_from_value(param: &mut GameParameter) {
    if param.key == "sg.ResolutionQuality" {
        param.min = Some("25".to_string());
        param.max = Some("200".to_string());
        param.value_hint = Some(crate::i18n::t(
            "Процент render scale, не индекс 0–4",
            "Render scale percentage, not a 0–4 index",
        ));
        return;
    }

    if param.value.trim() == "-1" || param.value.trim() == "-1.0" {
        param.value_hint = Some(crate::i18n::t(
            "−1 — автоматически (движок/меню сами выбирают значение). Задайте число вручную, чтобы зафиксировать.",
            "−1 — automatic (engine/menu chooses the value). Set a number manually to lock it in.",
        ));
        return;
    }

    let Ok(num) = param.value.parse::<f64>() else {
        return;
    };

    if num.fract() != 0.0 {
        let pad = num.abs().max(0.5);
        param.min = Some(format!("{:.4}", (num - pad).max(0.0)));
        param.max = Some(format!("{:.4}", num + pad));
        return;
    }

    let n = num as i64;
    if n <= 4 && !param.key.starts_with("r.") {
        return;
    }

    param.min = Some("0".to_string());
    param.max = Some(n.saturating_mul(2).max(100).to_string());
}

pub(crate) fn is_ue5_only_catalog_key(key: &str) -> bool {
    UE5_ONLY_SG_KEYS.contains(&key) || UE5_ONLY_CVAR_KEYS.contains(&key)
}

pub(crate) fn is_hidden_ue_manual_key(key: &str) -> bool {
    HIDDEN_UE_MANUAL_KEYS
        .iter()
        .any(|hidden| key.eq_ignore_ascii_case(hidden))
}

pub(crate) fn infer_value_type(value: &str) -> String {
    if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
        "bool".to_string()
    } else if value.eq_ignore_ascii_case("on") || value.eq_ignore_ascii_case("off") {
        "enum".to_string()
    } else if value.parse::<i64>().is_ok() {
        "int".to_string()
    } else if value.parse::<f64>().is_ok() {
        "float".to_string()
    } else {
        "string".to_string()
    }
}
