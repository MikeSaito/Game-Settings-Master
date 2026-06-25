use super::steam::steam_header_url;
use std::path::Path;

pub fn enrich_cover(profile: &mut crate::core::models::GameProfile) {
    if profile.custom_cover.is_some() {
        return;
    }

    if profile.cover_url.is_some() {
        return;
    }

    if let Some(app_id) = profile.id.strip_prefix("steam-") {
        profile.cover_url = Some(steam_header_url(app_id));
    }
}

pub fn merge_saved_cover(
    existing: &mut crate::core::models::GameProfile,
    saved: &crate::core::models::GameProfile,
) {
    if let Some(custom) = &saved.custom_cover {
        if Path::new(custom).exists() {
            existing.custom_cover = Some(custom.clone());
        }
    }
}
