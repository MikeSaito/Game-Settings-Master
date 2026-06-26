#[derive(Debug, Clone)]
pub struct ConfigIndexEntry {
    pub folder_name: String,
    pub config_dir: std::path::PathBuf,
}
