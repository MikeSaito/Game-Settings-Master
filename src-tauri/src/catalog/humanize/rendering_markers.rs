use std::sync::OnceLock;

static MARKERS: OnceLock<Vec<String>> = OnceLock::new();

fn markers() -> &'static [String] {
    MARKERS.get_or_init(|| {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../shared/game_rendering_key_markers.json"
        )))
        .expect("shared/game_rendering_key_markers.json must be valid JSON array")
    })
}

pub(crate) fn is_game_rendering_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    markers().iter().any(|needle| lower.contains(needle))
}
