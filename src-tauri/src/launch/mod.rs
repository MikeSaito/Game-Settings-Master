mod epic;
pub(crate) mod executable;
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
    if let Some(target) = &profile.launch_target {
        return match target.kind.as_str() {
            "exe" => executable::launch_exe(profile, &target.value),
            "package" if profile.source == "xbox" => executable::launch_package(&target.value),
            _ => Err("Unsupported launch target".into()),
        };
    }
    match profile.source.as_str() {
        "steam" => launch_steam_profile(profile),
        "epic" => launch_epic_profile(profile),
        "manual" => launch_manual_profile(profile),
        "gog" => launch_selected_executable(profile),
        "xbox" => Err(crate::i18n::t(
            "Для установки Xbox недоступна зарегистрированная цель запуска",
            "No registered launch target is available for this Xbox installation",
        )),
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
    launch_selected_executable(profile)
}

fn launch_selected_executable(profile: &GameProfile) -> Result<LaunchResult, String> {
    let files = executable::list_game_executables(profile.id.clone())?;
    if files.len() == 1 {
        return executable::launch_exe(profile, &files[0]);
    }
    Err(crate::i18n::t(
        "Выберите игровой EXE в параметрах запуска",
        "Select the game EXE in launch options",
    ))
}

#[cfg(test)]
#[path = "launch_tests.rs"]
mod tests;
