use crate::discovery::dedupe_paths;
use crate::discovery::drives::scannable_drive_roots;
use std::path::{Path, PathBuf};

const STEAM_LIBRARY_RELATIVE_PATHS: &[&str] = &[
    "SteamLibrary",
    "Steam",
    "Program Files (x86)\\Steam",
    "Program Files\\Steam",
    "Games\\SteamLibrary",
    "Games\\Steam",
];

pub(crate) fn find_steam_install_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(windows)]
    {
        for (hive, value) in [
            (r"HKCU\Software\Valve\Steam", "SteamPath"),
            (r"HKLM\SOFTWARE\WOW6432Node\Valve\Steam", "InstallPath"),
            (r"HKLM\SOFTWARE\Valve\Steam", "InstallPath"),
        ] {
            if let Some(path) = read_registry_path(hive, value) {
                paths.push(path);
            }
        }
    }

    if let Ok(program_files) = std::env::var("ProgramFiles(x86)") {
        paths.push(PathBuf::from(program_files).join("Steam"));
    }
    if let Ok(program_files) = std::env::var("ProgramFiles") {
        paths.push(PathBuf::from(program_files).join("Steam"));
    }

    for drive in scannable_drive_roots() {
        for rel in STEAM_LIBRARY_RELATIVE_PATHS {
            let candidate = drive.join(rel);
            if candidate.join("steam.exe").exists() || candidate.join("Steam.exe").exists() {
                paths.push(candidate);
            }
        }
    }

    paths.retain(|p| p.exists());
    dedupe_paths(paths)
}

/// All Steam library roots: registry/VDF paths plus common folders on every local drive.
pub(crate) fn collect_all_steam_library_roots() -> Vec<PathBuf> {
    let mut folders = Vec::new();
    for steam_root in dedupe_paths(find_steam_install_paths()) {
        folders.extend(parse_library_folders(&steam_root));
    }
    folders.extend(discover_steam_libraries_on_drives());
    dedupe_paths(folders)
}

pub(crate) fn parse_library_folders(steam_root: &Path) -> Vec<PathBuf> {
    let mut folders = vec![steam_root.to_path_buf()];
    let vdf_paths = [
        steam_root.join("steamapps").join("libraryfolders.vdf"),
        steam_root.join("config").join("libraryfolders.vdf"),
        steam_root.join("SteamApps").join("libraryfolders.vdf"),
    ];

    for vdf_path in vdf_paths {
        if !vdf_path.exists() {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&vdf_path) {
            for path in parse_all_vdf_paths(&content) {
                let expanded = expand_steam_path(&path);
                if is_steam_library_root(&expanded) {
                    folders.push(expanded);
                }
            }
        }
    }

    folders.sort();
    dedupe_paths(folders)
}

fn discover_steam_libraries_on_drives() -> Vec<PathBuf> {
    let mut found = Vec::new();
    for drive in scannable_drive_roots() {
        for rel in STEAM_LIBRARY_RELATIVE_PATHS {
            let candidate = drive.join(rel);
            if is_steam_library_root(&candidate) {
                found.push(candidate);
            }
        }
    }
    found
}

fn is_steam_library_root(path: &Path) -> bool {
    path.join("steamapps").is_dir()
}

#[cfg(windows)]
fn read_registry_path(hive_key: &str, value_name: &str) -> Option<PathBuf> {
    let output = crate::core::process_util::hidden_command("reg")
        .args(["query", hive_key, "/v", value_name])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_registry_path_line(&String::from_utf8_lossy(&output.stdout), value_name)
        .filter(|path| path.exists())
}

#[cfg(windows)]
fn parse_registry_path_line(text: &str, value_name: &str) -> Option<PathBuf> {
    for line in text.lines() {
        if !line.contains(value_name) {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if let Some(path) = parts.last() {
            return Some(PathBuf::from(path.replace('/', "\\")));
        }
    }
    None
}

fn parse_all_vdf_paths(content: &str) -> Vec<String> {
    let re = regex::Regex::new(r#""path"\s+"([^"]+)""#).unwrap();
    let mut paths = Vec::new();
    for cap in re.captures_iter(content) {
        if let Some(m) = cap.get(1) {
            paths.push(m.as_str().replace("\\\\", "\\"));
        }
    }
    paths.sort();
    paths.dedup();
    paths
}

fn expand_steam_path(path: &str) -> PathBuf {
    PathBuf::from(path.replace("\\\\", "\\"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parse_libraryfolders_vdf_extracts_multiple_drives() {
        let vdf = r#"
"libraryfolders"
{
    "0"
    {
        "path"		"C:\\Program Files (x86)\\Steam"
    }
    "1"
    {
        "path"		"D:\\SteamLibrary"
    }
    "2"
    {
        "path"		"E:/Games/SteamLibrary"
    }
}
"#;
        let paths = parse_all_vdf_paths(vdf);
        assert_eq!(paths.len(), 3);
        assert!(paths.contains(&"C:\\Program Files (x86)\\Steam".to_string()));
        assert!(paths.contains(&"D:\\SteamLibrary".to_string()));
        assert!(paths.contains(&"E:/Games/SteamLibrary".to_string()));
    }

    #[test]
    fn expand_steam_path_normalizes_slashes() {
        assert_eq!(
            expand_steam_path("D:\\\\SteamLibrary"),
            PathBuf::from("D:\\SteamLibrary")
        );
    }

    #[cfg(windows)]
    #[test]
    fn parse_registry_path_line_reads_install_path() {
        let stdout = r#"
HKEY_LOCAL_MACHINE\SOFTWARE\Valve\Steam
    InstallPath    REG_SZ    D:\Steam
"#;
        assert_eq!(
            parse_registry_path_line(stdout, "InstallPath"),
            Some(PathBuf::from("D:\\Steam"))
        );
    }

    #[test]
    fn collect_all_steam_library_roots_does_not_panic_without_steam() {
        let _ = collect_all_steam_library_roots();
    }

    #[test]
    fn is_steam_library_root_requires_steamapps_dir() {
        let temp = tempfile::tempdir().unwrap();
        assert!(!is_steam_library_root(temp.path()));
        std::fs::create_dir_all(temp.path().join("steamapps")).unwrap();
        assert!(is_steam_library_root(temp.path()));
    }
}

