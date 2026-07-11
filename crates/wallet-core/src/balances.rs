use taurvia_solana::{get_metadata, get_prices, lamports_to_sol, WRAPPED_SOL_MINT};
use models::{ActivityItem, TokenBalance, TokenInfo, WalletSnapshot};
use std::collections::HashMap;
use std::time::Duration;

use crate::session::WalletService;
use crate::WalletError;

const MARKET_DATA_BUDGET: Duration = Duration::from_secs(4);

impl WalletService {
    pub async fn get_snapshot(&self) -> Result<WalletSnapshot, WalletError> {
        let exists = self.wallet_exists();
        let network = self.wallet_network();
        if !exists {
            return Ok(WalletSnapshot {
                exists: false,
                unlocked: false,
                network,
                public_key: None,
                sol_balance: None,
                sol_price_usd: None,
                sol_value_usd: None,
                total_portfolio_usd: None,
                tokens: None,
            });
        }

        let unlocked = self.is_unlocked();
        let public_key = self.get_public_key();

        if !unlocked {
            return Ok(WalletSnapshot {
                exists: true,
                unlocked: false,
                network,
                public_key,
                sol_balance: None,
                sol_price_usd: None,
                sol_value_usd: None,
                total_portfolio_usd: None,
                tokens: None,
            });
        }

        let pubkey = self.require_pubkey()?;
        let rpc = self.rpc_handle();
        let (lamports, mut tokens) = rpc
            .get_balances_parallel(&pubkey)
            .await
            .map_err(WalletError::Operation)?;

        // Curated labels first; Jupiter enrichment is budgeted so unlock/dashboard
        // never waits on a slow market-data API.
        apply_local_metadata(&mut tokens);
        let mints: Vec<String> = tokens.iter().map(|token| token.mint.clone()).collect();
        let sol_mint = WRAPPED_SOL_MINT.to_string();
        let enrichment = tokio::time::timeout(MARKET_DATA_BUDGET, async {
            tokio::join!(
                get_metadata(&mints),
                get_prices(&mints),
                get_prices(std::slice::from_ref(&sol_mint)),
            )
        })
        .await;

        let mut sol_price_usd = None;
        if let Ok((metadata, prices, sol_prices)) = enrichment {
            apply_remote_enrichment(
                &mut tokens,
                metadata.unwrap_or_default(),
                prices.unwrap_or_default(),
            );
            sol_price_usd = sol_prices
                .ok()
                .and_then(|prices| prices.get(WRAPPED_SOL_MINT).copied());
        }

        let sol_balance = lamports_to_sol(lamports);
        let sol_value_usd = sol_price_usd.map(|price| price * sol_balance);
        let tokens_value: f64 = tokens.iter().filter_map(|token| token.value_usd).sum();
        let total_portfolio_usd = Some(sol_value_usd.unwrap_or(0.0) + tokens_value);

        Ok(WalletSnapshot {
            exists: true,
            unlocked: true,
            network,
            public_key,
            sol_balance: Some(sol_balance),
            sol_price_usd,
            sol_value_usd,
            total_portfolio_usd,
            tokens: Some(tokens),
        })
    }

    pub async fn get_sol_balance(&self) -> Result<f64, WalletError> {
        let pubkey = self.require_pubkey()?;
        let lamports = self
            .rpc_handle()
            .get_balance(&pubkey)
            .await
            .map_err(WalletError::Operation)?;
        Ok(lamports_to_sol(lamports))
    }

    pub async fn get_token_balances(&self) -> Result<Vec<TokenBalance>, WalletError> {
        let pubkey = self.require_pubkey()?;
        let mut tokens = self
            .rpc_handle()
            .get_token_balances(&pubkey)
            .await
            .map_err(WalletError::Operation)?;
        enrich_token_balances(&mut tokens).await;
        Ok(tokens)
    }

    pub async fn get_activity(&self, limit: usize) -> Result<Vec<ActivityItem>, WalletError> {
        let pubkey = self.require_pubkey()?;
        self.rpc_handle()
            .get_activity(&pubkey, limit)
            .await
            .map_err(WalletError::Operation)
    }
}

async fn enrich_token_balances(tokens: &mut [TokenBalance]) {
    if tokens.is_empty() {
        return;
    }
    apply_local_metadata(tokens);
    let mints: Vec<String> = tokens.iter().map(|token| token.mint.clone()).collect();
    let enrichment = tokio::time::timeout(MARKET_DATA_BUDGET, async {
        tokio::join!(get_metadata(&mints), get_prices(&mints))
    })
    .await;
    if let Ok((metadata, prices)) = enrichment {
        apply_remote_enrichment(
            tokens,
            metadata.unwrap_or_default(),
            prices.unwrap_or_default(),
        );
    }
}

fn apply_local_metadata(tokens: &mut [TokenBalance]) {
    for token in tokens.iter_mut() {
        if let Some(info) = taurvia_solana::resolve_mint_local(&token.mint) {
            token.symbol = info.symbol;
            token.name = info.name;
            if info.decimals > 0 {
                token.decimals = info.decimals;
                if let Ok(raw) = token.amount.parse::<u64>() {
                    token.ui_amount = raw as f64 / 10f64.powi(info.decimals as i32);
                }
            }
            token.logo_uri = info.logo_uri;
        }
    }
}

fn apply_remote_enrichment(
    tokens: &mut [TokenBalance],
    metadata: HashMap<String, TokenInfo>,
    prices: HashMap<String, f64>,
) {
    for token in tokens.iter_mut() {
        if let Some(info) = metadata.get(&token.mint) {
            // Prefer real symbols over shortened-mint fallbacks.
            if !info.symbol.contains("...") {
                token.symbol = info.symbol.clone();
                token.name = info.name.clone();
            }
            if info.decimals > 0 {
                token.decimals = info.decimals;
                if let Ok(raw) = token.amount.parse::<u64>() {
                    token.ui_amount = raw as f64 / 10f64.powi(info.decimals as i32);
                }
            }
            if info.logo_uri.is_some() {
                token.logo_uri = info.logo_uri.clone();
            }
        }
        if let Some(price) = prices.get(&token.mint).copied() {
            token.price_usd = Some(price);
            token.value_usd = Some(price * token.ui_amount);
        }
    }
}
