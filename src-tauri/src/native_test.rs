//! Test-only controls are compiled out of normal builds.
pub(crate) fn checkpoint(_name: &str) {
    #[cfg(feature = "native-test")]
    if std::env::var("GSM_TEST_CRASH_AT").ok().as_deref() == Some(_name) {
        std::process::exit(91);
    }
}

#[cfg(feature = "native-test")]
pub(crate) fn root() -> Result<std::path::PathBuf, String> {
    let root = std::env::var_os("GSM_NATIVE_TEST_ROOT")
        .ok_or("GSM_NATIVE_TEST_ROOT is required for native-test builds")?;
    let root = std::path::PathBuf::from(root);
    if !root.is_absolute() {
        return Err("Test root must be absolute".into());
    }
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    root.canonicalize().map_err(|e| e.to_string())
}

#[cfg(feature = "native-test")]
pub(crate) fn setup() -> Result<(), String> {
    let root = root()?;
    if root.join("initialized").exists() {
        return Ok(());
    }
    let config = root.join("fixture/Saved/Config/Windows");
    let install = root.join("fixture/install");
    std::fs::create_dir_all(&config).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&install).map_err(|e| e.to_string())?;
    let gus = config.join("GameUserSettings.ini");
    if !gus.exists() {
        std::fs::write(gus, "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\n")
            .map_err(|e| e.to_string())?;
    }
    let profile = serde_json::from_value(serde_json::json!({ "id": "manual-native-test", "name": "Native Test Game", "source": "manual", "install_dir": install, "config_dir": config, "exe_name": null, "is_ue": true, "engine_family": "ue5", "engine_version": "5.4" })).map_err(|e| e.to_string())?;
    crate::profiles::save_profile(&profile)?;
    std::fs::write(root.join("initialized"), b"1").map_err(|e| e.to_string())
}

#[cfg(feature = "native-test")]
pub(crate) fn record_launch(version: &str) -> Result<(), String> {
    use std::io::Write;
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(root()?.join("launches.txt"))
        .map_err(|e| e.to_string())?;
    writeln!(log, "{version}").map_err(|e| e.to_string())?;
    log.sync_all().map_err(|e| e.to_string())
}
