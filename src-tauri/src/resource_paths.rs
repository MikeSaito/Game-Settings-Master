use std::path::PathBuf;
use std::sync::OnceLock;

static RESOURCE_ROOT: OnceLock<PathBuf> = OnceLock::new();

/// Вызывается из Tauri setup с `app.path().resource_dir()`.
pub fn init_resource_root(root: PathBuf) {
    let _ = RESOURCE_ROOT.set(root);
}

fn compile_time_src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn exe_resources_root() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let resources = dir.join("resources");
    resources.is_dir().then_some(resources)
}

/// Корень бандла Tauri (`resources/` рядом с exe в релизе).
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
