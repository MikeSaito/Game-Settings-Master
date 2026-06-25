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

pub fn games_dir() -> PathBuf {
    resolve_subdir("games")
}

pub fn catalog_dir() -> PathBuf {
    resolve_subdir("catalog")
}

#[cfg(test)]
#[path = "resource_paths_tests.rs"]
mod tests;
