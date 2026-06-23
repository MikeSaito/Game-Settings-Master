use crate::discovery::dedupe_paths;
use std::path::{Path, PathBuf};

pub(crate) fn find_steam_install_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(windows)]
    {
        if let Ok(output) = crate::process_util::hidden_command("reg")
            .args(["query", r"HKCU\Software\Valve\Steam", "/v", "SteamPath"])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    if line.contains("SteamPath") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if let Some(path) = parts.last() {
                            let p = PathBuf::from(path.replace('/', "\\"));
                            if p.exists() {
                                paths.push(p);
                            }
                        }
                    }
                }
            }
        }
    }

    if let Ok(program_files) = std::env::var("ProgramFiles(x86)") {
        paths.push(PathBuf::from(program_files).join("Steam"));
    }
    if let Ok(program_files) = std::env::var("ProgramFiles") {
        paths.push(PathBuf::from(program_files).join("Steam"));
    }

    paths.retain(|p| p.exists());
    dedupe_paths(paths)
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
                if expanded.exists() {
                    folders.push(expanded);
                }
            }
        }
    }

    folders.sort();
    dedupe_paths(folders)
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
