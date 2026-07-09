use anyhow::{Context, Result};
use reqwest::Client;
use std::sync::OnceLock;
use std::time::Duration;

pub const WRAPPED_SOL_MINT: &str = "So11111111111111111111111111111111111111112";
pub const JUPITER_API_BASE: &str = "https://api.jup.ag";

/// Keep market-data calls short so unlock/dashboard never stalls on Jupiter.
pub const JUPITER_MARKET_DATA_TIMEOUT: Duration = Duration::from_secs(3);

pub fn jupiter_api_key() -> Option<String> {
    std::env::var("AEGIS_JUPITER_API_KEY")
        .ok()
        .filter(|value| !value.is_empty())
}

pub fn http_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("aegis-wallet/0.1")
            .build()
            .expect("failed to build HTTP client")
    })
}

pub async fn jupiter_get(path: &str) -> Result<reqwest::Response> {
    jupiter_get_timeout(path, Duration::from_secs(20)).await
}

pub async fn jupiter_get_timeout(path: &str, timeout: Duration) -> Result<reqwest::Response> {
    let url = format!("{JUPITER_API_BASE}{path}");
    let mut request = http_client().get(&url).timeout(timeout);
    if let Some(api_key) = jupiter_api_key() {
        request = request.header("x-api-key", api_key);
    }
    request
        .send()
        .await
        .with_context(|| format!("Jupiter GET failed: {path}"))?
        .error_for_status()
        .with_context(|| format!("Jupiter GET status error: {path}"))
}

pub async fn jupiter_post(path: &str, body: &impl serde::Serialize) -> Result<reqwest::Response> {
    let url = format!("{JUPITER_API_BASE}{path}");
    let mut request = http_client().post(&url).json(body);
    if let Some(api_key) = jupiter_api_key() {
        request = request.header("x-api-key", api_key);
    }
    request
        .send()
        .await
        .with_context(|| format!("Jupiter POST failed: {path}"))?
        .error_for_status()
        .with_context(|| format!("Jupiter POST status error: {path}"))
}

pub fn shorten_mint(mint: &str) -> String {
    if mint.len() <= 10 {
        return mint.to_string();
    }
    format!("{}...{}", &mint[..4], &mint[mint.len() - 4..])
}
