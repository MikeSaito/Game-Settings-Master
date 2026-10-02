use crate::core::models::{GameProfile, LaunchTarget};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static WARNINGS: Mutex<Vec<String>> = Mutex::new(Vec::new());
pub(crate) fn warn(message: String) {
    if let Ok(mut warnings) = WARNINGS.lock() {
        if warnings.len() < 32 && !warnings.contains(&message) {
            warnings.push(message);
        }
    }
}
pub(crate) fn clear_warnings() {
    if let Ok(mut warnings) = WARNINGS.lock() {
        warnings.clear();
    }
}
#[tauri::command]
pub fn get_discovery_warnings() -> Vec<String> {
    WARNINGS
        .lock()
        .map(|warnings| warnings.clone())
        .unwrap_or_default()
}

fn profile(
    id: String,
    name: &str,
    dir: &Path,
    source: &str,
    launch_target: Option<LaunchTarget>,
) -> Option<GameProfile> {
    let mut game = super::profile_from_manual_path(name, &dir.to_string_lossy()).ok()?;
    game.id = id;
    game.source = source.into();
    game.launch_target = launch_target;
    if source == "gog" {
        if let Some(metadata) = gog_metadata(dir) {
            if let Some(id) = metadata.id {
                game.id = format!("gog-{id}");
            }
            if let Some(name) = metadata.name {
                game.name = name;
            }
            game.launch_target = metadata.target.or(game.launch_target);
        }
    }
    game.exe_name = super::ue_detect::find_executables(dir)
        .first()
        .and_then(|exe| exe.file_name())
        .map(|name| name.to_string_lossy().into());
    Some(game)
}

struct GogMetadata {
    id: Option<String>,
    name: Option<String>,
    target: Option<LaunchTarget>,
}
fn metadata_text(path: &Path) -> Option<String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > 1024 * 1024 {
        return None;
    }
    String::from_utf8(bytes).ok()
}
fn gog_metadata(dir: &Path) -> Option<GogMetadata> {
    let mut files = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .take(4096)
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
            (name.starts_with("goggame-")
                && name.ends_with(".info")
                && crate::fs_util::path_within_root(dir, &entry.path()))
            .then(|| entry.path())
        })
        .collect::<Vec<_>>();
    files.sort();
    for file in files.into_iter().take(64) {
        let Some(text) = metadata_text(&file) else {
            continue;
        };
        let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        let id = data.get("gameId").and_then(|id| {
            id.as_str()
                .map(str::to_owned)
                .or_else(|| id.as_u64().map(|id| id.to_string()))
        });
        let root_id = data.get("rootGameId").and_then(serde_json::Value::as_str);
        if root_id.is_some_and(|root| id.as_deref() != Some(root)) {
            continue;
        }
        let id = id.filter(|id| {
            !id.is_empty() && id.len() < 100 && id.chars().all(|ch| ch.is_ascii_digit())
        });
        let name = data
            .get("name")
            .and_then(serde_json::Value::as_str)
            .filter(|name| !name.trim().is_empty() && name.len() <= 256)
            .map(str::to_owned);
        let primary: Vec<_> = data
            .get("playTasks")
            .or_else(|| data.get("tasks"))
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter(|task| {
                task.get("isPrimary").and_then(serde_json::Value::as_bool) == Some(true)
                    && task
                        .get("type")
                        .and_then(serde_json::Value::as_str)
                        .is_none_or(|kind| kind == "FileTask")
            })
            .collect();
        let target = if primary.len() == 1 {
            let task = primary[0];
            let executable = task
                .get("path")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned);
            let working_dir = task
                .get("workingDir")
                .and_then(serde_json::Value::as_str)
                .filter(|value| !value.is_empty())
                .map(str::to_owned);
            let cwd = dir.join(working_dir.as_deref().unwrap_or(""));
            // Tasks requiring arguments remain available through explicit EXE selection.
            let no_arguments = task
                .get("arguments")
                .is_none_or(|arguments| arguments.is_null() || arguments.as_str() == Some(""));
            executable
                .filter(|value| {
                    no_arguments
                        && cwd.is_dir()
                        && crate::fs_util::path_within_root(dir, &cwd)
                        && dir.join(value).is_file()
                        && dir
                            .join(value)
                            .extension()
                            .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
                        && crate::fs_util::path_within_root(dir, &dir.join(value))
                })
                .map(|value| LaunchTarget {
                    kind: "exe".into(),
                    value,
                    working_dir,
                })
        } else {
            None
        };
        if id.is_some() || name.is_some() {
            return Some(GogMetadata { id, name, target });
        }
    }
    None
}

#[cfg(windows)]
pub fn scan_gog() -> Vec<GameProfile> {
    use winreg::{enums::*, RegKey};
    let mut games = Vec::new();
    let machine = RegKey::predef(HKEY_LOCAL_MACHINE);
    for view in [KEY_WOW64_32KEY, KEY_WOW64_64KEY] {
        let Ok(root) = machine.open_subkey_with_flags(r"SOFTWARE\GOG.com\Games", KEY_READ | view)
        else {
            continue;
        };
        for id in root.enum_keys().flatten().take(2048) {
            let Ok(key) = root.open_subkey(&id) else {
                continue;
            };
            let Ok(path) = key.get_value::<String, _>("path") else {
                continue;
            };
            let name = key
                .get_value::<String, _>("gameName")
                .unwrap_or_else(|_| id.clone());
            let dir = PathBuf::from(path);
            let exe = key
                .get_value::<String, _>("exe")
                .ok()
                .map(PathBuf::from)
                .map(|exe| {
                    if exe.is_absolute() {
                        exe
                    } else {
                        dir.join(exe)
                    }
                });
            let target = exe
                .filter(|exe| exe.is_file() && crate::fs_util::path_within_root(&dir, exe))
                .and_then(|exe| {
                    exe.strip_prefix(&dir).ok().map(|relative| LaunchTarget {
                        working_dir: None,
                        kind: "exe".into(),
                        value: relative.to_string_lossy().into(),
                    })
                });
            if let Some(game) = profile(format!("gog-{id}"), &name, &dir, "gog", target) {
                games.push(game);
            }
        }
    }
    // Uninstall records cover offline installers without a Galaxy registration.
    for hive in [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
        let hive = RegKey::predef(hive);
        for view in [KEY_WOW64_32KEY, KEY_WOW64_64KEY] {
            let Ok(root) = hive.open_subkey_with_flags(
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
                KEY_READ | view,
            ) else {
                continue;
            };
            for id in root
                .enum_keys()
                .flatten()
                .take(4096)
                .filter(|id| id.ends_with("_is1"))
            {
                let Ok(key) = root.open_subkey(&id) else {
                    continue;
                };
                let publisher = key.get_value::<String, _>("Publisher").unwrap_or_default();
                if !publisher.to_ascii_lowercase().contains("gog") {
                    continue;
                }
                let Ok(path) = key.get_value::<String, _>("InstallLocation") else {
                    continue;
                };
                let dir = PathBuf::from(path);
                let name = key
                    .get_value::<String, _>("DisplayName")
                    .unwrap_or_else(|_| id.clone());
                if let Some(game) = profile(
                    format!(
                        "gog-{}",
                        crate::changes::digest(dir.to_string_lossy().as_bytes())
                    ),
                    &name,
                    &dir,
                    "gog",
                    None,
                ) {
                    games.push(game);
                }
            }
        }
    }
    for drive in super::drives::scannable_drive_roots() {
        for root in [
            drive.join("GOG Games"),
            drive.join("Games/GOG"),
            drive.join("Program Files (x86)/GOG Galaxy/Games"),
        ] {
            let Ok(entries) = std::fs::read_dir(&root) else {
                continue;
            };
            for entry in entries.flatten().take(2048) {
                if !entry.file_type().is_ok_and(|kind| kind.is_dir())
                    || !crate::fs_util::path_within_root(&root, &entry.path())
                {
                    continue;
                }
                if let Some(metadata) = gog_metadata(&entry.path()) {
                    let name = metadata
                        .name
                        .unwrap_or_else(|| entry.file_name().to_string_lossy().into_owned());
                    let id = metadata.id.unwrap_or_else(|| {
                        crate::changes::digest(entry.path().to_string_lossy().as_bytes())
                    });
                    if let Some(game) = profile(
                        format!("gog-{id}"),
                        &name,
                        &entry.path(),
                        "gog",
                        metadata.target,
                    ) {
                        games.push(game);
                    }
                }
            }
        }
    }
    games
}
#[cfg(not(windows))]
pub fn scan_gog() -> Vec<GameProfile> {
    Vec::new()
}

fn parse_xml(text: &str) -> Option<roxmltree::Document<'_>> {
    if text.len() > 1024 * 1024 {
        return None;
    }
    roxmltree::Document::parse_with_options(
        text,
        roxmltree::ParsingOptions {
            allow_dtd: false,
            nodes_limit: 10_000,
            ..Default::default()
        },
    )
    .ok()
}

fn app_id(text: &str) -> Option<String> {
    parse_xml(text)?
        .descendants()
        .find(|node| node.is_element() && node.tag_name().name() == "Application")
        .and_then(|node| node.attribute("Id"))
        .map(str::to_owned)
}

fn gaming_root(drive: &Path) -> Option<PathBuf> {
    let marker = drive.join(".GamingRoot");
    let bytes = crate::fs_util::read_file_bytes(&marker).ok()?;
    if bytes.len() < 10
        || bytes.len() > 8192
        || &bytes[..4] != b"RGBX"
        || u32::from_le_bytes(bytes[4..8].try_into().ok()?) != 1
        || bytes.len() % 2 != 0
    {
        return None;
    }
    let units = bytes[8..]
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect::<Vec<_>>();
    let text = String::from_utf16(&units).ok()?;
    let root = drive.join(text.trim_end_matches('\0'));
    if root != drive && root.is_dir() && crate::fs_util::path_within_root(drive, &root) {
        Some(root)
    } else {
        None
    }
}

#[cfg(windows)]
pub fn scan_xbox() -> Vec<GameProfile> {
    use windows::Management::Deployment::PackageManager;
    use windows::Win32::System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED};
    let mut games = Vec::new();
    let mut packages = Vec::new();
    let initialized = unsafe { RoInitialize(RO_INIT_MULTITHREADED).is_ok() };
    if initialized {
        let found = PackageManager::new().and_then(|manager| {
            manager.FindPackagesByUserSecurityId(&windows::core::HSTRING::new())
        });
        match found {
            Ok(found) => {
                for package in found.into_iter().take(2048) {
                    if package.IsFramework().unwrap_or(true)
                        || package.IsResourcePackage().unwrap_or(true)
                    {
                        continue;
                    }
                    let Ok(id) = package.Id() else {
                        continue;
                    };
                    let Ok(family) = id.FamilyName() else {
                        continue;
                    };
                    let Ok(location) = package.InstalledLocation().and_then(|folder| folder.Path())
                    else {
                        continue;
                    };
                    let dir = PathBuf::from(location.to_string());
                    let manifest = metadata_text(&dir.join("AppxManifest.xml"));
                    let aumid = manifest
                        .and_then(|text| app_id(&text))
                        .map(|app| format!("{family}!{app}"));
                    let name = package
                        .DisplayName()
                        .map(|name| name.to_string())
                        .unwrap_or_else(|_| family.to_string());
                    packages.push((dir.clone(), family.to_string(), aumid.clone()));
                    let target = aumid.map(|value| LaunchTarget {
                        working_dir: None,
                        kind: "package".into(),
                        value,
                    });
                    if let Some(game) =
                        profile(format!("xbox-{family}"), &name, &dir, "xbox", target)
                    {
                        games.push(game);
                    }
                }
            }
            Err(error) => warn(format!("Xbox: {error}")),
        }
        unsafe {
            RoUninitialize();
        }
    } else {
        warn(crate::i18n::t(
            "Xbox: список пакетов недоступен",
            "Xbox: package enumeration unavailable",
        ));
    }
    for drive in super::drives::scannable_drive_roots() {
        let mut roots = vec![drive.join("XboxGames")];
        if let Some(root) = gaming_root(&drive) {
            roots.push(root);
        }
        for root in super::dedupe_paths(roots) {
            let entries = match std::fs::read_dir(&root) {
                Ok(entries) => entries,
                Err(error) => {
                    if error.kind() != std::io::ErrorKind::NotFound {
                        warn(format!("Xbox: {}: {error}", root.display()));
                    }
                    continue;
                }
            };
            for entry in entries.flatten().take(2048) {
                if !entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                    continue;
                }
                let dir = entry.path().join("Content");
                if !dir.is_dir() || !crate::fs_util::path_within_root(&root, &dir) {
                    continue;
                }
                let config = metadata_text(&dir.join("MicrosoftGame.config"));
                let Some(config) = config else {
                    continue;
                };
                let Some(xml) = parse_xml(&config) else {
                    continue;
                };
                let name = xml
                    .descendants()
                    .find(|node| node.is_element() && node.tag_name().name() == "ShellVisuals")
                    .and_then(|node| node.attribute("DefaultDisplayName"))
                    .unwrap_or("");
                let name = if name.is_empty() {
                    entry.file_name().to_string_lossy().into_owned()
                } else {
                    name.into()
                };
                let package = packages
                    .iter()
                    .find(|(path, _, _)| crate::fs_util::path_within_root(&entry.path(), path));
                let target = package
                    .and_then(|(_, _, aumid)| aumid.clone())
                    .map(|value| LaunchTarget {
                        working_dir: None,
                        kind: "package".into(),
                        value,
                    });
                let id = package
                    .map(|(_, family, _)| format!("xbox-{family}"))
                    .unwrap_or_else(|| {
                        format!(
                            "xbox-{}",
                            crate::changes::digest(dir.to_string_lossy().as_bytes())
                        )
                    });
                if let Some(game) = profile(id, &name, &dir, "xbox", target) {
                    games.push(game);
                }
            }
        }
    }
    games
}
#[cfg(not(windows))]
pub fn scan_xbox() -> Vec<GameProfile> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn manifests_reject_external_entities_and_extract_application_id() {
        assert_eq!(
            app_id("<Package><Applications><Application Id='Game'/></Applications></Package>"),
            Some("Game".into())
        );
        assert!(
            parse_xml("<!DOCTYPE x [<!ENTITY data SYSTEM 'file:///private'>]><x>&data;</x>")
                .is_none()
        );
    }
    #[test]
    fn gog_local_metadata_extracts_primary_task_and_rejects_path_escape() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("bin")).unwrap();
        std::fs::write(root.path().join("bin/game.EXE"), b"MZ").unwrap();
        let metadata = serde_json::json!({ "gameId": "123", "rootGameId": "123", "name": "GOG Fixture", "playTasks": [{ "isPrimary": true, "type": "FileTask", "path": "bin/game.EXE", "workingDir": "bin" }] });
        std::fs::write(root.path().join("goggame-123.info"), metadata.to_string()).unwrap();
        let parsed = gog_metadata(root.path()).unwrap();
        assert_eq!(parsed.id.as_deref(), Some("123"));
        assert_eq!(
            parsed.target.as_ref().unwrap().working_dir.as_deref(),
            Some("bin")
        );
        let mut escaped = metadata;
        escaped["playTasks"][0]["path"] = "../outside.exe".into();
        std::fs::write(root.path().join("goggame-123.info"), escaped.to_string()).unwrap();
        assert!(gog_metadata(root.path()).unwrap().target.is_none());
    }
}
