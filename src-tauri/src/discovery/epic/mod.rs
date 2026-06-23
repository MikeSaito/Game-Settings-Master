mod manifest;
mod paths;
mod signal;

use crate::discovery::merge_game_profile;
use crate::core::models::GameProfile;
use std::collections::HashMap;
use std::fs;

pub use signal::epic_manifests_signal_mtime;

use manifest::parse_epic_manifest;
use paths::epic_manifest_dirs;

pub fn scan_epic_games() -> Vec<GameProfile> {
    let mut games: HashMap<String, GameProfile> = HashMap::new();
    let manifest_dirs = epic_manifest_dirs();

    for dir in manifest_dirs {
        if !dir.exists() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("item") {
                continue;
            }
            if let Some(game) = parse_epic_manifest(&path) {
                games
                    .entry(game.id.clone())
                    .and_modify(|existing| merge_game_profile(existing, &game))
                    .or_insert(game);
            }
        }
    }

    games.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::manifest::parse_epic_manifest;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    const MAX_EPIC_MANIFEST_BYTES: u64 = 512 * 1024;

    fn write_manifest(json: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{json}").unwrap();
        file
    }

    #[test]
    fn skips_manifest_with_invalid_app_name() {
        let install = tempfile::tempdir().unwrap();
        let loc = install.path().to_string_lossy().replace('\\', "\\\\");
        let file = write_manifest(&format!(
            r#"{{"InstallLocation":"{loc}","DisplayName":"Test","AppName":"bad name"}}"#
        ));
        assert!(parse_epic_manifest(file.path()).is_none());
    }

    #[test]
    fn skips_oversized_manifest_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("huge.item");
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(&[b'{'; MAX_EPIC_MANIFEST_BYTES as usize + 1])
            .unwrap();
        assert!(parse_epic_manifest(&path).is_none());
    }
}
