use crate::core::models::{GameProfile, LaunchTarget};
use crate::launch::LaunchResult;
use std::path::{Path, PathBuf};

pub(crate) fn validate_executable(profile: &GameProfile, value: &str) -> Result<PathBuf, String> {
    let root = Path::new(&profile.install_dir);
    let path = root.join(value);
    if !path.is_file()
        || !crate::fs_util::path_within_root(root, &path)
        || !path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
    {
        return Err(crate::i18n::t(
            "Выберите EXE внутри папки установки игры",
            "Select an EXE inside the game installation folder",
        ));
    }
    let metadata = std::fs::metadata(&path).map_err(|error| error.to_string())?;
    if metadata.len() < 2 {
        return Err("Invalid executable".into());
    }
    let mut header = [0u8; 2];
    use std::io::Read;
    std::fs::File::open(&path)
        .and_then(|mut file| file.read_exact(&mut header))
        .map_err(|error| error.to_string())?;
    if header != *b"MZ" {
        return Err(crate::i18n::t(
            "Файл не является Windows EXE",
            "File is not a Windows executable",
        ));
    }
    path.canonicalize().map_err(|error| error.to_string())
}

pub(crate) fn launch_exe(profile: &GameProfile, value: &str) -> Result<LaunchResult, String> {
    let path = validate_executable(profile, value)?;
    let working_dir = validate_working_dir(profile)?;
    std::process::Command::new(&path)
        .current_dir(working_dir)
        .spawn()
        .map_err(|error| error.to_string())?;
    Ok(LaunchResult {
        launcher: profile.source.clone(),
        detail: path.to_string_lossy().into(),
        warning: None,
    })
}

pub(crate) fn validate_working_dir(profile: &GameProfile) -> Result<PathBuf, String> {
    let root = Path::new(&profile.install_dir);
    let directory = root.join(
        profile
            .launch_target
            .as_ref()
            .and_then(|target| target.working_dir.as_deref())
            .unwrap_or(""),
    );
    if !directory.is_dir() || !crate::fs_util::path_within_root(root, &directory) {
        return Err(crate::i18n::t(
            "Рабочая папка должна находиться внутри установки игры",
            "Working directory must be inside the game installation",
        ));
    }
    directory.canonicalize().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_game_executables(game_id: String) -> Result<Vec<String>, String> {
    let profile =
        crate::commands::helpers::find_profile_by_id(&game_id)?.ok_or("Game not found")?;
    let root = Path::new(&profile.install_dir);
    let mut files: Vec<_> = crate::discovery::find_executables(root)
        .into_iter()
        .filter(|path| crate::fs_util::path_within_root(root, path))
        .filter_map(|path| {
            path.strip_prefix(root)
                .ok()
                .map(|relative| relative.to_string_lossy().into())
        })
        .collect();
    files.sort();
    files.dedup();
    Ok(files)
}

#[tauri::command]
pub fn set_game_executable(game_id: String, exe_path: String) -> Result<(), String> {
    let mut profile =
        crate::commands::helpers::find_profile_by_id(&game_id)?.ok_or("Game not found")?;
    if profile
        .launch_target
        .as_ref()
        .is_some_and(|target| target.kind == "package")
    {
        return Err(crate::i18n::t(
            "Зарегистрированная игра Xbox запускается через Windows",
            "Registered Xbox games launch through Windows",
        ));
    }
    let path = validate_executable(&profile, &exe_path)?;
    let root = Path::new(&profile.install_dir)
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let relative = path.strip_prefix(root).map_err(|error| error.to_string())?;
    profile.launch_target = Some(LaunchTarget {
        working_dir: None,
        kind: "exe".into(),
        value: relative.to_string_lossy().into(),
    });
    profile.exe_name = path.file_name().map(|name| name.to_string_lossy().into());
    crate::profiles::save_profile(&profile)?;
    crate::discovery::invalidate_game_scan_cache();
    Ok(())
}

#[cfg(windows)]
pub(crate) fn launch_package(aumid: &str) -> Result<LaunchResult, String> {
    use windows::Win32::System::Com::*;
    use windows::Win32::UI::Shell::*;
    if aumid.len() > 512
        || !aumid.contains('!')
        || !aumid
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '!'))
    {
        return Err("Invalid package application id".into());
    }
    unsafe {
        let initialized = CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok();
        let result = (|| -> windows::core::Result<u32> {
            let manager: IApplicationActivationManager =
                CoCreateInstance(&ApplicationActivationManager, None, CLSCTX_INPROC_SERVER)?;
            let id = windows::core::HSTRING::from(aumid);
            manager.ActivateApplication(
                windows::core::PCWSTR(id.as_ptr()),
                windows::core::w!(""),
                AO_NONE,
            )
        })();
        if initialized {
            CoUninitialize();
        }
        result
            .map(|_| LaunchResult {
                launcher: "xbox".into(),
                detail: aumid.into(),
                warning: None,
            })
            .map_err(|error| error.to_string())
    }
}
#[cfg(not(windows))]
pub(crate) fn launch_package(_aumid: &str) -> Result<LaunchResult, String> {
    Err("Package launch requires Windows".into())
}
