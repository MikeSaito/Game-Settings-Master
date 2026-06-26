use crate::core::models::GameProfile;
use crate::ini::paths::resolve_config_dir;
use uuid::Uuid;

use super::ue_detect::{detect_unreal_engine, is_non_game_install, UeDetectResult};

pub fn profile_from_manual_path(name: &str, install_dir: &str) -> Result<GameProfile, String> {
    let display_name = name.trim();
    if display_name.is_empty() || display_name.len() > 120 {
        return Err(crate::i18n::t(
            "Недопустимое имя игры (1–120 символов)",
            "Invalid game name (1–120 characters)",
        ));
    }
    let path = std::path::PathBuf::from(install_dir);
    if !path.exists() {
        return Err(crate::i18n::t(
            "Указанная папка не существует",
            "The specified folder does not exist",
        ));
    }

    if is_non_game_install(&path, display_name, None) {
        return Err(crate::i18n::t(
            "Это установка Unreal Engine или инструмент Epic, а не игра. Укажите папку с игрой.",
            "This is an Unreal Engine installation or Epic tool, not a game. Point to the game folder.",
        ));
    }

    let ue = detect_unreal_engine(&path);
    if ue == UeDetectResult::NotUe {
        return Err(crate::i18n::t(
            "Папка не похожа на Unreal Engine (нет Shipping.exe и т.д.)",
            "Folder does not look like Unreal Engine (no Shipping.exe, etc.)",
        ));
    }

    let config_dir = resolve_config_dir(&path, None, Some(display_name), None)
        .map(|p| p.to_string_lossy().to_string());

    Ok(GameProfile {
        id: format!("manual-{}", Uuid::new_v4()),
        name: display_name.to_string(),
        source: "manual".to_string(),
        install_dir: install_dir.to_string(),
        config_dir,
        exe_name: None,
        is_ue: true,
        possible_ue: ue == UeDetectResult::Probable,
        cover_url: None,
        custom_cover: None,
        build_id: None,
        engine_family: "unknown".to_string(),
        engine_version: None,
    })
}
