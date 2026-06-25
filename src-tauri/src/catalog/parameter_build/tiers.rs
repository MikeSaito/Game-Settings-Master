use crate::core::models::GameParameter;

fn is_sg_quality_key(key: &str) -> bool {
    key.starts_with("sg.") && key.len() > 3 && key[3..].to_ascii_lowercase().ends_with("quality")
}

pub(crate) fn attach_scalability_tier_hints(
    parameters: &mut [GameParameter],
    engine_version: Option<&str>,
) {
    for param in parameters.iter_mut() {
        if is_sg_quality_key(&param.key) {
            param.tier_hint =
                super::super::scalability_tiers::tier_hint_for_key(&param.key, engine_version);
        }
    }
}
