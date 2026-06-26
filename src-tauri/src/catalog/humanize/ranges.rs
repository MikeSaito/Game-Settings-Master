use crate::core::models::GameParameter;

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
