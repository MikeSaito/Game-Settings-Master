fn main() {
    println!("cargo:rerun-if-changed=capabilities");
    tauri_build::build()
}
