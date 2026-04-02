use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::AsyncWriteExt;
use zip::ZipArchive;

use crate::models::{DscanEntry, DscanParseResult, ScanTypeIndexEntry, SdeStatus};

const SDE_URL: &str =
    "https://developers.eveonline.com/static-data/eve-online-static-data-latest-jsonl.zip";
const INDEX_FILE: &str = "sde_type_index.json";
const INCLUDED_CATEGORY_IDS: [i64; 6] = [2, 3, 6, 18, 22, 65];

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SdeIndexCache {
    build_number: i64,
    generated_at: String,
    entries: Vec<ScanTypeIndexEntry>,
}

#[derive(Debug, Clone)]
pub struct SdeIndex {
    by_type_id: HashMap<i64, ScanTypeIndexEntry>,
    by_name: HashMap<String, ScanTypeIndexEntry>,
}

pub async fn ensure_sde_index(app_dir: &Path) -> Result<SdeStatus, String> {
    let current = load_index_cache(app_dir)?;
    let latest_build_number = fetch_latest_build_number().await.ok();

    if let Some(latest) = latest_build_number {
        let needs_update = current
            .as_ref()
            .map(|cache| cache.build_number != latest)
            .unwrap_or(true);

        if needs_update {
            build_index_from_remote(app_dir, latest).await?;
        }
    } else if current.is_none() {
        return Err("Unable to determine latest SDE build and no cached index exists".into());
    }

    let refreshed = load_index_cache(app_dir)?;
    Ok(SdeStatus {
        build_number: refreshed.as_ref().map(|cache| cache.build_number),
        latest_build_number,
        ready: refreshed.is_some(),
        updating: false,
        last_error: None,
    })
}

pub async fn get_sde_status(app_dir: &Path) -> Result<SdeStatus, String> {
    let current = load_index_cache(app_dir)?;
    let latest_build_number = fetch_latest_build_number().await.ok();

    Ok(SdeStatus {
        build_number: current.as_ref().map(|cache| cache.build_number),
        latest_build_number,
        ready: current.is_some(),
        updating: false,
        last_error: None,
    })
}

pub fn parse_dscan(text: &str, app_dir: &Path) -> Result<DscanParseResult, String> {
    let index = load_index(app_dir)?.ok_or_else(|| "SDE index is not ready yet".to_string())?;

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

        let classification = type_id
            .and_then(|id| index.by_type_id.get(&id).cloned())
            .or_else(|| index.by_name.get(&normalize_name(&type_name)).cloned());

        let is_ship = classification
            .as_ref()
            .map(|entry| entry.category_id == 6)
            .unwrap_or(false);

        if is_ship {
            ship_count += 1;
        }

        entries.push(DscanEntry {
            type_id,
            name,
            type_name,
            distance,
            group_name: classification
                .as_ref()
                .map(|entry| entry.group_name.clone()),
            category_name: classification
                .as_ref()
                .map(|entry| entry.category_name.clone()),
            is_ship,
        });
    }

    Ok(DscanParseResult {
        total_rows: entries.len(),
        ship_count,
        entries,
    })
}

pub fn load_index(app_dir: &Path) -> Result<Option<SdeIndex>, String> {
    let cache = load_index_cache(app_dir)?;
    Ok(cache.map(|cache| {
        let mut by_type_id = HashMap::new();
        let mut by_name = HashMap::new();

        for entry in cache.entries {
            by_name.insert(normalize_name(&entry.type_name), entry.clone());
            by_type_id.insert(entry.type_id, entry);
        }

        SdeIndex {
            by_type_id,
            by_name,
        }
    }))
}

fn index_path(app_dir: &Path) -> PathBuf {
    app_dir.join(INDEX_FILE)
}

fn normalize_name(name: &str) -> String {
    name.trim().to_lowercase()
}

async fn fetch_latest_build_number() -> Result<i64, String> {
    let client = reqwest::Client::new();
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

    let client = reqwest::Client::new();
    let mut response = client
        .get(SDE_URL)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    let mut file = tokio::fs::File::create(&temp_path)
        .await
        .map_err(|err| err.to_string())?;

    while let Some(chunk) = response.chunk().await.map_err(|err| err.to_string())? {
        file.write_all(&chunk)
            .await
            .map_err(|err| err.to_string())?;
    }
    file.flush().await.map_err(|err| err.to_string())?;

    tokio::fs::rename(&temp_path, &final_path)
        .await
        .map_err(|err| err.to_string())?;

    let cache = build_index_cache_from_zip(&final_path, expected_build)?;
    save_index_cache(app_dir, &cache)?;

    let _ = std::fs::remove_file(final_path);
    Ok(())
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
