use crate::jupiter::{jupiter_get, WRAPPED_SOL_MINT};
use anyhow::{Context, Result};
use moka::future::Cache;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct PriceEntry {
    #[serde(rename = "usdPrice")]
    usd_price: Option<f64>,
}

fn price_cache() -> &'static Cache<String, f64> {
    static CACHE: OnceLock<Cache<String, f64>> = OnceLock::new();
    CACHE.get_or_init(|| {
        Cache::builder()
            .time_to_live(Duration::from_secs(45))
            .max_capacity(512)
            .build()
    })
}

pub async fn get_prices(mints: &[String]) -> Result<HashMap<String, f64>> {
    let mut result = HashMap::new();
    let mut missing = Vec::new();

    for mint in mints {
        if let Some(price) = price_cache().get(mint).await {
            result.insert(mint.clone(), price);
        } else {
            missing.push(mint.clone());
        }
    }

    for chunk in missing.chunks(50) {
        if chunk.is_empty() {
            continue;
        }
        let ids = chunk.join(",");
        let response = jupiter_get(&format!("/price/v3?ids={ids}")).await?;
        let payload: HashMap<String, PriceEntry> = response
            .json()
            .await
            .context("failed to decode Jupiter price response")?;

        for mint in chunk {
            if let Some(price) = payload.get(mint).and_then(|entry| entry.usd_price) {
                price_cache().insert(mint.clone(), price).await;
                result.insert(mint.clone(), price);
            }
        }
    }

    Ok(result)
}

pub async fn get_sol_price() -> Result<Option<f64>> {
    let prices = get_prices(&[WRAPPED_SOL_MINT.to_string()]).await?;
    Ok(prices.get(WRAPPED_SOL_MINT).copied())
}
