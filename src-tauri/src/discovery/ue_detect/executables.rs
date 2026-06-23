use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn find_executables(install_dir: &Path) -> Vec<PathBuf> {
    let mut exes = Vec::new();
    for entry in WalkDir::new(install_dir)
        .max_depth(6)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("exe") {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if !name.contains("uninstall")
                    && !name.contains("setup")
                    && !name.contains("redist")
                    && !name.contains("crash")
                    && !name.contains("launcher")
                    && !name.contains("eac")
                    && !name.contains("battleye")
                {
                    exes.push(entry.path().to_path_buf());
                }
            }
        }
    }
    exes.sort_by_key(|p| {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.contains("Shipping") {
            0
        } else {
            p.components().count()
        }
    });
    exes
}
