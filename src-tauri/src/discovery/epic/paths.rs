use std::path::PathBuf;

pub(crate) fn epic_manifest_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(program_data) = std::env::var("ProgramData") {
        dirs.push(
            PathBuf::from(&program_data)
                .join("Epic")
                .join("EpicGamesLauncher")
                .join("Data")
                .join("Manifests"),
        );
    }
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        dirs.push(
            PathBuf::from(&local_app_data)
                .join("EpicGamesLauncher")
                .join("Data")
                .join("Manifests"),
        );
    }
    dirs
}
