use anyhow::{anyhow, bail, Context, Result};
use models::{SwapQuote, SwapResult};
use serde::Deserialize;
use std::str::FromStr;

use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol;
use alloy::sol_types::SolCall;

use crate::derive::EvmSigner;
use crate::rpc::EvmRpc;
use crate::tokens::{curated_tokens, f64_to_u256};

sol! {
    function approve(address spender, uint256 amount) external returns (bool);
}

const NATIVE: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ZeroxQuote {
    buy_amount: Option<String>,
    sell_amount: Option<String>,
    min_buy_amount: Option<String>,
    transaction: Option<ZeroxTx>,
    fees: Option<ZeroxFees>,
    issues: Option<ZeroxIssues>,
    #[serde(default)]
    liquidity_available: Option<bool>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ZeroxTx {
    to: String,
    data: String,
    value: Option<String>,
    gas: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ZeroxFees {
    gas: Option<ZeroxGas>,
}

#[derive(Debug, Deserialize)]
struct ZeroxGas {
    amount: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ZeroxIssues {
    allowance: Option<ZeroxAllowance>,
}

#[derive(Debug, Deserialize)]
struct ZeroxAllowance {
    spender: Option<String>,
}

impl EvmRpc {
    pub async fn quote_swap(
        &self,
        taker: &str,
        sell_token: &str,
        buy_token: &str,
        amount_ui: f64,
        slippage_bps: u16,
        api_key: Option<&str>,
    ) -> Result<SwapQuote> {
        let quote = self
            .zerox_quote(
                taker,
                sell_token,
                buy_token,
                amount_ui,
                slippage_bps,
                api_key,
            )
            .await?;
        let sell_meta = evm_asset_meta(self.descriptor.id, sell_token)?;
        let buy_meta = evm_asset_meta(self.descriptor.id, buy_token)?;
        let sell_amount = quote.sell_amount.unwrap_or_default();
        let buy_amount = quote
            .buy_amount
            .or(quote.min_buy_amount)
            .ok_or_else(|| anyhow!("0x quote missing buyAmount"))?;
        let in_raw: f64 = sell_amount.parse().unwrap_or(0.0);
        let out_raw: f64 = buy_amount.parse().unwrap_or(0.0);
        let gas_fee = quote
            .fees
            .and_then(|f| f.gas)
            .and_then(|g| g.amount)
            .and_then(|a| a.parse::<f64>().ok())
            .map(|wei| wei / 1e18)
            .unwrap_or(0.0);

        Ok(SwapQuote {
            input_mint: sell_token.to_string(),
            output_mint: buy_token.to_string(),
            input_symbol: sell_meta.0,
            output_symbol: buy_meta.0,
            in_amount: sell_amount,
            out_amount: buy_amount,
            in_amount_ui: in_raw / 10f64.powi(sell_meta.1 as i32),
            out_amount_ui: out_raw / 10f64.powi(buy_meta.1 as i32),
            price_impact_pct: None,
            network_fee: gas_fee,
            fee_symbol: "ETH".into(),
            slippage_bps,
            route: "0x".into(),
        })
    }

    pub async fn execute_swap(
        &self,
        signer: &EvmSigner,
        sell_token: &str,
        buy_token: &str,
        amount_ui: f64,
        slippage_bps: u16,
        api_key: Option<&str>,
    ) -> Result<SwapResult> {
        let quote = self
            .zerox_quote(
                &signer.address,
                sell_token,
                buy_token,
                amount_ui,
                slippage_bps,
                api_key,
            )
            .await?;
        if let Some(allowance) = quote.issues.as_ref().and_then(|i| i.allowance.as_ref()) {
            if let Some(spender) = allowance.spender.as_deref() {
                self.approve(signer, sell_token, spender, amount_ui).await?;
            }
        }
        let tx = quote
            .transaction
            .ok_or_else(|| anyhow!("0x quote has no transaction (API key may be required)"))?;
        self.send_calldata(
            signer,
            &tx.to,
            &tx.data,
            tx.value.as_deref(),
            tx.gas.as_deref(),
        )
        .await
    }

    async fn zerox_quote(
        &self,
        taker: &str,
        sell_token: &str,
        buy_token: &str,
        amount_ui: f64,
        slippage_bps: u16,
        api_key: Option<&str>,
    ) -> Result<ZeroxQuote> {
        let chain_id = self.descriptor.eip155_chain_id.unwrap_or(1);
        let sell = to_zerox_token(sell_token);
        let buy = to_zerox_token(buy_token);
        if sell.eq_ignore_ascii_case(&buy) {
            bail!("input and output tokens must differ");
        }
        let decimals = evm_asset_meta(self.descriptor.id, sell_token)?.1;
        let sell_amount = f64_to_u256(amount_ui, decimals);
        if sell_amount.is_zero() {
            bail!("swap amount must be greater than zero");
        }
        let url = format!(
            "https://api.0x.org/swap/allowance-holder/quote?chainId={chain_id}&sellToken={sell}&buyToken={buy}&sellAmount={sell_amount}&taker={taker}&slippageBps={slippage_bps}"
        );
        let mut req = taurvia_chain::http_client()
            .get(&url)
            .header("0x-version", "v2");
        if let Some(key) = api_key.map(str::trim).filter(|s| !s.is_empty()) {
            req = req.header("0x-api-key", key);
        }
        let response = req.send().await.context("0x quote request")?;
        let status = response.status();
        let body = response.text().await.context("0x quote body")?;
        if !status.is_success() {
            bail!("0x quote failed ({status}): {body}");
        }
        let parsed: ZeroxQuote = serde_json::from_str(&body).context("0x quote json")?;
        if parsed.liquidity_available == Some(false) {
            bail!("no 0x liquidity for this pair");
        }
        if parsed.transaction.is_none() && parsed.message.is_some() {
            bail!(parsed.message.unwrap_or_default());
        }
        Ok(parsed)
    }

    async fn approve(
        &self,
        signer: &EvmSigner,
        token: &str,
        spender: &str,
        _amount_ui: f64,
    ) -> Result<()> {
        if is_native(token) {
            return Ok(());
        }
        let spender = Address::from_str(spender).context("invalid 0x spender")?;
        let call = approveCall {
            spender,
            amount: U256::MAX,
        };
        self.send_calldata(
            signer,
            token,
            &format!("0x{}", hex::encode(call.abi_encode())),
            None,
            None,
        )
        .await?;
        Ok(())
    }

    async fn send_calldata(
        &self,
        signer: &EvmSigner,
        to: &str,
        data: &str,
        value: Option<&str>,
        _gas: Option<&str>,
    ) -> Result<SwapResult> {
        let pk = hex::encode(signer.secret_bytes());
        let local: PrivateKeySigner = pk.parse().context("evm signer")?;
        let url = self
            .rpc_url
            .parse()
            .map_err(|e| anyhow!("invalid evm rpc url: {e}"))?;
        let wallet = alloy::network::EthereumWallet::from(local);
        let provider = ProviderBuilder::new().wallet(wallet).connect_http(url);
        let to_addr = Address::from_str(to).context("invalid 0x target")?;
        let data_bytes = Bytes::from(
            hex::decode(data.strip_prefix("0x").unwrap_or(data)).context("0x calldata")?,
        );
        let value = parse_u256(value.unwrap_or("0"))?;
        let mut tx = TransactionRequest::default()
            .with_to(to_addr)
            .with_input(data_bytes)
            .with_value(value);
        tx.set_chain_id(self.descriptor.eip155_chain_id.unwrap_or(1));
        let pending = provider
            .send_transaction(tx)
            .await
            .context("0x swap send")?;
        let hash = *pending.tx_hash();
        Ok(SwapResult {
            signature: format!("{hash:#x}"),
            status: "submitted".into(),
        })
    }
}

fn to_zerox_token(asset: &str) -> String {
    if is_native(asset) {
        NATIVE.to_string()
    } else {
        asset.to_string()
    }
}

fn is_native(asset: &str) -> bool {
    asset.eq_ignore_ascii_case("eth")
        || asset.eq_ignore_ascii_case("native")
        || asset.eq_ignore_ascii_case(NATIVE)
}

fn evm_asset_meta(network_id: &str, asset: &str) -> Result<(String, u8)> {
    if is_native(asset) {
        return Ok(("ETH".into(), 18));
    }
    curated_tokens(network_id)
        .iter()
        .find(|t| t.address.eq_ignore_ascii_case(asset))
        .map(|t| (t.symbol.to_string(), t.decimals))
        .ok_or_else(|| anyhow!("unsupported Ethereum token"))
}

fn parse_u256(value: &str) -> Result<U256> {
    let v = value.trim();
    if v.is_empty() || v == "0" || v == "0x0" {
        return Ok(U256::ZERO);
    }
    if let Some(hex) = v.strip_prefix("0x").or_else(|| v.strip_prefix("0X")) {
        return U256::from_str_radix(hex, 16).map_err(|e| anyhow!("invalid value: {e}"));
    }
    U256::from_str(v).map_err(|e| anyhow!("invalid value: {e}"))
}
