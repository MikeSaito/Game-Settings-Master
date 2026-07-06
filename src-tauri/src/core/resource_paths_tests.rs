use super::{catalog_dir, compile_time_src_root};

#[test]
fn compile_time_catalog_dir_exists_in_workspace() {
    let dir = compile_time_src_root().join("catalog");
    assert!(dir.is_dir(), "dev catalog dir: {}", dir.display());
}

#[test]
fn debug_catalog_dir_points_at_source_tree() {
    let dir = catalog_dir();
    let subnautica = dir.join("subnautica2.json");
    assert!(
        subnautica.is_file(),
        "expected source catalog at {}",
        subnautica.display()
    );
    #[cfg(debug_assertions)]
    assert_eq!(
        dir,
        compile_time_src_root().join("catalog"),
        "debug builds must read catalog from src-tauri/catalog"
    );
}
