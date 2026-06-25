mod executables;
mod markers;
mod non_game;

use std::path::Path;

pub use executables::find_executables;
pub use non_game::is_non_game_install;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UeDetectResult {
    Confirmed,
    Probable,
    NotUe,
}

pub fn detect_unreal_engine(install_dir: &Path) -> UeDetectResult {
    if !install_dir.is_dir() {
        return UeDetectResult::NotUe;
    }

    if markers::is_definitely_not_ue(install_dir) {
        return UeDetectResult::NotUe;
    }

    if is_non_game_install(install_dir, "", None) {
        return UeDetectResult::NotUe;
    }

    if install_dir.join("Engine").join("Binaries").exists() {
        return UeDetectResult::Confirmed;
    }

    if markers::has_ue_default_config(install_dir) {
        return UeDetectResult::Confirmed;
    }

    if markers::has_win64_shipping_exe(install_dir) {
        return UeDetectResult::Confirmed;
    }

    if markers::has_ue_content_paks(install_dir) {
        return UeDetectResult::Confirmed;
    }

    if markers::has_ue_project_layout(install_dir) {
        return UeDetectResult::Probable;
    }

    UeDetectResult::NotUe
}

#[cfg(test)]
#[path = "ue_detect_tests.rs"]
mod tests;
