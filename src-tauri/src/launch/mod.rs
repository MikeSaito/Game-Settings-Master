mod epic;
mod steam;
mod types;
mod url;

use crate::core::models::GameProfile;

pub use epic::{launch_epic_app_name, validate_epic_app_name};
pub use steam::launch_steam_app_id;
pub use types::LaunchResult;

use epic::{find_epic_app_name_for_install, launch_epic_profile};
use steam::{find_steam_app_id_for_install, launch_steam_profile};

pub fn launch_game(profile: &GameProfile) -> Result<LaunchResult, String> {
    match profile.source.as_str() {
        "steam" => launch_steam_profile(profile),
        "epic" => launch_epic_profile(profile),
        "manual" => launch_manual_profile(profile),
        other => Err(crate::i18n::t(
            &format!("Запуск через магазин не поддерживается для источника «{other}»"),
            &format!("Store launch is not supported for source «{other}»"),
        )),
    }
}

fn launch_manual_profile(profile: &GameProfile) -> Result<LaunchResult, String> {
    if let Some(app_id) = find_steam_app_id_for_install(&profile.install_dir) {
        return launch_steam_app_id(&app_id);
    }
    if let Some(app_name) = find_epic_app_name_for_install(&profile.install_dir) {
        return launch_epic_app_name(&app_name);
    }
    Err(crate::i18n::t(
        "Не удалось определить лаунчер. Добавьте игру через сканирование Steam/Epic или укажите папку из steamapps/common.",
        "Could not determine launcher. Add the game via Steam/Epic scan or point to a folder under steamapps/common.",
    ))
}

#[cfg(test)]
#[path = "launch_tests.rs"]
mod tests;
