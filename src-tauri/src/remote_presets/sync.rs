use super::config::{cache_root, load_config, save_config, PresetServerConfig};
use super::manifest::{PackManifest, PresetCatalog};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{copy, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

const HTTP_TIMEOUT_SECS: u64 = 15;

#[derive(serde::Serialize)]
pub struct SyncReport {
    pub ok: bool,
    pub message: String,
    pub packs_synced: usize,
    pub catalog_version: Option<String>,
}

pub fn sync_now(force: bool) -> Result<SyncReport, String> {
    let base_url = super::config::effective_base_url()
        .ok_or("URL сервера пресетов не задан".to_string())?;

    let catalog_url = join_url(&base_url, "catalog.json");
    let catalog_raw = fetch_text(&catalog_url)?;
    let catalog: PresetCatalog = serde_json::from_str(&catalog_raw)
        .map_err(|e| format!("Некорректный catalog.json: {e}"))?;

    if catalog.schema_version != 1 {
        return Err(format!(
            "Неподдерживаемая schema_version каталога: {}",
            catalog.schema_version
        ));
    }

    let root = cache_root()?;
    fs::create_dir_all(&root).map_err(|e| format!("Не удалось создать кэш: {e}"))?;
    fs::write(root.join("catalog.json"), &catalog_raw)
        .map_err(|e| format!("Не удалось сохранить catalog.json: {e}"))?;

    let mut packs_synced = 0usize;
    let active_ids: Vec<&str> = catalog.packs.iter().map(|p| p.id.as_str()).collect();
    for pack_ref in &catalog.packs {
        if sync_pack(&base_url, pack_ref.id.as_str(), &pack_ref.manifest_url, force)? {
            packs_synced += 1;
        }
    }
    prune_orphan_packs(&active_ids)?;

    let mut cfg = load_config()?;
    cfg.last_sync_at = Some(chrono::Utc::now().to_rfc3339());
    cfg.last_sync_ok = true;
    cfg.last_sync_error = None;
    cfg.catalog_version = Some(catalog.version.clone());
    save_config(&cfg)?;

    Ok(SyncReport {
        ok: true,
        message: format!("Синхронизировано паков: {packs_synced}"),
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
    let pack_cache = cache_root()?.join("packs").join(pack_id);
    let manifest_path = pack_cache.join("manifest.json");

    let manifest_full_url = resolve_url(base_url, manifest_url);
    let manifest_raw = fetch_text(&manifest_full_url)?;
    let manifest: PackManifest = serde_json::from_str(&manifest_raw)
        .map_err(|e| format!("Некорректный manifest пака '{pack_id}': {e}"))?;

    if manifest.schema_version != 1 {
        return Err(format!(
            "Неподдерживаемая schema_version пака '{pack_id}': {}",
            manifest.schema_version
        ));
    }

    let bundle = manifest
        .bundle
        .as_ref()
        .ok_or_else(|| format!("Пак '{pack_id}' не содержит bundle"))?;

    let manifest_base = manifest_full_url
        .rsplit_once('/')
        .map(|(base, _)| base)
        .unwrap_or(base_url);
    let bundle_url = resolve_url(manifest_base, &bundle.file);

    let stamp_path = pack_cache.join(".bundle.sha256");
    let expected_sha = bundle.sha256.as_deref();
    let cached_sha = fs::read_to_string(&stamp_path).unwrap_or_default();

    let needs_download = force
        || !pack_cache.join("extracted").is_dir()
        || expected_sha.is_some_and(|sha| sha != cached_sha.trim());

    fs::create_dir_all(&pack_cache)
        .map_err(|e| format!("Не удалось создать кэш пака: {e}"))?;
    fs::write(&manifest_path, &manifest_raw)
        .map_err(|e| format!("Не удалось сохранить manifest: {e}"))?;

    if needs_download {
        let zip_bytes = fetch_bytes(&bundle_url)?;
        if let Some(expected) = expected_sha {
            let actual = hex_sha256(&zip_bytes);
            if actual != expected {
                return Err(format!(
                    "SHA256 пака '{pack_id}' не совпадает: ожидалось {expected}, получено {actual}"
                ));
            }
        }

        let zip_path = pack_cache.join("pack.zip");
        fs::write(&zip_path, &zip_bytes)
            .map_err(|e| format!("Не удалось сохранить pack.zip: {e}"))?;

        let extract_dir = pack_cache.join("extracted");
        if extract_dir.exists() {
            fs::remove_dir_all(&extract_dir)
                .map_err(|e| format!("Не удалось очистить extracted: {e}"))?;
        }
        extract_zip(&zip_path, &extract_dir)?;

        if let Some(expected) = expected_sha {
            fs::write(&stamp_path, expected)
                .map_err(|e| format!("Не удалось сохранить stamp: {e}"))?;
        }
    }

    Ok(needs_download)
}

fn extract_zip(zip_path: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| format!("Не удалось создать extracted: {e}"))?;
    let file = File::open(zip_path).map_err(|e| format!("Не удалось открыть pack.zip: {e}"))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Некорректный pack.zip: {e}"))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Ошибка чтения zip entry: {e}"))?;
        let out_path = dest.join(
            entry
                .enclosed_name()
                .ok_or_else(|| "Небезопасный путь в zip".to_string())?,
        );
        if entry.is_dir() {
            fs::create_dir_all(&out_path)
                .map_err(|e| format!("Не удалось создать каталог в zip: {e}"))?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Не удалось создать родительский каталог: {e}"))?;
            }
            let mut out =
                File::create(&out_path).map_err(|e| format!("Не удалось создать файл: {e}"))?;
            copy(&mut entry, &mut out).map_err(|e| format!("Ошибка распаковки: {e}"))?;
        }
    }
    Ok(())
}

fn fetch_text(url: &str) -> Result<String, String> {
    let bytes = fetch_bytes(url)?;
    String::from_utf8(bytes).map_err(|e| format!("Ответ не UTF-8: {e}"))
}

fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .timeout_connect(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build();

    let response = agent
        .get(url)
        .call()
        .map_err(|e| format!("HTTP GET {url}: {e}"))?;

    let mut reader = response.into_reader();
    let mut buf = Vec::new();
    reader
        .read_to_end(&mut buf)
        .map_err(|e| format!("Чтение ответа {url}: {e}"))?;
    Ok(buf)
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

fn resolve_url(base: &str, relative: &str) -> String {
    if relative.starts_with("http://") || relative.starts_with("https://") {
        return relative.to_string();
    }
    join_url(base, relative)
}

pub fn load_cached_catalog() -> Option<PresetCatalog> {
    let path = cache_root().ok()?.join("catalog.json");
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn load_cached_pack(pack_id: &str) -> Option<(PackManifest, PathBuf)> {
    let pack_dir = cache_root().ok()?.join("packs").join(pack_id);
    let manifest_path = pack_dir.join("manifest.json");
    let raw = fs::read_to_string(manifest_path).ok()?;
    let manifest: PackManifest = serde_json::from_str(&raw).ok()?;
    let root = pack_dir.join("extracted");
    if !root.is_dir() {
        return None;
    }
    Some((manifest, root))
}

fn prune_orphan_packs(active_ids: &[&str]) -> Result<(), String> {
    let packs_dir = cache_root()?.join("packs");
    if !packs_dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(&packs_dir).map_err(|e| format!("Не удалось прочитать кэш паков: {e}"))? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if active_ids.iter().any(|id| *id == name) {
            continue;
        }
        fs::remove_dir_all(entry.path())
            .map_err(|e| format!("Не удалось удалить устаревший пак '{name}': {e}"))?;
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
}
