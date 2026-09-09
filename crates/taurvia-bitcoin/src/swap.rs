use anyhow::{anyhow, bail, Context, Result};
use models::SwapQuote;
use serde::Deserialize;

const THORNODE: &str = "https://thornode.ninerealms.com";

#[derive(Debug, Deserialize)]
pub struct ThorchainQuote {
    pub inbound_address: String,
    pub memo: String,
    pub expected_amount_out: Option<String>,
    pub fees: Option<ThorFees>,
    pub recommended_min_amount_in: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ThorFees {
    pub outbound: Option<String>,
    pub affiliate: Option<String>,
    pub asset: Option<String>,
}

pub async fn quote_swap(
    from_asset: &str,
    to_asset: &str,
    amount_1e8: u64,
    destination: &str,
    slippage_bps: u16,
) -> Result<ThorchainQuote> {
    if amount_1e8 == 0 {
        bail!("swap amount must be greater than zero");
    }
    let url = format!(
        "{THORNODE}/thorchain/quote/swap?from_asset={from_asset}&to_asset={to_asset}&amount={amount_1e8}&destination={destination}&tolerance_bps={slippage_bps}"
    );
    let response = taurvia_chain::http_client()
        .get(&url)
        .send()
        .await
        .context("thorchain quote")?;
    let status = response.status();
    let body = response.text().await.context("thorchain body")?;
    if !status.is_success() {
        bail!("Thorchain quote failed ({status}): {body}");
    }
    let parsed: ThorchainQuote = serde_json::from_str(&body).context("thorchain json")?;
    if let Some(err) = parsed.error.as_deref().filter(|s| !s.is_empty()) {
        bail!("Thorchain: {err}");
    }
    if parsed.inbound_address.is_empty() || parsed.memo.is_empty() {
        bail!("Thorchain quote missing inbound address or memo");
    }
    if let Some(min) = parsed
        .recommended_min_amount_in
        .as_deref()
        .and_then(|s| s.parse::<u64>().ok())
    {
        if amount_1e8 < min {
            bail!("amount is below Thorchain's recommended minimum");
        }
    }
    Ok(parsed)
}

pub fn quote_to_swap(
    input_mint: &str,
    output_mint: &str,
    input_symbol: &str,
    output_symbol: &str,
    in_amount_ui: f64,
    slippage_bps: u16,
    quote: &ThorchainQuote,
) -> Result<SwapQuote> {
    let out_1e8: f64 = quote
        .expected_amount_out
        .as_deref()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0.0);
    let fee_1e8: f64 = quote
        .fees
        .as_ref()
        .and_then(|f| f.outbound.as_deref())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    Ok(SwapQuote {
        input_mint: input_mint.to_string(),
        output_mint: output_mint.to_string(),
        input_symbol: input_symbol.to_string(),
        output_symbol: output_symbol.to_string(),
        in_amount: ((in_amount_ui * 1e8).round() as u64).to_string(),
        out_amount: quote.expected_amount_out.clone().unwrap_or_default(),
        in_amount_ui,
        out_amount_ui: out_1e8 / 1e8,
        price_impact_pct: None,
        network_fee: fee_1e8 / 1e8,
        fee_symbol: quote
            .fees
            .as_ref()
            .and_then(|f| f.asset.clone())
            .unwrap_or_else(|| "BTC".into()),
        slippage_bps,
        route: "thorchain".into(),
    })
}

pub fn thor_asset(mint: &str) -> Result<&'static str> {
    let m = mint.trim();
    if is_btc(m) {
        return Ok("BTC.BTC");
    }
    if is_eth_native(m) {
        return Ok("ETH.ETH");
    }
    if is_sol_native(m) {
        return Ok("SOL.SOL");
    }
    Err(anyhow!(
        "Thorchain supports BTC, ETH, and SOL natives in this wallet"
    ))
}

pub fn is_btc(mint: &str) -> bool {
    mint.eq_ignore_ascii_case("btc") || mint.eq_ignore_ascii_case("btc:native")
}

pub fn is_eth_native(mint: &str) -> bool {
    mint.eq_ignore_ascii_case("eth")
        || mint.eq_ignore_ascii_case("eth:native")
        || mint.eq_ignore_ascii_case("eip155:1:native")
}

pub fn is_sol_native(mint: &str) -> bool {
    mint.eq_ignore_ascii_case("sol")
        || mint.eq_ignore_ascii_case("sol:native")
        || mint == taurvia_solana_mint()
}

fn taurvia_solana_mint() -> &'static str {
    "So11111111111111111111111111111111111111112"
}
