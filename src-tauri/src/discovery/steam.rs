use crate::discovery::config_index::scan_local_appdata_configs;
use crate::discovery::dedupe_paths;
use crate::discovery::known_games::load_known_games;
use crate::discovery::ue_detect::{detect_unreal_engine, find_executables, UeDetectResult};
use crate::discovery::unity_detect::{detect_unity_engine, UnityDetectResult};
use crate::ini::paths::resolve_config_dir;
use crate::models::GameProfile;
use crate::unity::resolve_unity_config_dir;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn scan_steam_games() -> Vec<GameProfile> {
    let mut games: HashMap<String, GameProfile> = HashMap::new();
    let known = load_known_games();
    let steam_paths = dedupe_paths(find_steam_install_paths());

    for steam_root in steam_paths {
        let library_folders = dedupe_paths(parse_library_folders(&steam_root));
        for library in library_folders {
            let steamapps = library.join("steamapps");
            if !steamapps.exists() {
                continue;
            }
            let Ok(entries) = fs::read_dir(&steamapps) else {
                continue;
            };
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.starts_with("appmanifest_") || !name.ends_with(".acf") {
                    continue;
                }
                if let Some(game) = parse_steam_manifest(&entry.path(), &library, &known) {
                    games
                        .entry(game.id.clone())
                        .and_modify(|existing| {
                            if existing.config_dir.is_none() {
                                existing.config_dir = game.config_dir.clone();
                            }
                            if existing.exe_name.is_none() {
                                existing.exe_name = game.exe_name.clone();
                            }
                        })
                        .or_insert(game);
                }
            }
        }
    }

    games.into_values().collect()
}

fn find_steam_install_paths() -> Vec<PathBuf> {
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

fn parse_library_folders(steam_root: &Path) -> Vec<PathBuf> {
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
        if let Ok(content) = fs::read_to_string(&vdf_path) {
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
    let re = Regex::new(r#""path"\s+"([^"]+)""#).unwrap();
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

fn parse_steam_manifest(
    manifest_path: &Path,
    library: &Path,
    known: &std::collections::HashMap<String, crate::discovery::known_games::KnownGameEntry>,
) -> Option<GameProfile> {
    let content = fs::read_to_string(manifest_path).ok()?;
    let app_id = extract_acf_value(&content, "appid")?;
    let name = extract_acf_value(&content, "name").unwrap_or_else(|| "Unknown".to_string());
    let installdir = extract_acf_value(&content, "installdir")?;
    let build_id = extract_acf_value(&content, "buildid");
    let install_path = library.join("steamapps").join("common").join(&installdir);

    if !install_path.exists() {
        return None;
    }

    let known_forza = known.get(&app_id).and_then(|k| k.engine_family.as_deref()) == Some("forza");
    let is_forza = known_forza || crate::forza::is_forza_install(&install_path);

    let unity = detect_unity_engine(&install_path);
    let is_unity = !is_forza && unity != UnityDetectResult::NotUnity;
    let ue = detect_unreal_engine(&install_path);
    let is_ue = !is_forza && !is_unity && ue != UeDetectResult::NotUe;

    let exe_name = find_executables(&install_path)
        .first()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));

    let config_dir = if is_forza {
        crate::forza::resolve_forza_config_dir(Some(&app_id))
    } else if is_unity {
        resolve_unity_config_dir(
            &install_path,
            exe_name.as_deref(),
            Some(&name),
            Some(&app_id),
        )
    } else {
        resolve_config_dir(
            &install_path,
            exe_name.as_deref(),
            Some(&name),
            Some(&app_id),
        )
        .or_else(|| {
            let index = scan_local_appdata_configs();
            crate::discovery::config_index::match_config_from_index(
                &index,
                &crate::discovery::config_index::build_match_candidates(
                    &install_path,
                    exe_name.as_deref(),
                    Some(&name),
                    known.get(&app_id).map(|k| k.local_app_folder.as_str()),
                ),
            )
        })
    }
    .map(|p| p.to_string_lossy().to_string());

    let profile = GameProfile {
        id: format!("steam-{app_id}"),
        name,
        source: "steam".to_string(),
        install_dir: install_path.to_string_lossy().to_string(),
        config_dir,
        exe_name,
        is_ue,
        is_unity,
        is_author_curated: is_forza
            || crate::discovery::known_games::is_author_curated_app(&app_id),
        possible_unity: unity == UnityDetectResult::Probable,
        possible_ue: ue == UeDetectResult::Probable,
        cover_url: Some(crate::covers::steam_header_url(&app_id)),
        custom_cover: None,
        build_id,
        engine_family: if is_forza {
            "forza".to_string()
        } else if is_unity {
            "unity".to_string()
        } else {
            "unknown".to_string()
        },
        engine_version: None,
    };
    Some(profile)
}

fn extract_acf_value(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("\"{key}\"")) {
            return extract_quoted_value(trimmed);
        }
    }
    None
}

fn extract_quoted_value(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split('"').collect();
    if parts.len() >= 4 {
        Some(parts[3].replace("\\\\", "\\"))
    } else {
        None
    }
}
