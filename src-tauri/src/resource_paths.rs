use std::path::PathBuf;
use std::sync::OnceLock;

static RESOURCE_ROOT: OnceLock<PathBuf> = OnceLock::new();

/// Called from Tauri setup with `app.path().resource_dir()`.
pub fn init_resource_root(root: PathBuf) {
    let _ = RESOURCE_ROOT.set(root);
}

fn compile_time_src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn exe_resources_root() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    // On Windows Tauri places resources next to the exe (not in a resources/ subfolder).
    #[cfg(windows)]
    if dir.join("presets").is_dir() {
        return Some(dir.to_path_buf());
    }
    let resources = dir.join("resources");
    if resources.is_dir() {
        return Some(resources);
    }
    #[cfg(windows)]
    {
        return None;
    }
    #[cfg(not(windows))]
    None
}

/// Tauri bundle root (`resources/` next to the exe in release builds).
pub fn resource_root() -> PathBuf {
    if let Some(root) = RESOURCE_ROOT.get() {
        return root.clone();
    }
    if let Some(exe_root) = exe_resources_root() {
        return exe_root;
    }
    compile_time_src_root()
}

fn resolve_subdir(name: &str) -> PathBuf {
    let bundled = resource_root().join(name);
    if bundled.is_dir() {
        return bundled;
    }
    compile_time_src_root().join(name)
}

pub fn presets_dir() -> PathBuf {
    resolve_subdir("presets")
}

pub fn games_dir() -> PathBuf {
    resolve_subdir("games")
}

pub fn catalog_dir() -> PathBuf {
    resolve_subdir("catalog")
}

/// Shipped author preset catalog (`vps/public` in repo; synced from GitHub by default).
pub fn bundled_remote_presets_dir() -> PathBuf {
    let bundled = resource_root().join("bundled-remote-presets");
    if bundled.join("catalog.json").is_file() {
        return bundled;
    }
    let dev = compile_time_src_root()
        .join("..")
        .join("vps")
        .join("public");
    if dev.join("catalog.json").is_file() {
        return dev;
    }
    bundled
}

pub fn reshade_bundle_dir() -> PathBuf {
    presets_dir().join("reshade")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_time_presets_dir_exists_in_workspace() {
        let dir = compile_time_src_root().join("presets");
        assert!(dir.is_dir(), "dev presets dir: {}", dir.display());
    }

    #[test]
    fn reshade_bundle_has_bin_dir_in_workspace() {
        let bin = compile_time_src_root()
            .join("presets")
            .join("reshade")
            .join("bin");
        assert!(bin.is_dir(), "reshade bin: {}", bin.display());
    }
}
