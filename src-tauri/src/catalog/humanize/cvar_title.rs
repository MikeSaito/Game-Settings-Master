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
