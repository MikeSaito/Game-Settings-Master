use std::path::Path;

#[cfg(windows)]
const IMPLICIT_LAYERS_KEY: &str = r"Software\Khronos\Vulkan\ImplicitLayers";

/// Register ReShade Vulkan implicit layer (matches official setup tool, HKLM).
#[cfg(windows)]
pub fn register_vulkan_layer(manifest_path: &Path) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let manifest = manifest_path
        .canonicalize()
        .map_err(|e| {
            crate::i18n::t(
                &format!("Некорректный путь Vulkan manifest: {e}"),
                &format!("Invalid Vulkan manifest path: {e}"),
            )
        })?;
    if !manifest.is_file() {
        return Err(crate::i18n::t(
            &format!("Vulkan manifest не найден: {}", manifest_path.display()),
            &format!("Vulkan manifest not found: {}", manifest_path.display()),
        ));
    }

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (key, _) = hklm
        .create_subkey(IMPLICIT_LAYERS_KEY)
        .map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось открыть реестр Vulkan layers: {e}"),
                &format!("Failed to open Vulkan layers registry: {e}"),
            )
        })?;
    key.set_value(manifest.to_string_lossy().as_ref(), &0u32)
        .map_err(|e| {
            crate::i18n::t(
                &format!(
                    "Не удалось зарегистрировать Vulkan layer (нужны права администратора?): {e}"
                ),
                &format!(
                    "Failed to register Vulkan layer (administrator rights required?): {e}"
                ),
            )
        })?;
    Ok(())
}

#[cfg(windows)]
pub fn unregister_vulkan_layer(manifest_path: &Path) -> Result<(), String> {
    use std::io::ErrorKind;
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey_with_flags(IMPLICIT_LAYERS_KEY, KEY_SET_VALUE)
        .map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось открыть реестр Vulkan layers: {e}"),
                &format!("Failed to open Vulkan layers registry: {e}"),
            )
        })?;

    let raw = manifest_path.to_string_lossy().to_string();
    let mut candidates: Vec<String> = Vec::new();
    if let Ok(canonical) = manifest_path.canonicalize() {
        candidates.push(canonical.to_string_lossy().to_string());
    }
    if !candidates.iter().any(|v| v.eq_ignore_ascii_case(&raw)) {
        candidates.push(raw);
    }

    for candidate in candidates {
        match key.delete_value(candidate.as_str()) {
            Ok(_) => return Ok(()),
            Err(e) if e.kind() == ErrorKind::NotFound => continue,
            Err(e) => {
                return Err(crate::i18n::t(
                    &format!("Не удалось удалить запись Vulkan layer из реестра: {e}"),
                    &format!("Failed to remove Vulkan layer registry entry: {e}"),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn register_vulkan_layer(_manifest_path: &Path) -> Result<(), String> {
    Err(crate::i18n::t(
        "Vulkan layer registration поддерживается только на Windows.",
        "Vulkan layer registration is only supported on Windows.",
    ))
}

#[cfg(not(windows))]
pub fn unregister_vulkan_layer(_manifest_path: &Path) -> Result<(), String> {
    Ok(())
}
