use anyhow::{Context, Result};
use models::{ActivityItem, NetworkDescriptor};
use serde::Deserialize;

use crate::tokens::u256_to_f64;
use alloy::primitives::U256;

#[derive(Deserialize)]
struct EtherscanResponse {
    status: String,
    result: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EtherscanTx {
    hash: String,
    time_stamp: String,
    to: String,
    value: String,
    is_error: Option<String>,
    txreceipt_status: Option<String>,
}

pub async fn activity(
    descriptor: NetworkDescriptor,
    address: &str,
    limit: usize,
) -> Result<Vec<ActivityItem>> {
    let Some(api) = descriptor.explorer_api else {
        return Ok(Vec::new());
    };
    let url = format!(
        "{api}?module=account&action=txlist&address={}&page=1&offset={}&sort=desc",
        address,
        limit.clamp(1, 25)
    );
    let resp: EtherscanResponse = taurvia_chain::http_client()
        .get(url)
        .send()
        .await
        .context("etherscan request")?
        .json()
        .await
        .context("etherscan json")?;
    if resp.status != "1" {
        return Ok(Vec::new());
    }
    let txs: Vec<EtherscanTx> = serde_json::from_value(resp.result).unwrap_or_default();
    let me = address.to_lowercase();
    Ok(txs
        .into_iter()
        .map(|tx| {
            let failed =
                tx.is_error.as_deref() == Some("1") || tx.txreceipt_status.as_deref() == Some("0");
            let incoming = tx.to.to_lowercase() == me;
            let raw: U256 = tx.value.parse().unwrap_or(U256::ZERO);
            let amount = u256_to_f64(raw, 18);
            let direction = if incoming { "in" } else { "out" };
            let description = if incoming {
                format!("Received {amount:.6} {}", descriptor.native_symbol)
            } else {
                format!("Sent {:.6} {}", amount, descriptor.native_symbol)
            };
            ActivityItem {
                txid: tx.hash,
                timestamp: tx.time_stamp.parse().ok(),
                status: if failed {
                    "failed".into()
                } else {
                    "confirmed".into()
                },
                direction: direction.into(),
                amount: if raw.is_zero() { None } else { Some(amount) },
                amount_symbol: Some(descriptor.native_symbol.to_string()),
                description,
            }
        })
        .collect())
}
