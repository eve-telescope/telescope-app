use reqwest::Client;

use crate::models::*;

const USER_AGENT: &str = "Telescope | https://eve-telescope.com";

pub fn build_client(token: &str) -> Result<Client, String> {
    Client::builder()
        .user_agent(USER_AGENT)
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", token).parse().unwrap(),
            );
            headers.insert(reqwest::header::ACCEPT, "application/json".parse().unwrap());
            headers.insert(
                reqwest::header::CONTENT_TYPE,
                "application/json".parse().unwrap(),
            );
            headers
        })
        .build()
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Networks
// ---------------------------------------------------------------------------

pub async fn fetch_networks(client: &Client, base_url: &str) -> Result<Vec<IntelNetwork>, String> {
    let resp = client
        .get(format!("{}/api/networks", base_url))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("API error: {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

pub async fn create_network(
    client: &Client,
    base_url: &str,
    name: &str,
) -> Result<IntelNetwork, String> {
    let resp = client
        .post(format!("{}/api/networks", base_url))
        .json(&serde_json::json!({ "name": name }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("API error: {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

pub async fn delete_network(
    client: &Client,
    base_url: &str,
    network_id: i64,
) -> Result<(), String> {
    let resp = client
        .delete(format!("{}/api/networks/{}", base_url, network_id))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("API error: {}", resp.status()));
    }
    Ok(())
}

pub async fn get_network_detail(
    client: &Client,
    base_url: &str,
    network_id: i64,
) -> Result<NetworkDetail, String> {
    let resp = client
        .get(format!("{}/api/networks/{}", base_url, network_id))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("API error: {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Intel entries
// ---------------------------------------------------------------------------

pub async fn lookup_intel(
    client: &Client,
    base_url: &str,
    entity_ids: &[i64],
) -> Result<Vec<IntelEntry>, String> {
    let params: String = entity_ids
        .iter()
        .map(|id| format!("entity_ids[]={}", id))
        .collect::<Vec<_>>()
        .join("&");
    let resp = client
        .get(format!("{}/api/intel/lookup?{}", base_url, params))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("API error: {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

pub async fn add_intel_entry(
    client: &Client,
    base_url: &str,
    network_id: i64,
    entity_type: &str,
    entity_id: i64,
    entity_name: &str,
    color: &str,
    label: &str,
    notes: Option<&str>,
) -> Result<IntelEntry, String> {
    let resp = client
        .post(format!("{}/api/networks/{}/entries", base_url, network_id))
        .json(&serde_json::json!({
            "entity_type": entity_type,
            "entity_id": entity_id,
            "entity_name": entity_name,
            "color": color,
            "label": label,
            "notes": notes,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("API error: {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

pub async fn update_intel_entry(
    client: &Client,
    base_url: &str,
    network_id: i64,
    entry_id: i64,
    entity_type: &str,
    entity_id: i64,
    entity_name: &str,
    color: &str,
    label: &str,
    notes: Option<&str>,
) -> Result<IntelEntry, String> {
    let request_body = serde_json::json!({
        "entity_type": entity_type,
        "entity_id": entity_id,
        "entity_name": entity_name,
        "color": color,
        "label": label,
        "notes": notes,
    });

    let endpoint = format!(
        "{}/api/networks/{}/entries/{}",
        base_url, network_id, entry_id
    );
    let mut resp = client
        .patch(&endpoint)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status() == reqwest::StatusCode::METHOD_NOT_ALLOWED {
        resp = client
            .put(&endpoint)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
    }

    if !resp.status().is_success() {
        return Err(format!("API error: {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

pub async fn remove_intel_entry(
    client: &Client,
    base_url: &str,
    network_id: i64,
    entry_id: i64,
) -> Result<(), String> {
    let resp = client
        .delete(format!(
            "{}/api/networks/{}/entries/{}",
            base_url, network_id, entry_id
        ))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("API error: {}", resp.status()));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Network access
// ---------------------------------------------------------------------------

pub async fn add_network_access(
    client: &Client,
    base_url: &str,
    network_id: i64,
    accessible_type: &str,
    accessible_id: i64,
    accessible_name: &str,
    permission: &str,
) -> Result<NetworkAccess, String> {
    let resp = client
        .post(format!("{}/api/networks/{}/access", base_url, network_id))
        .json(&serde_json::json!({
            "accessible_type": accessible_type,
            "accessible_id": accessible_id,
            "accessible_name": accessible_name,
            "permission": permission,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("API error: {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

pub async fn remove_network_access(
    client: &Client,
    base_url: &str,
    network_id: i64,
    access_id: i64,
) -> Result<(), String> {
    let resp = client
        .delete(format!(
            "{}/api/networks/{}/access/{}",
            base_url, network_id, access_id
        ))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("API error: {}", resp.status()));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Network scans
// ---------------------------------------------------------------------------

pub async fn share_scan(
    client: &Client,
    base_url: &str,
    network_id: i64,
    scan_type: &str,
    raw_text: &str,
    solar_system: Option<&str>,
) -> Result<NetworkScan, String> {
    let resp = client
        .post(format!("{}/api/networks/{}/scans", base_url, network_id))
        .json(&serde_json::json!({
            "scan_type": scan_type,
            "raw_text": raw_text,
            "solar_system": solar_system,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("API error: {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

pub async fn fetch_scans(
    client: &Client,
    base_url: &str,
    network_id: i64,
    page: i64,
) -> Result<PaginatedScans, String> {
    let resp = client
        .get(format!(
            "{}/api/networks/{}/scans?page={}",
            base_url, network_id, page
        ))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("API error: {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Entity search
// ---------------------------------------------------------------------------

pub async fn search_entities(
    client: &Client,
    base_url: &str,
    query: &str,
    category: Option<&str>,
) -> Result<Vec<SearchResult>, String> {
    let mut url = format!(
        "{}/api/search?query={}",
        base_url,
        urlencoding::encode(query)
    );
    if let Some(cat) = category {
        url.push_str(&format!("&category={}", urlencoding::encode(cat)));
    }

    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("API error: {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}
