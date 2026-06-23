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
