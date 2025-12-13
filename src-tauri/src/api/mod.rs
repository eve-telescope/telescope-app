pub mod esi;
pub mod zkill;

use reqwest::Client;

pub fn create_client() -> Result<Client, String> {
    Client::builder()
        .user_agent("Telescope EVE Intel Tool - Contact: github.com/telescope")
        .build()
        .map_err(|e| e.to_string())
}


