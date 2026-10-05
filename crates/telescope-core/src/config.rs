//! Endpoints, overridable at build time through environment variables.

pub const IDENTIFIER: &str = "com.timkunze.telescope";

pub fn api_base_url() -> String {
    option_env!("TELESCOPE_API_BASE_URL")
        .unwrap_or("https://eve-telescope.com")
        .trim_end_matches('/')
        .to_string()
}

pub struct ReverbConfig {
    pub key: &'static str,
    pub host: &'static str,
    pub port: u16,
    pub tls: bool,
}

pub fn reverb() -> ReverbConfig {
    ReverbConfig {
        key: option_env!("TELESCOPE_REVERB_APP_KEY").unwrap_or("04pcwy13bvcyjoio6mf6"),
        host: option_env!("TELESCOPE_REVERB_HOST").unwrap_or("ws.eve-telescope.com"),
        port: option_env!("TELESCOPE_REVERB_PORT")
            .and_then(|p| p.parse().ok())
            .unwrap_or(443),
        tls: option_env!("TELESCOPE_REVERB_SCHEME").unwrap_or("https") == "https",
    }
}
