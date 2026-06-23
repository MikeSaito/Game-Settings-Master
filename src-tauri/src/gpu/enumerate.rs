#[cfg(windows)]
pub(crate) fn enumerate_gpu_names() -> Vec<String> {
    if let Some(names) = enumerate_gpu_from_registry() {
        if !names.is_empty() {
            return names;
        }
    }
    vec!["Unknown GPU".to_string()]
}

#[cfg(not(windows))]
pub(crate) fn enumerate_gpu_names() -> Vec<String> {
    vec!["Unknown GPU".to_string()]
}

#[cfg(windows)]
fn enumerate_gpu_from_registry() -> Option<Vec<String>> {
    use winreg::enums::*;
    use winreg::RegKey;

    const SKIP: &[&str] = &[
        "microsoft basic",
        "remote",
        "parsec",
        "virtual",
        "vmware",
        "citrix",
        "meta virtual",
        "spice",
        "qxl",
    ];

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let class_key = hklm
        .open_subkey(
            r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}",
        )
        .ok()?;

    let mut names = Vec::new();
    for i in 0..32 {
        let Ok(sub) = class_key.open_subkey(format!("{i:04}")) else {
            continue;
        };
        let Ok(desc) = sub.get_value::<String, _>("DriverDesc") else {
            continue;
        };
        let lower = desc.to_lowercase();
        if SKIP.iter().any(|needle| lower.contains(needle)) {
            continue;
        }
        names.push(desc);
    }

    Some(names)
}
