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
