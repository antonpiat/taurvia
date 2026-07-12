use crate::jupiter::{
    jupiter_get_timeout, shorten_mint, JUPITER_MARKET_DATA_TIMEOUT, WRAPPED_SOL_MINT,
};
use anyhow::{Context, Result};
use models::TokenInfo;
use moka::future::Cache;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct JupiterToken {
    id: Option<String>,
    address: Option<String>,
    symbol: Option<String>,
    name: Option<String>,
    decimals: Option<u8>,
    #[serde(alias = "logoURI", alias = "icon")]
    logo_uri: Option<String>,
}

fn metadata_cache() -> &'static Cache<String, TokenInfo> {
    static CACHE: OnceLock<Cache<String, TokenInfo>> = OnceLock::new();
    CACHE.get_or_init(|| {
        Cache::builder()
            .time_to_live(Duration::from_secs(3600))
            .max_capacity(2048)
            .build()
    })
}

fn curated_token_info(mint: &str) -> Option<TokenInfo> {
    match mint {
        WRAPPED_SOL_MINT => Some(TokenInfo {
            mint: WRAPPED_SOL_MINT.to_string(),
            symbol: "SOL".into(),
            name: "Solana".into(),
            decimals: 9,
            logo_uri: Some(
                "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png".into(),
            ),
        }),
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => Some(TokenInfo {
            mint: mint.to_string(),
            symbol: "USDC".into(),
            name: "USD Coin".into(),
            decimals: 6,
            logo_uri: Some(
                "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/logo.png".into(),
            ),
        }),
        "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => Some(TokenInfo {
            mint: mint.to_string(),
            symbol: "USDT".into(),
            name: "Tether USD".into(),
            decimals: 6,
            logo_uri: Some(
                "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB/logo.svg".into(),
            ),
        }),
        "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN" => Some(TokenInfo {
            mint: mint.to_string(),
            symbol: "JUP".into(),
            name: "Jupiter".into(),
            decimals: 6,
            logo_uri: Some("https://static.jup.ag/jup/icon.png".into()),
        }),
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263" => Some(TokenInfo {
            mint: mint.to_string(),
            symbol: "BONK".into(),
            name: "Bonk".into(),
            decimals: 5,
            logo_uri: Some(
                "https://arweave.net/hQiPZOsRZXGXBJd_82PhVdlM_hACsT_q6wqwf5cSY7I".into(),
            ),
        }),
        _ => None,
    }
}

fn fallback_token_info(mint: &str, decimals: Option<u8>) -> TokenInfo {
    let short = shorten_mint(mint);
    TokenInfo {
        mint: mint.to_string(),
        symbol: short.clone(),
        name: format!("Token {short}"),
        decimals: decimals.unwrap_or(0),
        logo_uri: None,
    }
}

fn token_id(token: &JupiterToken) -> Option<&str> {
    token.id.as_deref().or(token.address.as_deref())
}

fn map_jupiter_token(mint: &str, token: JupiterToken) -> TokenInfo {
    let short = shorten_mint(mint);
    TokenInfo {
        mint: mint.to_string(),
        symbol: token.symbol.filter(|s| !s.is_empty()).unwrap_or(short.clone()),
        name: token
            .name
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("Token {short}")),
        decimals: token.decimals.unwrap_or(0),
        logo_uri: token.logo_uri.filter(|s| !s.is_empty()),
    }
}

/// Instant curated metadata only (no network). Used to paint UI before Jupiter returns.
pub fn resolve_mint_local(mint: &str) -> Option<TokenInfo> {
    if mint.eq_ignore_ascii_case("sol") {
        return curated_token_info(WRAPPED_SOL_MINT);
    }
    curated_token_info(mint)
}

pub async fn resolve_mint(mint: &str) -> Result<TokenInfo> {
    if let Some(info) = resolve_mint_local(mint) {
        return Ok(info);
    }

    if let Some(cached) = metadata_cache().get(mint).await {
        return Ok(cached);
    }

    match fetch_mint_metadata(mint).await {
        Ok(info) => {
            metadata_cache().insert(mint.to_string(), info.clone()).await;
            Ok(info)
        }
        Err(_) => Ok(fallback_token_info(mint, None)),
    }
}

pub async fn get_metadata(mints: &[String]) -> Result<HashMap<String, TokenInfo>> {
    let mut result = HashMap::new();
    let mut missing = Vec::new();

    for mint in mints {
        if mint.eq_ignore_ascii_case("sol") {
            result.insert(
                mint.clone(),
                curated_token_info(WRAPPED_SOL_MINT).expect("SOL curated"),
            );
            continue;
        }
        if let Some(info) = curated_token_info(mint) {
            result.insert(mint.clone(), info);
            continue;
        }
        if let Some(cached) = metadata_cache().get(mint).await {
            result.insert(mint.clone(), cached);
        } else {
            missing.push(mint.clone());
        }
    }

    for chunk in missing.chunks(50) {
        match fetch_mints_metadata(chunk).await {
            Ok(fetched) => {
                for mint in chunk {
                    if let Some(info) = fetched.get(mint) {
                        metadata_cache().insert(mint.clone(), info.clone()).await;
                        result.insert(mint.clone(), info.clone());
                    } else {
                        // Do not cache misses — transient API failures should retry.
                        result.insert(mint.clone(), fallback_token_info(mint, None));
                    }
                }
            }
            Err(_) => {
                for mint in chunk {
                    result.insert(mint.clone(), fallback_token_info(mint, None));
                }
            }
        }
    }

    Ok(result)
}

async fn fetch_mint_metadata(mint: &str) -> Result<TokenInfo> {
    let fetched = fetch_mints_metadata(&[mint.to_string()]).await?;
    fetched
        .into_iter()
        .find(|(id, _)| id == mint)
        .map(|(_, info)| info)
        .context("token metadata not found")
}

async fn fetch_mints_metadata(mints: &[String]) -> Result<HashMap<String, TokenInfo>> {
    if mints.is_empty() {
        return Ok(HashMap::new());
    }

    // Mint addresses are base58; commas are safe in this query string.
    let query = mints.join(",");
    let response = jupiter_get_timeout(
        &format!("/tokens/v2/search?query={query}"),
        JUPITER_MARKET_DATA_TIMEOUT,
    )
    .await?;
    let tokens: Vec<JupiterToken> = response
        .json()
        .await
        .context("failed to decode Jupiter token metadata")?;

    let wanted: HashMap<&str, ()> = mints.iter().map(|mint| (mint.as_str(), ())).collect();
    let mut result = HashMap::new();
    for token in tokens {
        let Some(id) = token_id(&token).map(str::to_string) else {
            continue;
        };
        if !wanted.contains_key(id.as_str()) {
            continue;
        }
        result.insert(id.clone(), map_jupiter_token(&id, token));
    }
    Ok(result)
}

fn search_cache() -> &'static Cache<String, Vec<TokenInfo>> {
    static CACHE: OnceLock<Cache<String, Vec<TokenInfo>>> = OnceLock::new();
    CACHE.get_or_init(|| {
        Cache::builder()
            .time_to_live(Duration::from_secs(30 * 60))
            .max_capacity(256)
            .build()
    })
}

const SEARCH_RESULT_LIMIT: usize = 20;

/// Keyword / mint search via Jupiter Tokens v2. Cached per lowercased query.
pub async fn search_tokens(query: &str) -> Result<Vec<TokenInfo>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let key = trimmed.to_ascii_lowercase();
    if let Some(cached) = search_cache().get(&key).await {
        return Ok(cached);
    }

    let encoded = urlencoding_encode(trimmed);
    let response = jupiter_get_timeout(
        &format!("/tokens/v2/search?query={encoded}"),
        JUPITER_MARKET_DATA_TIMEOUT,
    )
    .await?;
    let tokens: Vec<JupiterToken> = response
        .json()
        .await
        .context("failed to decode Jupiter token search")?;

    let mut results = Vec::new();
    let mut seen = HashMap::new();
    for token in tokens {
        let Some(id) = token_id(&token).map(str::to_string) else {
            continue;
        };
        if seen.contains_key(&id) {
            continue;
        }
        seen.insert(id.clone(), ());
        let info = map_jupiter_token(&id, token);
        // Warm mint metadata cache for later resolve/quote.
        metadata_cache().insert(id, info.clone()).await;
        results.push(info);
        if results.len() >= SEARCH_RESULT_LIMIT {
            break;
        }
    }

    search_cache().insert(key, results.clone()).await;
    Ok(results)
}

/// Minimal URL-encode for Jupiter query (spaces + reserved chars).
fn urlencoding_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len() * 3);
    for b in value.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

pub fn curated_major_mints() -> Vec<&'static str> {
    vec![
        WRAPPED_SOL_MINT,
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
        "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", // USDT
        "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN",   // JUP
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", // BONK
    ]
}
