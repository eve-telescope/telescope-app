pub mod esi;
pub mod zkill;

use reqwest::Client;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn create_client() -> Result<Client, String> {
    let user_agent = format!(
        "Telescope/{} (eve-telescope.com; github.com/eve-telescope/telescope-app)",
        VERSION
    );

    Client::builder()
        .user_agent(user_agent)
        .build()
        .map_err(|e| e.to_string())
}
