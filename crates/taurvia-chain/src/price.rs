use anyhow::{Context, Result};
use moka::future::Cache;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;

use crate::http_client;

const PRICE_TTL: Duration = Duration::from_secs(45);

fn native_cache() -> &'static Cache<String, f64> {
    static CACHE: OnceLock<Cache<String, f64>> = OnceLock::new();
    CACHE.get_or_init(|| {
        Cache::builder()
            .time_to_live(PRICE_TTL)
            .max_capacity(32)
            .build()
    })
}

fn token_cache() -> &'static Cache<String, f64> {
    static CACHE: OnceLock<Cache<String, f64>> = OnceLock::new();
    CACHE.get_or_init(|| {
        Cache::builder()
            .time_to_live(PRICE_TTL)
            .max_capacity(256)
            .build()
    })
}

#[derive(Deserialize)]
struct CoinGeckoPrice(HashMap<String, HashMap<String, f64>>);

pub async fn native_price_usd(coingecko_id: &str) -> Result<f64> {
    if let Some(hit) = native_cache().get(coingecko_id).await {
        return Ok(hit);
    }
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={coingecko_id}&vs_currencies=usd"
    );
    let body: CoinGeckoPrice = http_client()
        .get(url)
        .send()
        .await
        .context("coingecko price request failed")?
        .error_for_status()
        .context("coingecko price HTTP error")?
        .json()
        .await
        .context("coingecko price JSON")?;
    let price = body
        .0
        .get(coingecko_id)
        .and_then(|m| m.get("usd").copied())
        .context("coingecko missing usd price")?;
    native_cache().insert(coingecko_id.to_string(), price).await;
    Ok(price)
}

/// ERC-20 USD prices keyed by lowercase contract address (Ethereum platform).
pub async fn token_prices_usd(
    platform: &str,
    contracts: &[String],
) -> Result<HashMap<String, f64>> {
    if contracts.is_empty() {
        return Ok(HashMap::new());
    }
    let mut out = HashMap::new();
    let mut missing = Vec::new();
    for contract in contracts {
        let key = format!("{platform}:{}", contract.to_lowercase());
        if let Some(hit) = token_cache().get(&key).await {
            out.insert(contract.to_lowercase(), hit);
        } else {
            missing.push(contract.to_lowercase());
        }
    }
    if missing.is_empty() {
        return Ok(out);
    }
    let joined = missing.join(",");
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/token_price/{platform}?contract_addresses={joined}&vs_currencies=usd"
    );
    let body: HashMap<String, HashMap<String, f64>> = http_client()
        .get(url)
        .send()
        .await
        .context("coingecko token price request failed")?
        .error_for_status()
        .context("coingecko token price HTTP error")?
        .json()
        .await
        .context("coingecko token price JSON")?;
    for (contract, prices) in body {
        if let Some(usd) = prices.get("usd").copied() {
            let key = format!("{platform}:{}", contract.to_lowercase());
            token_cache().insert(key, usd).await;
            out.insert(contract.to_lowercase(), usd);
        }
    }
    Ok(out)
}
