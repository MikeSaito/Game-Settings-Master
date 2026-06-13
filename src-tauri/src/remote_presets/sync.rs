use super::config::{cache_root, load_config, save_config, PresetServerConfig};
use super::manifest::{PackManifest, PresetCatalog};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::fs::{self, File};
use std::io::{copy, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

const HTTP_TIMEOUT_SECS: u64 = 15;
const MAX_DOWNLOAD_BYTES: usize = 50 * 1024 * 1024;
const MAX_EXTRACT_BYTES: usize = 200 * 1024 * 1024;

#[derive(serde::Serialize)]
pub struct SyncReport {
    pub ok: bool,
    pub message: String,
    pub packs_synced: usize,
    pub catalog_version: Option<String>,
}

fn preset_server_url_missing() -> String {
    crate::i18n::t(
        "URL сервера пресетов не задан",
        "Preset server URL is not configured",
    )
}

pub fn sync_now(force: bool) -> Result<SyncReport, String> {
    if !force && crate::process_util::is_app_background() {
        let version = load_config().ok().and_then(|c| c.catalog_version);
        return Ok(SyncReport {
            ok: true,
            message: "sync skipped while app in background".to_string(),
            packs_synced: 0,
            catalog_version: version,
        });
    }

    let base_url = super::config::effective_base_url().ok_or_else(preset_server_url_missing)?;
    super::config::validate_preset_server_url(&base_url)?;

    let catalog_url = join_url(&base_url, "catalog.json");
    let catalog_raw = fetch_text(&catalog_url)?;
    let catalog: PresetCatalog = serde_json::from_str(&catalog_raw)
        .map_err(|e| {
            crate::i18n::t(
                &format!("Некорректный catalog.json: {e}"),
                &format!("Invalid catalog.json: {e}"),
            )
        })?;

    if catalog.schema_version != 1 {
        return Err(crate::i18n::t(
            &format!(
                "Неподдерживаемая schema_version каталога: {}",
                catalog.schema_version
            ),
            &format!(
                "Unsupported catalog schema_version: {}",
                catalog.schema_version
            ),
        ));
    }

    let root = cache_root()?;
    fs::create_dir_all(&root).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось создать кэш: {e}"),
            &format!("Failed to create cache: {e}"),
        )
    })?;

    let mut packs_synced = 0usize;
    let active_ids: Vec<&str> = catalog
        .packs
        .iter()
        .filter_map(|p| crate::fs_util::is_safe_pack_id(&p.id).then_some(p.id.as_str()))
        .collect();
    for pack_ref in &catalog.packs {
        if sync_pack(
            &base_url,
            pack_ref.id.as_str(),
            &pack_ref.manifest_url,
            force,
        )? {
            packs_synced += 1;
        }
    }
    write_catalog_atomic(&root, &catalog_raw)?;
    prune_orphan_packs(&active_ids)?;

    let mut cfg = load_config()?;
    cfg.last_sync_at = Some(chrono::Utc::now().to_rfc3339());
    cfg.last_sync_ok = true;
    cfg.last_sync_error = None;
    cfg.catalog_version = Some(catalog.version.clone());
    save_config(&cfg)?;
    super::invalidate_resolved_packs_cache();

    Ok(SyncReport {
        ok: true,
        message: crate::i18n::t(
            &format!("Синхронизировано паков: {packs_synced}"),
            &format!("Synced packs: {packs_synced}"),
        ),
        packs_synced,
        catalog_version: Some(catalog.version),
    })
}

fn sync_pack(
    base_url: &str,
    pack_id: &str,
    manifest_url: &str,
    force: bool,
) -> Result<bool, String> {
    if !crate::fs_util::is_safe_pack_id(pack_id) {
        return Err(crate::i18n::t(
            &format!("Недопустимый идентификатор пака: {pack_id}"),
            &format!("Invalid pack identifier: {pack_id}"),
        ));
    }
    let pack_cache = cache_root()?.join("packs").join(pack_id);
    let manifest_path = pack_cache.join("manifest.json");

    let manifest_full_url = resolve_url(base_url, manifest_url)?;
    let manifest_raw = fetch_text(&manifest_full_url)?;
    let manifest: PackManifest = serde_json::from_str(&manifest_raw)
        .map_err(|e| {
            crate::i18n::t(
                &format!("Некорректный manifest пака '{pack_id}': {e}"),
                &format!("Invalid pack '{pack_id}' manifest: {e}"),
            )
        })?;

    if manifest.schema_version != 1 {
        return Err(crate::i18n::t(
            &format!(
                "Неподдерживаемая schema_version пака '{pack_id}': {}",
                manifest.schema_version
            ),
            &format!(
                "Unsupported pack '{pack_id}' schema_version: {}",
                manifest.schema_version
            ),
        ));
    }

    let bundle = manifest.bundle.as_ref().ok_or_else(|| {
        crate::i18n::t(
            &format!("Пак '{pack_id}' не содержит bundle"),
            &format!("Pack '{pack_id}' has no bundle"),
        )
    })?;

    let manifest_base = manifest_full_url
        .rsplit_once('/')
        .map(|(base, _)| base)
        .unwrap_or(base_url);
    let bundle_url = resolve_url(manifest_base, &bundle.file)?;

    let stamp_path = pack_cache.join(".bundle.sha256");
    let expected_sha = bundle.sha256.as_deref();
    let cached_sha = fs::read_to_string(&stamp_path).unwrap_or_default();

    // Reject packs without bundle.sha256 regardless of cache: otherwise a once-downloaded
    // unsigned pack (with GSM_ALLOW_UNVERIFIED_PACKS=1) would stay active forever.
    if expected_sha.is_none() && !unverified_packs_allowed() {
        return Err(crate::i18n::t(
            &format!(
                "Пак '{pack_id}' не содержит bundle.sha256 — отклонён. \
                 Задайте GSM_ALLOW_UNVERIFIED_PACKS=1 только для dev."
            ),
            &format!(
                "Pack '{pack_id}' has no bundle.sha256 — rejected. \
                 Set GSM_ALLOW_UNVERIFIED_PACKS=1 for dev only."
            ),
        ));
    }

    let needs_download = force
        || !pack_cache.join("extracted").is_dir()
        || expected_sha.is_some_and(|sha| sha != cached_sha.trim());

    fs::create_dir_all(&pack_cache).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось создать кэш пака: {e}"),
            &format!("Failed to create pack cache: {e}"),
        )
    })?;

    if needs_download {
        let zip_bytes = fetch_bytes(&bundle_url)?;
        if let Some(expected) = expected_sha {
            let actual = hex_sha256(&zip_bytes);
            if actual != expected {
                return Err(crate::i18n::t(
                    &format!(
                        "SHA256 пака '{pack_id}' не совпадает: ожидалось {expected}, получено {actual}"
                    ),
                    &format!(
                        "Pack '{pack_id}' SHA256 mismatch: expected {expected}, got {actual}"
                    ),
                ));
            }
        }

        let zip_path = pack_cache.join("pack.zip");
        let zip_tmp = pack_cache.join("pack.zip.tmp");
        crate::fs_util::write_file_bytes_opts(&zip_tmp, &zip_bytes, true)
            .map_err(|e| {
                crate::i18n::t(
                    &format!("Не удалось сохранить pack.zip.tmp: {e}"),
                    &format!("Failed to save pack.zip.tmp: {e}"),
                )
            })?;
        if zip_path.exists() {
            fs::remove_file(&zip_path).map_err(|e| {
                crate::i18n::t(
                    &format!("Не удалось удалить старый pack.zip: {e}"),
                    &format!("Failed to remove old pack.zip: {e}"),
                )
            })?;
        }
        fs::rename(&zip_tmp, &zip_path).map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось активировать pack.zip: {e}"),
                &format!("Failed to activate pack.zip: {e}"),
            )
        })?;

        let extract_dir = pack_cache.join("extracted");
        let extract_new = pack_cache.join("extracted.new");
        if extract_new.exists() {
            fs::remove_dir_all(&extract_new).map_err(|e| {
                crate::i18n::t(
                    &format!("Не удалось очистить extracted.new: {e}"),
                    &format!("Failed to clean extracted.new: {e}"),
                )
            })?;
        }
        extract_zip(&zip_path, &extract_new)?;
        replace_extracted_dir(&extract_dir, &extract_new)?;

        if let Some(expected) = expected_sha {
            fs::write(&stamp_path, expected).map_err(|e| {
                crate::i18n::t(
                    &format!("Не удалось сохранить stamp: {e}"),
                    &format!("Failed to save stamp: {e}"),
                )
            })?;
        }
    }

    crate::fs_util::write_file_bytes_opts(&manifest_path, manifest_raw.as_bytes(), true)
        .map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось сохранить manifest: {e}"),
                &format!("Failed to save manifest: {e}"),
            )
        })?;
    super::invalidate_resolved_packs_cache();

    Ok(needs_download)
}

/// Download a single pack by id (without a full sync_now over the entire catalog).
pub fn sync_pack_by_id(pack_id: &str, force: bool) -> Result<bool, String> {
    if !crate::fs_util::is_safe_pack_id(pack_id) {
        return Err(crate::i18n::t(
            &format!("Недопустимый идентификатор пака: {pack_id}"),
            &format!("Invalid pack identifier: {pack_id}"),
        ));
    }
    if !force && crate::process_util::is_app_background() {
        return Ok(false);
    }

    let base_url = super::config::effective_base_url().ok_or_else(preset_server_url_missing)?;
    super::config::validate_preset_server_url(&base_url)?;

    let catalog = match load_cached_catalog() {
        Some(c) => c,
        None => {
            sync_now(false)?;
            load_cached_catalog().ok_or_else(|| {
                crate::i18n::t(
                    "Каталог пресетов не загружен. Проверьте интернет.",
                    "Preset catalog not loaded. Check your internet connection.",
                )
            })?
        }
    };

    let pack_ref = catalog
        .packs
        .iter()
        .find(|p| p.id == pack_id)
        .ok_or_else(|| {
            crate::i18n::t(
                &format!("Пак '{pack_id}' не найден в catalog.json"),
                &format!("Pack '{pack_id}' not found in catalog.json"),
            )
        })?;

    sync_pack(&base_url, pack_id, &pack_ref.manifest_url, force)
}

fn extract_zip(zip_path: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось создать extracted: {e}"),
            &format!("Failed to create extracted: {e}"),
        )
    })?;
    let file = File::open(zip_path).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось открыть pack.zip: {e}"),
            &format!("Failed to open pack.zip: {e}"),
        )
    })?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| {
        crate::i18n::t(
            &format!("Некорректный pack.zip: {e}"),
            &format!("Invalid pack.zip: {e}"),
        )
    })?;

    let mut extracted_bytes = 0usize;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| {
            crate::i18n::t(
                &format!("Ошибка чтения zip entry: {e}"),
                &format!("Error reading zip entry: {e}"),
            )
        })?;
        let entry_size = entry.size() as usize;
        extracted_bytes = extracted_bytes.saturating_add(entry_size);
        if extracted_bytes > MAX_EXTRACT_BYTES {
            return Err(crate::i18n::t(
                &format!(
                    "Распаковка пака превышает лимит {} MB",
                    MAX_EXTRACT_BYTES / (1024 * 1024)
                ),
                &format!(
                    "Pack extraction exceeds limit of {} MB",
                    MAX_EXTRACT_BYTES / (1024 * 1024)
                ),
            ));
        }
        let out_path = dest.join(
            entry
                .enclosed_name()
                .ok_or_else(|| {
                    crate::i18n::t(
                        "Небезопасный путь в zip",
                        "Unsafe path in zip archive",
                    )
                })?,
        );
        if entry.is_dir() {
            fs::create_dir_all(&out_path).map_err(|e| {
                crate::i18n::t(
                    &format!("Не удалось создать каталог в zip: {e}"),
                    &format!("Failed to create directory in zip: {e}"),
                )
            })?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    crate::i18n::t(
                        &format!("Не удалось создать родительский каталог: {e}"),
                        &format!("Failed to create parent directory: {e}"),
                    )
                })?;
            }
            let mut out = File::create(&out_path).map_err(|e| {
                crate::i18n::t(
                    &format!("Не удалось создать файл: {e}"),
                    &format!("Failed to create file: {e}"),
                )
            })?;
            copy(&mut entry, &mut out).map_err(|e| {
                crate::i18n::t(
                    &format!("Ошибка распаковки: {e}"),
                    &format!("Extraction error: {e}"),
                )
            })?;
        }
    }
    Ok(())
}

fn write_catalog_atomic(root: &Path, content: &str) -> Result<(), String> {
    let path = root.join("catalog.json");
    crate::fs_util::write_file_bytes_opts(&path, content.as_bytes(), true)
}

fn replace_extracted_dir(extract_dir: &Path, extract_new: &Path) -> Result<(), String> {
    let extract_old = extract_dir.with_extension("old");
    if extract_old.exists() {
        fs::remove_dir_all(&extract_old).map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось очистить extracted.old: {e}"),
                &format!("Failed to clean extracted.old: {e}"),
            )
        })?;
    }

    if extract_dir.exists() {
        fs::rename(extract_dir, &extract_old).map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось переименовать extracted в extracted.old: {e}"),
                &format!("Failed to rename extracted to extracted.old: {e}"),
            )
        })?;
    }
    match fs::rename(extract_new, extract_dir) {
        Ok(_) => {
            if extract_old.exists() {
                let _ = fs::remove_dir_all(&extract_old);
            }
            Ok(())
        }
        Err(e) => {
            if extract_old.exists() {
                let _ = fs::rename(&extract_old, extract_dir);
            }
            Err(crate::i18n::t(
                &format!("Не удалось активировать extracted.new: {e}"),
                &format!("Failed to activate extracted.new: {e}"),
            ))
        }
    }
}

fn fetch_text(url: &str) -> Result<String, String> {
    let bytes = fetch_bytes(url)?;
    String::from_utf8(bytes).map_err(|e| {
        crate::i18n::t(
            &format!("Ответ не UTF-8: {e}"),
            &format!("Response is not UTF-8: {e}"),
        )
    })
}

fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    validate_fetch_url(url)?;

    let agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .timeout_connect(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build();

    let response = agent
        .get(url)
        .call()
        .map_err(|e| {
            crate::i18n::t(
                &format!("HTTP GET {url}: {e}"),
                &format!("HTTP GET {url}: {e}"),
            )
        })?;

    // Redirect-attack protection: the final URL after redirects must pass validation again
    // (https, or http only for localhost). Otherwise an allowed https URL could redirect
    // to http:// (downgrade) or to LAN/localhost (SSRF).
    let final_url = response.get_url().to_string();
    if final_url != url {
        validate_fetch_url(&final_url).map_err(|e| {
            crate::i18n::t(
                &format!("Небезопасный редирект {url} → {final_url}: {e}"),
                &format!("Unsafe redirect {url} → {final_url}: {e}"),
            )
        })?;
    }

    let mut reader = response.into_reader();
    let mut buf = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let n = reader.read(&mut chunk).map_err(|e| {
            crate::i18n::t(
                &format!("Чтение ответа {url}: {e}"),
                &format!("Reading response {url}: {e}"),
            )
        })?;
        if n == 0 {
            break;
        }
        if buf.len() + n > MAX_DOWNLOAD_BYTES {
            return Err(crate::i18n::t(
                &format!("Ответ слишком большой (>{MAX_DOWNLOAD_BYTES} bytes): {url}"),
                &format!("Response too large (>{MAX_DOWNLOAD_BYTES} bytes): {url}"),
            ));
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    Ok(buf)
}

pub(crate) fn validate_fetch_url(url: &str) -> Result<(), String> {
    super::config::validate_preset_server_url(url)
}

fn hex_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn join_url(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    let path = path.trim_start_matches('/');
    format!("{base}/{path}")
}

fn resolve_url(base: &str, relative: &str) -> Result<String, String> {
    if relative.starts_with("http://") || relative.starts_with("https://") {
        return Err(crate::i18n::t(
            "Абсолютные URL в manifest не поддерживаются",
            "Absolute URLs in manifest are not supported",
        ));
    }
    if !crate::fs_util::is_safe_manifest_relative_path(relative) {
        return Err(crate::i18n::t(
            &format!("Недопустимый путь в manifest URL: {relative}"),
            &format!("Invalid path in manifest URL: {relative}"),
        ));
    }
    Ok(join_url(base, relative))
}

pub fn load_cached_catalog() -> Option<PresetCatalog> {
    let _ = seed_bundled_presets_if_needed();
    read_catalog_file(&cache_root().ok()?.join("catalog.json"))
}

fn version_cmp(a: &str, b: &str) -> Ordering {
    let pa: Vec<u32> = a.split('.').filter_map(|p| p.parse().ok()).collect();
    let pb: Vec<u32> = b.split('.').filter_map(|p| p.parse().ok()).collect();
    let len = pa.len().max(pb.len());
    for i in 0..len {
        let va = *pa.get(i).unwrap_or(&0);
        let vb = *pb.get(i).unwrap_or(&0);
        match va.cmp(&vb) {
            Ordering::Equal => {}
            other => return other,
        }
    }
    Ordering::Equal
}

fn catalog_is_newer(candidate: &PresetCatalog, baseline: &PresetCatalog) -> bool {
    if let (Some(c_ts), Some(b_ts)) = (&candidate.updated_at, &baseline.updated_at) {
        if c_ts != b_ts {
            return c_ts > b_ts;
        }
    }
    version_cmp(&candidate.version, &baseline.version) == Ordering::Greater
}

fn read_catalog_file(path: &Path) -> Option<PresetCatalog> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn seed_bundled_presets_if_needed() -> Result<(), String> {
    let bundled = crate::resource_paths::bundled_remote_presets_dir();
    let catalog_path = bundled.join("catalog.json");
    if !catalog_path.is_file() {
        return Ok(());
    }

    let root = cache_root()?;
    fs::create_dir_all(&root).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось создать кэш: {e}"),
            &format!("Failed to create cache: {e}"),
        )
    })?;

    let cache_catalog = root.join("catalog.json");
    let bundled_raw = fs::read_to_string(&catalog_path).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось прочитать bundled catalog.json: {e}"),
            &format!("Failed to read bundled catalog.json: {e}"),
        )
    })?;
    let catalog: PresetCatalog = serde_json::from_str(&bundled_raw).map_err(|e| {
        crate::i18n::t(
            &format!("Некорректный bundled catalog.json: {e}"),
            &format!("Invalid bundled catalog.json: {e}"),
        )
    })?;

    let should_refresh_catalog = match read_catalog_file(&cache_catalog) {
        None => true,
        Some(cached) => catalog_is_newer(&catalog, &cached),
    };
    if should_refresh_catalog {
        write_catalog_atomic(&root, &bundled_raw)?;
        super::invalidate_resolved_packs_cache();
    }

    for pack in &catalog.packs {
        if crate::fs_util::is_safe_pack_id(&pack.id) {
            let _ = seed_bundled_pack(&pack.id);
        }
    }
    Ok(())
}

pub fn seed_bundled_pack(pack_id: &str) -> Result<bool, String> {
    if !crate::fs_util::is_safe_pack_id(pack_id) {
        return Err(crate::i18n::t(
            &format!("Недопустимый идентификатор пака: {pack_id}"),
            &format!("Invalid pack identifier: {pack_id}"),
        ));
    }

    let bundled = crate::resource_paths::bundled_remote_presets_dir();
    let pack_dir = bundled.join("packs").join(pack_id);
    let manifest_path = pack_dir.join("manifest.json");
    let zip_path = pack_dir.join("pack.zip");
    if !manifest_path.is_file() || !zip_path.is_file() {
        return Err(crate::i18n::t(
            &format!("Bundled-пак '{pack_id}' не найден"),
            &format!("Bundled pack '{pack_id}' not found"),
        ));
    }

    let manifest_raw = fs::read_to_string(&manifest_path).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось прочитать bundled manifest '{pack_id}': {e}"),
            &format!("Failed to read bundled manifest '{pack_id}': {e}"),
        )
    })?;
    let manifest: PackManifest = serde_json::from_str(&manifest_raw).map_err(|e| {
        crate::i18n::t(
            &format!("Некорректный bundled manifest '{pack_id}': {e}"),
            &format!("Invalid bundled manifest '{pack_id}': {e}"),
        )
    })?;

    let expected_sha = manifest
        .bundle
        .as_ref()
        .and_then(|b| b.sha256.as_deref());
    let pack_cache = cache_root()?.join("packs").join(pack_id);
    let stamp_path = pack_cache.join(".bundle.sha256");
    let cached_sha = fs::read_to_string(&stamp_path).unwrap_or_default();
    if try_load_cached_pack(pack_id).is_some()
        && expected_sha.is_some_and(|sha| sha == cached_sha.trim())
    {
        let cached_manifest_path = pack_cache.join("manifest.json");
        if fs::read_to_string(&cached_manifest_path).ok().as_deref() != Some(manifest_raw.as_str())
        {
            fs::create_dir_all(&pack_cache).map_err(|e| {
                crate::i18n::t(
                    &format!("Не удалось создать кэш пака: {e}"),
                    &format!("Failed to create pack cache: {e}"),
                )
            })?;
            crate::fs_util::write_file_bytes_opts(
                &cached_manifest_path,
                manifest_raw.as_bytes(),
                true,
            )
            .map_err(|e| {
                crate::i18n::t(
                    &format!("Не удалось сохранить manifest: {e}"),
                    &format!("Failed to save manifest: {e}"),
                )
            })?;
            super::invalidate_resolved_packs_cache();
            return Ok(true);
        }
        return Ok(false);
    }

    if manifest.schema_version != 1 {
        return Err(crate::i18n::t(
            &format!(
                "Неподдерживаемая schema_version bundled-пака '{pack_id}': {}",
                manifest.schema_version
            ),
            &format!(
                "Unsupported bundled pack '{pack_id}' schema_version: {}",
                manifest.schema_version
            ),
        ));
    }

    if manifest.bundle.is_none() {
        return Err(crate::i18n::t(
            &format!("Bundled-пак '{pack_id}' не содержит bundle"),
            &format!("Bundled pack '{pack_id}' has no bundle"),
        ));
    }

    let zip_bytes = fs::read(&zip_path).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось прочитать bundled pack.zip '{pack_id}': {e}"),
            &format!("Failed to read bundled pack.zip '{pack_id}': {e}"),
        )
    })?;

    if expected_sha.is_none() && !unverified_packs_allowed() {
        return Err(crate::i18n::t(
            &format!("Bundled-пак '{pack_id}' не содержит bundle.sha256"),
            &format!("Bundled pack '{pack_id}' has no bundle.sha256"),
        ));
    }
    if let Some(expected) = expected_sha {
        let actual = hex_sha256(&zip_bytes);
        if actual != expected {
            return Err(crate::i18n::t(
                &format!(
                    "SHA256 bundled-пака '{pack_id}' не совпадает: ожидалось {expected}, получено {actual}"
                ),
                &format!(
                    "Bundled pack '{pack_id}' SHA256 mismatch: expected {expected}, got {actual}"
                ),
            ));
        }
    }

    fs::create_dir_all(&pack_cache).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось создать кэш пака: {e}"),
            &format!("Failed to create pack cache: {e}"),
        )
    })?;

    crate::fs_util::write_file_bytes_opts(&pack_cache.join("manifest.json"), manifest_raw.as_bytes(), true)
        .map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось сохранить manifest: {e}"),
                &format!("Failed to save manifest: {e}"),
            )
        })?;
    crate::fs_util::write_file_bytes_opts(&pack_cache.join("pack.zip"), &zip_bytes, true)
        .map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось сохранить pack.zip: {e}"),
                &format!("Failed to save pack.zip: {e}"),
            )
        })?;

    let extract_dir = pack_cache.join("extracted");
    let extract_new = pack_cache.join("extracted.new");
    if extract_new.exists() {
        fs::remove_dir_all(&extract_new).map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось очистить extracted.new: {e}"),
                &format!("Failed to clean extracted.new: {e}"),
            )
        })?;
    }
    extract_zip(&pack_cache.join("pack.zip"), &extract_new)?;
    replace_extracted_dir(&extract_dir, &extract_new)?;

    if let Some(expected) = expected_sha {
        fs::write(pack_cache.join(".bundle.sha256"), expected).map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось сохранить stamp: {e}"),
                &format!("Failed to save stamp: {e}"),
            )
        })?;
    }

    super::invalidate_resolved_packs_cache();
    Ok(true)
}

pub fn load_cached_pack(pack_id: &str) -> Option<(PackManifest, PathBuf)> {
    try_load_cached_pack(pack_id).or_else(|| {
        seed_bundled_pack(pack_id).ok()?;
        try_load_cached_pack(pack_id)
    })
}

fn try_load_cached_pack(pack_id: &str) -> Option<(PackManifest, PathBuf)> {
    if !crate::fs_util::is_safe_pack_id(pack_id) {
        return None;
    }
    let pack_dir = cache_root().ok()?.join("packs").join(pack_id);
    let manifest_path = pack_dir.join("manifest.json");
    let raw = fs::read_to_string(manifest_path).ok()?;
    let manifest: PackManifest = serde_json::from_str(&raw).ok()?;
    // Do not serve an unsigned pack (without bundle.sha256) from cache without an explicit dev flag,
    // even if it was extracted earlier.
    let signed = manifest
        .bundle
        .as_ref()
        .and_then(|b| b.sha256.as_ref())
        .is_some();
    if !signed && !unverified_packs_allowed() {
        return None;
    }
    let root = pack_dir.join("extracted");
    if !root.is_dir() {
        return None;
    }
    Some((manifest, root))
}

fn unverified_packs_allowed() -> bool {
    std::env::var("GSM_ALLOW_UNVERIFIED_PACKS").ok().as_deref() == Some("1")
}

fn prune_orphan_packs(active_ids: &[&str]) -> Result<(), String> {
    let safe_active: std::collections::HashSet<&str> = active_ids
        .iter()
        .copied()
        .filter(|id| crate::fs_util::is_safe_pack_id(id))
        .collect();
    let packs_dir = cache_root()?.join("packs");
    if !packs_dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(&packs_dir).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось прочитать кэш паков: {e}"),
            &format!("Failed to read pack cache: {e}"),
        )
    })? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if safe_active.contains(name.as_str()) {
            continue;
        }
        fs::remove_dir_all(entry.path()).map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось удалить устаревший пак '{name}': {e}"),
                &format!("Failed to remove stale pack '{name}': {e}"),
            )
        })?;
    }
    Ok(())
}

pub fn mark_sync_error(err: &str) -> Result<PresetServerConfig, String> {
    let mut cfg = load_config()?;
    cfg.last_sync_ok = false;
    cfg.last_sync_error = Some(err.to_string());
    save_config(&cfg)?;
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_url_rejects_absolute() {
        assert!(resolve_url("https://example.com", "https://evil.com/x").is_err());
    }

    #[test]
    fn join_url_works() {
        assert_eq!(
            join_url("http://localhost:8787", "catalog.json"),
            "http://localhost:8787/catalog.json"
        );
        assert_eq!(
            join_url("http://localhost:8787/", "/packs/x/manifest.json"),
            "http://localhost:8787/packs/x/manifest.json"
        );
    }

    #[test]
    fn version_cmp_orders_semver_parts() {
        assert_eq!(version_cmp("1.5.0", "1.4.9"), Ordering::Greater);
        assert_eq!(version_cmp("1.5.0", "1.5.0"), Ordering::Equal);
        assert_eq!(version_cmp("1.4.0", "1.5.0"), Ordering::Less);
    }

    #[test]
    fn catalog_is_newer_prefers_updated_at() {
        use super::super::manifest::{CatalogPackRef, PresetCatalog};
        let older = PresetCatalog {
            schema_version: 1,
            catalog_id: "a".into(),
            version: "2.0.0".into(),
            updated_at: Some("2026-01-01T00:00:00Z".into()),
            base_url: None,
            packs: vec![],
        };
        let newer = PresetCatalog {
            schema_version: 1,
            catalog_id: "a".into(),
            version: "1.0.0".into(),
            updated_at: Some("2026-06-01T00:00:00Z".into()),
            base_url: None,
            packs: vec![CatalogPackRef {
                id: "x".into(),
                manifest_url: "packs/x/manifest.json".into(),
                sha256: None,
            }],
        };
        assert!(catalog_is_newer(&newer, &older));
        assert!(!catalog_is_newer(&older, &newer));
    }
}
