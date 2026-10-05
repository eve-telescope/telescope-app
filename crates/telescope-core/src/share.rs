//! Public scan sharing via `/api/share` (no auth needed).

use serde::{Deserialize, Serialize};

use crate::api::create_client;

#[derive(Debug, Clone, Deserialize)]
pub struct ShareResponse {
    pub code: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareData {
    pub code: String,
    pub pilots: Vec<String>,
}

#[derive(Serialize)]
struct CreateShare<'a> {
    pilots: Vec<&'a str>,
}

pub async fn create_share(base_url: &str, pilot_names: &str) -> Result<ShareResponse, String> {
    let pilots = pilot_names
        .lines()
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .collect();
    let response = create_client()?
        .post(format!("{}/api/share", base_url))
        .header("Accept", "application/json")
        .json(&CreateShare { pilots })
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!(
            "Failed to create share: HTTP {}",
            response.status()
        ));
    }
    response.json().await.map_err(|e| e.to_string())
}

pub async fn fetch_share(base_url: &str, code: &str) -> Result<ShareData, String> {
    let response = create_client()?
        .get(format!(
            "{}/api/share/{}",
            base_url,
            urlencoding::encode(code)
        ))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!(
            "Failed to load share {}: HTTP {}",
            code,
            response.status()
        ));
    }
    response.json().await.map_err(|e| e.to_string())
}
