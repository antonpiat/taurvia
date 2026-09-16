use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use models::{
    ActivityItem, NetworkDescriptor, SendPreview, SendResult, TokenBalance, WalletSnapshot,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::Duration;

use crate::derive::{intent_digest, validate_address, SuiSigner};
use crate::{SUI_COIN_TYPE, SUI_DECIMALS, SUI_NATIVE};

const MARKET_DATA_BUDGET: Duration = Duration::from_secs(4);
const MIST_PER_SUI: f64 = 1_000_000_000.0;
const DEFAULT_GAS_BUDGET: u64 = 10_000_000;
const MAX_TOKEN_ROWS: usize = 15;

pub struct SuiRpc {
    rpc_url: String,
    descriptor: NetworkDescriptor,
}

impl SuiRpc {
    pub fn new(rpc_url: &str, descriptor: NetworkDescriptor) -> Self {
        Self {
            rpc_url: rpc_url.trim_end_matches('/').to_string(),
            descriptor,
        }
    }

    pub async fn snapshot(&self, address: &str) -> Result<WalletSnapshot> {
        let price_id = self.descriptor.coingecko_id.unwrap_or("sui");
        let (balances, price) = tokio::join!(
            self.all_balances(address),
            tokio::time::timeout(
                MARKET_DATA_BUDGET,
                taurvia_chain::native_price_usd(price_id)
            ),
        );
        let all = balances.context("suix_getAllBalances failed")?;
        let mut native_mist = 0u64;
        let mut token_rows = Vec::new();
        for bal in all {
            let raw: u64 = bal.total_balance.parse().unwrap_or(0);
            if is_sui_coin(&bal.coin_type) {
                native_mist = raw;
                continue;
            }
            if raw == 0 {
                continue;
            }
            token_rows.push((bal.coin_type, raw));
            if token_rows.len() >= MAX_TOKEN_ROWS {
                break;
            }
        }
        let tokens = self.enrich_token_balances(token_rows).await;
        let native_balance = native_mist as f64 / MIST_PER_SUI;
        let native_price_usd = price.ok().and_then(|r| r.ok());
        let native_value_usd = native_price_usd.map(|p| p * native_balance);
        let tokens_value: f64 = tokens.iter().filter_map(|t| t.value_usd).sum();
        Ok(WalletSnapshot {
            exists: true,
            unlocked: true,
            network: self.descriptor.id.to_string(),
            public_key: Some(address.to_string()),
            native_balance: Some(native_balance),
            native_symbol: self.descriptor.native_symbol.to_string(),
            native_price_usd,
            native_value_usd,
            total_portfolio_usd: Some(native_value_usd.unwrap_or(0.0) + tokens_value),
            tokens: Some(tokens),
            chains: Vec::new(),
            account_name: String::new(),
            import_kind: models::ImportKind::Mnemonic,
            enabled_networks: Vec::new(),
            can_reveal_mnemonic: false,
        })
    }

    pub async fn activity(&self, address: &str, limit: usize) -> Result<Vec<ActivityItem>> {
        let limit = limit.clamp(1, 25);
        let result: QueryBlocks = self
            .rpc(
                "suix_queryTransactionBlocks",
                json!([
                    {
                        "filter": { "FromOrToAddress": { "addr": address } },
                        "options": { "showBalanceChanges": true, "showEffects": true }
                    },
                    null,
                    limit,
                    true
                ]),
            )
            .await
            .unwrap_or(QueryBlocks { data: Vec::new() });
        let me = normalize_addr(address);
        Ok(result
            .data
            .into_iter()
            .map(|tx| {
                let mut in_mist: i128 = 0;
                let mut out_mist: i128 = 0;
                for change in tx.balance_changes.unwrap_or_default() {
                    if !is_sui_coin(&change.coin_type) {
                        continue;
                    }
                    if !owner_is(change.owner.as_ref(), &me) {
                        continue;
                    }
                    let amt: i128 = change.amount.parse().unwrap_or(0);
                    if amt >= 0 {
                        in_mist += amt;
                    } else {
                        out_mist += -amt;
                    }
                }
                let (direction, amount_mist) = if out_mist > in_mist {
                    ("out", out_mist - in_mist)
                } else {
                    ("in", in_mist - out_mist)
                };
                let status = tx
                    .effects
                    .as_ref()
                    .and_then(|e| e.status.as_ref())
                    .and_then(|s| s.status.as_deref())
                    .unwrap_or("unknown");
                ActivityItem {
                    txid: tx.digest,
                    timestamp: tx.timestamp_ms.map(|ms| ms / 1000),
                    status: status.to_string(),
                    direction: direction.into(),
                    amount: Some(amount_mist as f64 / MIST_PER_SUI),
                    amount_symbol: Some("SUI".into()),
                    description: if direction == "out" {
                        "Sent SUI".into()
                    } else {
                        "Received SUI".into()
                    },
                }
            })
            .collect())
    }

    pub async fn preview_send(
        &self,
        from: &str,
        to: &str,
        amount: f64,
        asset: Option<&str>,
    ) -> Result<SendPreview> {
        validate_address(to)?;
        let (symbol, coin_type, decimals) = self.resolve_asset(asset).await?;
        let mist = ui_to_raw(amount, decimals)?;
        let tx = self
            .build_unsigned(from, to, mist, &coin_type)
            .await
            .context("failed to build Sui transfer")?;
        let fee = self
            .dry_run_fee(&tx.tx_bytes)
            .await
            .unwrap_or(DEFAULT_GAS_BUDGET);
        Ok(SendPreview {
            from: from.to_string(),
            to: to.to_string(),
            token: symbol,
            amount: format!("{amount}"),
            network_name: self.descriptor.name.to_string(),
            estimated_fee: fee as f64 / MIST_PER_SUI,
            fee_symbol: self.descriptor.native_symbol.to_string(),
            creates_token_account: false,
        })
    }

    pub async fn send(
        &self,
        signer: &SuiSigner,
        to: &str,
        amount: f64,
        asset: Option<&str>,
    ) -> Result<SendResult> {
        validate_address(to)?;
        let (_, coin_type, decimals) = self.resolve_asset(asset).await?;
        let mist = ui_to_raw(amount, decimals)?;
        let built = self
            .build_unsigned(&signer.address, to, mist, &coin_type)
            .await?;
        let raw_tx = B64
            .decode(built.tx_bytes.as_bytes())
            .context("decode Sui txBytes")?;
        let digest = intent_digest(&raw_tx);
        let sig = signer.sign_intent_digest(&digest);
        let mut serialized = Vec::with_capacity(97);
        serialized.push(0x00);
        serialized.extend_from_slice(&sig);
        serialized.extend_from_slice(&signer.public_key_bytes());
        let signature = B64.encode(serialized);
        let result: ExecuteResult = self
            .rpc(
                "sui_executeTransactionBlock",
                json!([
                    built.tx_bytes,
                    [signature],
                    {
                        "showInput": false,
                        "showEffects": true,
                        "showEvents": false,
                        "showObjectChanges": false,
                        "showBalanceChanges": false
                    },
                    "WaitForLocalExecution"
                ]),
            )
            .await
            .context("sui_executeTransactionBlock")?;
        if let Some(status) = result
            .effects
            .as_ref()
            .and_then(|e| e.status.as_ref())
            .and_then(|s| s.status.as_deref())
        {
            if status != "success" {
                let err = result
                    .effects
                    .as_ref()
                    .and_then(|e| e.status.as_ref())
                    .and_then(|s| s.error.clone())
                    .unwrap_or_else(|| "transaction failed".into());
                bail!("Sui transaction failed: {err}");
            }
        }
        Ok(SendResult {
            txid: result.digest,
            status: "submitted".into(),
        })
    }

    async fn resolve_asset(&self, asset: Option<&str>) -> Result<(String, String, u8)> {
        let a = asset.unwrap_or(SUI_NATIVE).trim();
        if is_native_asset(a) {
            return Ok((
                self.descriptor.native_symbol.to_string(),
                SUI_COIN_TYPE.to_string(),
                SUI_DECIMALS,
            ));
        }
        let meta = self.coin_metadata(a).await.context("unknown Sui coin")?;
        let (decimals, symbol, _) = metadata_fields(Some(&meta), a);
        Ok((symbol, a.to_string(), decimals))
    }

    async fn build_unsigned(
        &self,
        from: &str,
        to: &str,
        amount: u64,
        coin_type: &str,
    ) -> Result<BuiltTx> {
        if is_sui_coin(coin_type) {
            let coins = self.coin_ids(from, SUI_COIN_TYPE, 50).await?;
            if coins.is_empty() {
                bail!("no SUI coins to spend");
            }
            self.rpc(
                "unsafe_paySui",
                json!([
                    from,
                    coins,
                    [to],
                    [amount.to_string()],
                    DEFAULT_GAS_BUDGET.to_string()
                ]),
            )
            .await
            .context("unsafe_paySui")
        } else {
            let coins = self.coin_ids(from, coin_type, 50).await?;
            if coins.is_empty() {
                bail!("no coins of this type to spend");
            }
            let gas = self
                .coin_ids(from, SUI_COIN_TYPE, 1)
                .await?
                .into_iter()
                .next()
                .context("need SUI for gas")?;
            self.rpc(
                "unsafe_pay",
                json!([
                    from,
                    coins,
                    [to],
                    [amount.to_string()],
                    gas,
                    DEFAULT_GAS_BUDGET.to_string()
                ]),
            )
            .await
            .context("unsafe_pay")
        }
    }

    async fn dry_run_fee(&self, tx_bytes: &str) -> Result<u64> {
        let result: DryRun = self
            .rpc("sui_dryRunTransactionBlock", json!([tx_bytes]))
            .await
            .context("sui_dryRunTransactionBlock")?;
        let used = result.effects.gas_used.unwrap_or_default();
        let computation: u64 = used.computation_cost.parse().unwrap_or(0);
        let storage: u64 = used.storage_cost.parse().unwrap_or(0);
        let rebate: u64 = used.storage_rebate.parse().unwrap_or(0);
        Ok(computation
            .saturating_add(storage)
            .saturating_sub(rebate)
            .max(1))
    }

    async fn all_balances(&self, address: &str) -> Result<Vec<CoinBalance>> {
        self.rpc("suix_getAllBalances", json!([address]))
            .await
            .context("suix_getAllBalances")
    }

    async fn enrich_token_balances(&self, rows: Vec<(String, u64)>) -> Vec<TokenBalance> {
        let futs = rows
            .into_iter()
            .map(|(coin_type, raw)| self.token_row(coin_type, raw));
        futures::future::join_all(futs).await
    }

    async fn token_row(&self, coin_type: String, raw: u64) -> TokenBalance {
        let meta = self.coin_metadata(&coin_type).await;
        let (decimals, symbol, name) = metadata_fields(meta.as_ref(), &coin_type);
        let ui_amount = raw as f64 / 10f64.powi(i32::from(decimals));
        TokenBalance {
            mint: coin_type,
            symbol,
            name,
            amount: raw.to_string(),
            decimals,
            ui_amount,
            logo_uri: meta.and_then(|m| m.icon_url),
            price_usd: None,
            value_usd: None,
        }
    }

    async fn coin_metadata(&self, coin_type: &str) -> Option<CoinMetadata> {
        self.rpc("suix_getCoinMetadata", json!([coin_type]))
            .await
            .unwrap_or(None)
    }

    async fn coin_ids(&self, owner: &str, coin_type: &str, max: usize) -> Result<Vec<String>> {
        let max = max.max(1);
        let mut ids = Vec::with_capacity(max);
        let mut cursor: Option<String> = None;
        loop {
            let page_size = (max - ids.len()).min(50);
            let page: CoinPage = self
                .rpc(
                    "suix_getCoins",
                    json!([owner, coin_type, cursor, page_size]),
                )
                .await
                .context("suix_getCoins")?;
            for coin in page.data {
                ids.push(coin.coin_object_id);
                if ids.len() >= max {
                    return Ok(ids);
                }
            }
            if !page.has_next_page || page.next_cursor.is_none() {
                break;
            }
            cursor = page.next_cursor;
        }
        Ok(ids)
    }

    async fn rpc<T: for<'de> Deserialize<'de>>(&self, method: &str, params: Value) -> Result<T> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        let resp: RpcResponse<T> = taurvia_chain::http_client()
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("{method} request failed"))?
            .error_for_status()
            .with_context(|| format!("{method} HTTP error"))?
            .json()
            .await
            .with_context(|| format!("{method} JSON"))?;
        if let Some(err) = resp.error {
            bail!("{method}: {}", err.message);
        }
        resp.result
            .ok_or_else(|| anyhow!("{method}: missing result"))
    }
}

fn metadata_fields(meta: Option<&CoinMetadata>, coin_type: &str) -> (u8, String, String) {
    let decimals = meta
        .and_then(|m| m.decimals)
        .unwrap_or(9)
        .min(u8::MAX as u32) as u8;
    let symbol = meta
        .and_then(|m| m.symbol.clone())
        .unwrap_or_else(|| short_coin_type(coin_type));
    let name = meta
        .and_then(|m| m.name.clone())
        .unwrap_or_else(|| symbol.clone());
    (decimals, symbol, name)
}

fn is_native_asset(asset: &str) -> bool {
    let a = asset.trim();
    a.eq_ignore_ascii_case(SUI_NATIVE) || a.eq_ignore_ascii_case("native") || is_sui_coin(a)
}

fn is_sui_coin(coin_type: &str) -> bool {
    let n = coin_type.trim();
    let n = n
        .strip_prefix("0x")
        .or_else(|| n.strip_prefix("0X"))
        .unwrap_or(n)
        .trim_start_matches('0');
    n.eq_ignore_ascii_case("2::sui::SUI")
}

fn short_coin_type(coin_type: &str) -> String {
    coin_type
        .rsplit("::")
        .next()
        .unwrap_or(coin_type)
        .chars()
        .take(8)
        .collect()
}

fn normalize_addr(address: &str) -> String {
    address.trim().to_ascii_lowercase()
}

fn owner_is(owner: Option<&Value>, me: &str) -> bool {
    let Some(owner) = owner else {
        return false;
    };
    if let Some(addr) = owner.as_str() {
        return normalize_addr(addr) == me;
    }
    owner
        .get("AddressOwner")
        .and_then(|v| v.as_str())
        .map(|a| normalize_addr(a) == me)
        .unwrap_or(false)
}

fn ui_to_raw(amount: f64, decimals: u8) -> Result<u64> {
    if amount <= 0.0 || !amount.is_finite() {
        bail!("amount must be positive");
    }
    let scale = 10f64.powi(i32::from(decimals));
    let raw = (amount * scale).round();
    if raw < 1.0 || raw > u64::MAX as f64 {
        bail!("invalid amount");
    }
    Ok(raw as u64)
}

#[derive(Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Deserialize)]
struct RpcError {
    message: String,
}

#[derive(Deserialize)]
struct CoinBalance {
    #[serde(rename = "coinType", default)]
    coin_type: String,
    #[serde(rename = "totalBalance", default)]
    total_balance: String,
}

#[derive(Deserialize)]
struct CoinMetadata {
    #[serde(default)]
    decimals: Option<u32>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    symbol: Option<String>,
    #[serde(rename = "iconUrl", default)]
    icon_url: Option<String>,
}

#[derive(Deserialize)]
struct CoinPage {
    #[serde(default)]
    data: Vec<CoinObject>,
    #[serde(rename = "nextCursor", default)]
    next_cursor: Option<String>,
    #[serde(rename = "hasNextPage", default)]
    has_next_page: bool,
}

#[derive(Deserialize)]
struct CoinObject {
    #[serde(rename = "coinObjectId")]
    coin_object_id: String,
}

#[derive(Deserialize)]
struct BuiltTx {
    #[serde(rename = "txBytes")]
    tx_bytes: String,
}

#[derive(Deserialize)]
struct DryRun {
    #[serde(default)]
    effects: DryEffects,
}

#[derive(Default, Deserialize)]
struct DryEffects {
    #[serde(rename = "gasUsed", default)]
    gas_used: Option<GasUsed>,
}

#[derive(Default, Deserialize)]
struct GasUsed {
    #[serde(rename = "computationCost", default)]
    computation_cost: String,
    #[serde(rename = "storageCost", default)]
    storage_cost: String,
    #[serde(rename = "storageRebate", default)]
    storage_rebate: String,
}

#[derive(Deserialize)]
struct ExecuteResult {
    digest: String,
    #[serde(default)]
    effects: Option<TxEffects>,
}

#[derive(Deserialize)]
struct QueryBlocks {
    #[serde(default)]
    data: Vec<TxBlock>,
}

#[derive(Deserialize)]
struct TxBlock {
    digest: String,
    #[serde(rename = "timestampMs", default)]
    timestamp_ms: Option<i64>,
    #[serde(rename = "balanceChanges", default)]
    balance_changes: Option<Vec<BalanceChange>>,
    #[serde(default)]
    effects: Option<TxEffects>,
}

#[derive(Deserialize)]
struct TxEffects {
    #[serde(default)]
    status: Option<TxStatus>,
}

#[derive(Deserialize)]
struct TxStatus {
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Deserialize)]
struct BalanceChange {
    #[serde(default)]
    owner: Option<Value>,
    #[serde(rename = "coinType", default)]
    coin_type: String,
    #[serde(default)]
    amount: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_coin_type_matches_padded_package_id() {
        assert!(is_sui_coin("0x2::sui::SUI"));
        assert!(is_sui_coin(
            "0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI"
        ));
        assert!(!is_sui_coin("0x2::usdc::USDC"));
    }
}
