use crate::jupiter::{jupiter_get, jupiter_post, shorten_mint, WRAPPED_SOL_MINT};
use crate::lamports_to_sol;
use crate::rpc::SolanaRpc;
use crate::token_metadata::resolve_mint;
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use models::{SwapQuote, SwapResult};
use serde::{Deserialize, Serialize};
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::VersionedTransaction,
};
use std::str::FromStr;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JupiterQuoteResponse {
    input_mint: String,
    output_mint: String,
    in_amount: String,
    out_amount: String,
    price_impact_pct: Option<serde_json::Value>,
    #[serde(default)]
    prioritization_fee_lamports: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JupiterSwapRequest {
    user_public_key: String,
    quote_response: serde_json::Value,
    wrap_and_unwrap_sol: bool,
    dynamic_compute_unit_limit: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JupiterSwapResponse {
    swap_transaction: String,
}

impl SolanaRpc {
    pub async fn quote_swap(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount_raw: u64,
        slippage_bps: u16,
    ) -> Result<SwapQuote> {
        if input_mint == output_mint {
            anyhow::bail!("input and output mints must differ");
        }
        if amount_raw == 0 {
            anyhow::bail!("swap amount must be greater than zero");
        }

        let path = format!(
            "/swap/v1/quote?inputMint={input_mint}&outputMint={output_mint}&amount={amount_raw}&slippageBps={slippage_bps}"
        );
        let response = jupiter_get(&path).await?;
        let quote: JupiterQuoteResponse = response
            .json()
            .await
            .context("failed to decode Jupiter quote")?;

        let (input_meta, output_meta) =
            tokio::try_join!(resolve_mint(&quote.input_mint), resolve_mint(&quote.output_mint))?;

        let in_amount: u64 = quote.in_amount.parse().unwrap_or(amount_raw);
        let out_amount: u64 = quote
            .out_amount
            .parse()
            .context("invalid outAmount from Jupiter")?;
        let price_impact_pct = quote.price_impact_pct.and_then(|value| match value {
            serde_json::Value::String(text) => text.parse().ok(),
            serde_json::Value::Number(number) => number.as_f64(),
            _ => None,
        });
        let network_fee_lamports = quote.prioritization_fee_lamports.unwrap_or(5_000);

        Ok(SwapQuote {
            input_mint: quote.input_mint,
            output_mint: quote.output_mint,
            input_symbol: input_meta.symbol,
            output_symbol: output_meta.symbol,
            in_amount: in_amount.to_string(),
            out_amount: out_amount.to_string(),
            in_amount_ui: in_amount as f64 / 10f64.powi(input_meta.decimals as i32),
            out_amount_ui: out_amount as f64 / 10f64.powi(output_meta.decimals as i32),
            price_impact_pct,
            network_fee_lamports,
            network_fee_sol: lamports_to_sol(network_fee_lamports),
            slippage_bps,
        })
    }

    pub async fn execute_swap(
        &self,
        keypair: &Keypair,
        input_mint: &str,
        output_mint: &str,
        amount_raw: u64,
        slippage_bps: u16,
    ) -> Result<SwapResult> {
        let path = format!(
            "/swap/v1/quote?inputMint={input_mint}&outputMint={output_mint}&amount={amount_raw}&slippageBps={slippage_bps}"
        );
        let quote_response: serde_json::Value = jupiter_get(&path)
            .await?
            .json()
            .await
            .context("failed to decode Jupiter quote for swap")?;

        let swap_body = JupiterSwapRequest {
            user_public_key: keypair.pubkey().to_string(),
            quote_response,
            wrap_and_unwrap_sol: true,
            dynamic_compute_unit_limit: true,
        };

        let swap_response: JupiterSwapResponse = jupiter_post("/swap/v1/swap", &swap_body)
            .await?
            .json()
            .await
            .context("failed to decode Jupiter swap transaction")?;

        let tx_bytes = BASE64
            .decode(swap_response.swap_transaction)
            .context("failed to decode swap transaction")?;
        let unsigned: VersionedTransaction =
            bincode::deserialize(&tx_bytes).context("failed to deserialize versioned transaction")?;
        let signed = VersionedTransaction::try_new(unsigned.message, &[keypair])
            .map_err(|e| anyhow!("failed to sign swap transaction: {e}"))?;

        let sig = self
            .client()
            .send_transaction_with_config(
                &signed,
                RpcSendTransactionConfig {
                    skip_preflight: false,
                    ..Default::default()
                },
            )
            .await
            .context("failed to send swap transaction")?;

        let latest = self
            .client()
            .get_latest_blockhash()
            .await
            .context("failed to fetch blockhash for confirmation")?;
        self.client()
            .confirm_transaction_with_spinner(&sig, &latest, CommitmentConfig::confirmed())
            .await
            .context("failed to confirm swap transaction")?;

        Ok(SwapResult {
            signature: sig.to_string(),
            status: "confirmed".into(),
        })
    }
}

pub fn normalize_mint(mint: &str) -> Result<String> {
    if mint.eq_ignore_ascii_case("sol") {
        return Ok(WRAPPED_SOL_MINT.to_string());
    }
    let _ = solana_sdk::pubkey::Pubkey::from_str(mint)
        .map_err(|_| anyhow!("invalid mint address: {}", shorten_mint(mint)))?;
    Ok(mint.to_string())
}

pub async fn ui_amount_to_raw(mint: &str, amount_ui: f64) -> Result<u64> {
    if amount_ui <= 0.0 {
        anyhow::bail!("amount must be greater than zero");
    }
    let info = resolve_mint(mint).await?;
    Ok((amount_ui * 10f64.powi(info.decimals as i32)).round() as u64)
}
