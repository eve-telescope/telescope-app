use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::Utc;
use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::AsyncWriteExt;
use zip::ZipArchive;

use crate::models::{DscanEntry, DscanParseResult, ScanTypeIndexEntry, SdeStatus};

const SDE_URL: &str =
    "https://developers.eveonline.com/static-data/eve-online-static-data-latest-jsonl.zip";
const INDEX_FILE: &str = "sde_type_index.json";
const INCLUDED_CATEGORY_IDS: [i64; 6] = [2, 3, 6, 18, 22, 65];
const SHIP_CATEGORY_ID: i64 = 6;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SdeIndexCache {
    build_number: i64,
    generated_at: String,
    entries: Vec<ScanTypeIndexEntry>,
}

// ---------------------------------------------------------------------------
// In-memory index (pure, no I/O)
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct SdeIndex {
    by_type_id: HashMap<i64, ScanTypeIndexEntry>,
    name_to_type_id: HashMap<String, i64>,
}

impl SdeIndex {
    pub fn from_entries(entries: Vec<ScanTypeIndexEntry>) -> Self {
        let mut by_type_id = HashMap::with_capacity(entries.len());
        let mut name_to_type_id = HashMap::with_capacity(entries.len());

        for entry in entries {
            name_to_type_id.insert(normalize_name(&entry.type_name), entry.type_id);
            by_type_id.insert(entry.type_id, entry);
        }

        SdeIndex {
            by_type_id,
            name_to_type_id,
        }
    }

    /// Classify a scan row by type ID, falling back to a name lookup.
    fn classify(&self, type_id: Option<i64>, type_name: &str) -> Option<&ScanTypeIndexEntry> {
        type_id.and_then(|id| self.by_type_id.get(&id)).or_else(|| {
            self.name_to_type_id
                .get(&normalize_name(type_name))
                .and_then(|id| self.by_type_id.get(id))
        })
    }
}

/// Parse raw d-scan text against an in-memory index. Pure function.
pub fn parse_dscan_text(index: &SdeIndex, text: &str) -> DscanParseResult {
    let mut entries = Vec::new();
    let mut ship_count = 0;

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        let columns: Vec<&str> = line.split('\t').map(str::trim).collect();
        if columns.len() < 3 {
            continue;
        }

        let type_id = columns.first().and_then(|value| value.parse::<i64>().ok());
        let name = columns.get(1).unwrap_or(&"").to_string();
        let type_name = columns.get(2).unwrap_or(&"").to_string();
        let distance = columns
            .get(3)
            .map(|value| value.to_string())
            .filter(|value| !value.is_empty() && value != "-");

        let classification = index.classify(type_id, &type_name);

        let is_ship = classification
            .map(|entry| entry.category_id == SHIP_CATEGORY_ID)
            .unwrap_or(false);

        if is_ship {
            ship_count += 1;
        }

        entries.push(DscanEntry {
            type_id,
            name,
            type_name,
            distance,
            group_name: classification.map(|entry| entry.group_name.clone()),
            category_name: classification.map(|entry| entry.category_name.clone()),
            is_ship,
        });
    }

    DscanParseResult {
        total_rows: entries.len(),
        ship_count,
        entries,
    }
}

// ---------------------------------------------------------------------------
// Index service (managed state): parse the on-disk cache once, keep it hot
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct SdeService {
    index: tokio::sync::RwLock<Option<Arc<SdeIndex>>>,
    /// Serializes SDE update checks and builds: without it, two concurrent
    /// `ensure_sde_index` calls would both download the archive and write
    /// the same `sde-{build}.zip.download` temp file. Never held across
    /// `parse_dscan` reads — those go through the `index` RwLock as usual.
    update_lock: tokio::sync::Mutex<()>,
}

impl SdeService {
    /// Return the in-memory index, loading it from disk on first use.
    /// Returns Ok(None) when no index has been built yet.
    pub async fn index(&self, app_dir: &Path) -> Result<Option<Arc<SdeIndex>>, String> {
        if let Some(index) = self.index.read().await.clone() {
            return Ok(Some(index));
        }

        let dir = app_dir.to_path_buf();
        let loaded = tokio::task::spawn_blocking(move || -> Result<Option<SdeIndex>, String> {
            Ok(load_index_cache(&dir)?.map(|cache| SdeIndex::from_entries(cache.entries)))
        })
        .await
        .map_err(|err| err.to_string())??;

        let Some(index) = loaded else {
            return Ok(None);
        };

        let index = Arc::new(index);
        *self.index.write().await = Some(index.clone());
        Ok(Some(index))
    }

    /// Drop the cached index so the next access reloads from disk
    /// (call after an SDE update rewrites the cache file).
    pub async fn invalidate(&self) {
        *self.index.write().await = None;
    }
}

// ---------------------------------------------------------------------------
// Status / update
// ---------------------------------------------------------------------------

pub async fn ensure_sde_index(app_dir: &Path, service: &SdeService) -> Result<SdeStatus, String> {
    // Hold the update guard across the needs_update check + build so
    // concurrent calls can't both download/build the same archive.
    let _update_guard = service.update_lock.lock().await;

    let mut current = load_index_cache_async(app_dir).await?;
    let latest_build_number = fetch_latest_build_number().await.ok();

    if let Some(latest) = latest_build_number {
        let needs_update = current
            .as_ref()
            .map(|cache| cache.build_number != latest)
            .unwrap_or(true);

        if needs_update {
            build_index_from_remote(app_dir, latest).await?;
            service.invalidate().await;
            // Reload only after the build actually rewrote the cache file.
            current = load_index_cache_async(app_dir).await?;
        }
    } else if current.is_none() {
        return Err("Unable to determine latest SDE build and no cached index exists".into());
    }

    Ok(make_status(current.as_ref(), latest_build_number))
}

pub async fn get_sde_status(app_dir: &Path) -> Result<SdeStatus, String> {
    let current = load_index_cache_async(app_dir).await?;
    let latest_build_number = fetch_latest_build_number().await.ok();

    Ok(make_status(current.as_ref(), latest_build_number))
}

fn make_status(cache: Option<&SdeIndexCache>, latest_build_number: Option<i64>) -> SdeStatus {
    SdeStatus {
        build_number: cache.map(|cache| cache.build_number),
        latest_build_number,
        ready: cache.is_some(),
        updating: false,
        last_error: None,
    }
}

fn index_path(app_dir: &Path) -> PathBuf {
    app_dir.join(INDEX_FILE)
}

fn normalize_name(name: &str) -> String {
    name.trim().to_lowercase()
}

async fn fetch_latest_build_number() -> Result<i64, String> {
    let client = crate::api::create_client()?;
    let response = client
        .head(SDE_URL)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if let Some(build) = response
        .headers()
        .get("x-sde-build-number")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<i64>().ok())
    {
        return Ok(build);
    }

    extract_build_from_url(response.url().as_str())
        .ok_or_else(|| "Unable to determine latest SDE build number".to_string())
}

async fn build_index_from_remote(app_dir: &Path, expected_build: i64) -> Result<(), String> {
    let temp_path = app_dir.join(format!("sde-{}.zip.download", expected_build));
    let final_path = app_dir.join(format!("sde-{}.zip", expected_build));

    if let Err(err) = download_sde_zip(&temp_path).await {
        // Don't leave partial downloads behind in app-data.
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(err);
    }

    if let Err(err) = tokio::fs::rename(&temp_path, &final_path).await {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(err.to_string());
    }

    // Decoding a multi-hundred-MB zip and parsing tens of thousands of JSONL
    // lines is blocking CPU/IO work — keep it off the async runtime.
    let dir = app_dir.to_path_buf();
    let zip_path = final_path.clone();
    let build_result = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let cache = build_index_cache_from_zip(&zip_path, expected_build)?;
        save_index_cache(&dir, &cache)
    })
    .await
    .map_err(|err| err.to_string())?;

    if let Err(err) = tokio::fs::remove_file(&final_path).await {
        warn!("Failed to remove SDE archive {:?}: {}", final_path, err);
    }

    build_result
}

async fn download_sde_zip(temp_path: &Path) -> Result<(), String> {
    let client = crate::api::create_client()?;
    let mut response = client
        .get(SDE_URL)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    let mut file = tokio::fs::File::create(temp_path)
        .await
        .map_err(|err| err.to_string())?;

    while let Some(chunk) = response.chunk().await.map_err(|err| err.to_string())? {
        file.write_all(&chunk)
            .await
            .map_err(|err| err.to_string())?;
    }
    file.flush().await.map_err(|err| err.to_string())
}

fn build_index_cache_from_zip(
    zip_path: &Path,
    fallback_build: i64,
) -> Result<SdeIndexCache, String> {
    let file = File::open(zip_path).map_err(|err| err.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|err| err.to_string())?;

    let build_number = read_sde_build_number(&mut archive)?.unwrap_or(fallback_build);
    let categories = read_categories(&mut archive)?;
    let groups = read_groups(&mut archive)?;
    let entries = read_type_entries(&mut archive, &categories, &groups)?;

    Ok(SdeIndexCache {
        build_number,
        generated_at: Utc::now().to_rfc3339(),
        entries,
    })
}

fn read_sde_build_number(archive: &mut ZipArchive<File>) -> Result<Option<i64>, String> {
    let mut file = archive
        .by_name("_sde.jsonl")
        .map_err(|err| err.to_string())?;
    let reader = BufReader::new(&mut file);

    for line in reader.lines() {
        let line = line.map_err(|err| err.to_string())?;
        if line.trim().is_empty() {
            continue;
        }

        let value: Value = serde_json::from_str(&line).map_err(|err| err.to_string())?;
        if let Some(build) = value.get("buildNumber").and_then(Value::as_i64) {
            return Ok(Some(build));
        }
    }

    Ok(None)
}

fn read_categories(archive: &mut ZipArchive<File>) -> Result<HashMap<i64, String>, String> {
    let mut file = archive
        .by_name("categories.jsonl")
        .map_err(|err| err.to_string())?;
    let reader = BufReader::new(&mut file);
    let included: HashSet<i64> = INCLUDED_CATEGORY_IDS.into_iter().collect();
    let mut categories = HashMap::new();

    for line in reader.lines() {
        let line = line.map_err(|err| err.to_string())?;
        if line.trim().is_empty() {
            continue;
        }

        let value: Value = serde_json::from_str(&line).map_err(|err| err.to_string())?;
        let key = value
            .get("_key")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        if !included.contains(&key) {
            continue;
        }

        if let Some(name) = value
            .get("name")
            .and_then(|name| name.get("en"))
            .and_then(Value::as_str)
        {
            categories.insert(key, name.to_string());
        }
    }

    Ok(categories)
}

fn read_groups(archive: &mut ZipArchive<File>) -> Result<HashMap<i64, (i64, String)>, String> {
    let mut file = archive
        .by_name("groups.jsonl")
        .map_err(|err| err.to_string())?;
    let reader = BufReader::new(&mut file);
    let included: HashSet<i64> = INCLUDED_CATEGORY_IDS.into_iter().collect();
    let mut groups = HashMap::new();

    for line in reader.lines() {
        let line = line.map_err(|err| err.to_string())?;
        if line.trim().is_empty() {
            continue;
        }

        let value: Value = serde_json::from_str(&line).map_err(|err| err.to_string())?;
        let key = value
            .get("_key")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        let category_id = value
            .get("categoryID")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        if !included.contains(&category_id) {
            continue;
        }

        if let Some(name) = value
            .get("name")
            .and_then(|name| name.get("en"))
            .and_then(Value::as_str)
        {
            groups.insert(key, (category_id, name.to_string()));
        }
    }

    Ok(groups)
}

fn read_type_entries(
    archive: &mut ZipArchive<File>,
    categories: &HashMap<i64, String>,
    groups: &HashMap<i64, (i64, String)>,
) -> Result<Vec<ScanTypeIndexEntry>, String> {
    let mut file = archive
        .by_name("types.jsonl")
        .map_err(|err| err.to_string())?;
    let reader = BufReader::new(&mut file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|err| err.to_string())?;
        if line.trim().is_empty() {
            continue;
        }

        let value: Value = serde_json::from_str(&line).map_err(|err| err.to_string())?;
        let type_id = value
            .get("_key")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        let group_id = value
            .get("groupID")
            .and_then(Value::as_i64)
            .unwrap_or_default();

        let Some((category_id, group_name)) = groups.get(&group_id) else {
            continue;
        };
        let Some(category_name) = categories.get(category_id) else {
            continue;
        };
        let Some(type_name) = value
            .get("name")
            .and_then(|name| name.get("en"))
            .and_then(Value::as_str)
        else {
            continue;
        };

        entries.push(ScanTypeIndexEntry {
            type_id,
            type_name: type_name.to_string(),
            group_id,
            group_name: group_name.clone(),
            category_id: *category_id,
            category_name: category_name.clone(),
        });
    }

    Ok(entries)
}

fn save_index_cache(app_dir: &Path, cache: &SdeIndexCache) -> Result<(), String> {
    let path = index_path(app_dir);
    let json = serde_json::to_vec(cache).map_err(|err| err.to_string())?;
    std::fs::write(path, json).map_err(|err| err.to_string())
}

fn load_index_cache(app_dir: &Path) -> Result<Option<SdeIndexCache>, String> {
    let path = index_path(app_dir);
    if !path.exists() {
        return Ok(None);
    }

    let json = std::fs::read(path).map_err(|err| err.to_string())?;
    let cache = serde_json::from_slice(&json).map_err(|err| err.to_string())?;
    Ok(Some(cache))
}

async fn load_index_cache_async(app_dir: &Path) -> Result<Option<SdeIndexCache>, String> {
    let dir = app_dir.to_path_buf();
    tokio::task::spawn_blocking(move || load_index_cache(&dir))
        .await
        .map_err(|err| err.to_string())?
}

fn extract_build_from_url(url: &str) -> Option<i64> {
    let marker = "eve-online-static-data-";
    let start = url.find(marker)? + marker.len();
    let tail = &url[start..];
    let digits: String = tail
        .chars()
        .take_while(|char| char.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(
        type_id: i64,
        type_name: &str,
        group_name: &str,
        category_id: i64,
        category_name: &str,
    ) -> ScanTypeIndexEntry {
        ScanTypeIndexEntry {
            type_id,
            type_name: type_name.to_string(),
            group_id: 1,
            group_name: group_name.to_string(),
            category_id,
            category_name: category_name.to_string(),
        }
    }

    fn test_index() -> SdeIndex {
        SdeIndex::from_entries(vec![
            entry(587, "Rifter", "Frigate", 6, "Ship"),
            entry(35832, "Astrahus", "Citadel", 65, "Structure"),
        ])
    }

    #[test]
    fn parse_dscan_classifies_ships_by_type_id() {
        let result = parse_dscan_text(&test_index(), "587\tSome Pilot's Rifter\tRifter\t2,3 km");
        assert_eq!(result.total_rows, 1);
        assert_eq!(result.ship_count, 1);
        let row = &result.entries[0];
        assert_eq!(row.type_id, Some(587));
        assert_eq!(row.group_name.as_deref(), Some("Frigate"));
        assert_eq!(row.category_name.as_deref(), Some("Ship"));
        assert!(row.is_ship);
        assert_eq!(row.distance.as_deref(), Some("2,3 km"));
    }

    #[test]
    fn parse_dscan_falls_back_to_name_lookup() {
        // Unknown type ID, but the type name matches (case-insensitively).
        let result = parse_dscan_text(&test_index(), "999999\tUnknown\trifter\t-");
        assert_eq!(result.ship_count, 1);
        assert!(result.entries[0].is_ship);
    }

    #[test]
    fn parse_dscan_dash_distance_is_none() {
        let result = parse_dscan_text(&test_index(), "587\tShip\tRifter\t-");
        assert_eq!(result.entries[0].distance, None);
    }

    #[test]
    fn parse_dscan_skips_short_and_empty_lines() {
        let text = "587\tShip\tRifter\t1 km\n\nnot\ttabs\n just text\n";
        let result = parse_dscan_text(&test_index(), text);
        // "not\ttabs" has 2 columns, "just text" has 1 — both skipped.
        assert_eq!(result.total_rows, 1);
    }

    #[test]
    fn parse_dscan_structures_are_not_ships() {
        let result = parse_dscan_text(&test_index(), "35832\tFortizar Home\tAstrahus\t10 km");
        assert_eq!(result.total_rows, 1);
        assert_eq!(result.ship_count, 0);
        assert_eq!(
            result.entries[0].category_name.as_deref(),
            Some("Structure")
        );
    }

    #[test]
    fn parse_dscan_unknown_type_has_no_classification() {
        let result = parse_dscan_text(&test_index(), "111\tThing\tMystery Object\t5 km");
        let row = &result.entries[0];
        assert_eq!(row.group_name, None);
        assert_eq!(row.category_name, None);
        assert!(!row.is_ship);
    }

    #[test]
    fn normalize_name_trims_and_lowercases() {
        assert_eq!(normalize_name("  Rifter  "), "rifter");
        assert_eq!(normalize_name("ASTRAHUS"), "astrahus");
    }

    #[test]
    fn extract_build_from_url_parses_trailing_digits() {
        assert_eq!(
            extract_build_from_url("https://host/eve-online-static-data-2825200-jsonl.zip"),
            Some(2825200)
        );
        assert_eq!(extract_build_from_url("https://host/other.zip"), None);
        assert_eq!(
            extract_build_from_url("https://host/eve-online-static-data-jsonl.zip"),
            None
        );
    }
}
