use super::compile_time_src_root;

#[test]
fn compile_time_catalog_dir_exists_in_workspace() {
    let dir = compile_time_src_root().join("catalog");
    assert!(dir.is_dir(), "dev catalog dir: {}", dir.display());
}
