use aegis_solana::{get_metadata, get_prices, lamports_to_sol, WRAPPED_SOL_MINT};
use models::{ActivityItem, TokenBalance, WalletSnapshot};

use crate::session::WalletService;
use crate::WalletError;

impl WalletService {
    pub async fn get_snapshot(&self) -> Result<WalletSnapshot, WalletError> {
        let exists = self.wallet_exists();
        if !exists {
            return Ok(WalletSnapshot {
                exists: false,
                unlocked: false,
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
                public_key,
                sol_balance: None,
                sol_price_usd: None,
                sol_value_usd: None,
                total_portfolio_usd: None,
                tokens: None,
            });
        }

        let pubkey = self.require_pubkey()?;
        let (lamports, mut tokens) = self
            .rpc
            .get_balances_parallel(&pubkey)
            .await
            .map_err(WalletError::Operation)?;

        enrich_token_balances(&mut tokens).await;
        let sol_balance = lamports_to_sol(lamports);
        let sol_price_usd = get_prices(&[WRAPPED_SOL_MINT.to_string()])
            .await
            .ok()
            .and_then(|prices| prices.get(WRAPPED_SOL_MINT).copied());
        let sol_value_usd = sol_price_usd.map(|price| price * sol_balance);
        let tokens_value: f64 = tokens.iter().filter_map(|token| token.value_usd).sum();
        let total_portfolio_usd = Some(sol_value_usd.unwrap_or(0.0) + tokens_value);

        Ok(WalletSnapshot {
            exists: true,
            unlocked: true,
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
            .rpc
            .get_balance(&pubkey)
            .await
            .map_err(WalletError::Operation)?;
        Ok(lamports_to_sol(lamports))
    }

    pub async fn get_token_balances(&self) -> Result<Vec<TokenBalance>, WalletError> {
        let pubkey = self.require_pubkey()?;
        let mut tokens = self
            .rpc
            .get_token_balances(&pubkey)
            .await
            .map_err(WalletError::Operation)?;
        enrich_token_balances(&mut tokens).await;
        Ok(tokens)
    }

    pub async fn get_activity(&self, limit: usize) -> Result<Vec<ActivityItem>, WalletError> {
        let pubkey = self.require_pubkey()?;
        self.rpc
            .get_activity(&pubkey, limit)
            .await
            .map_err(WalletError::Operation)
    }
}

async fn enrich_token_balances(tokens: &mut [TokenBalance]) {
    if tokens.is_empty() {
        return;
    }

    let mints: Vec<String> = tokens.iter().map(|token| token.mint.clone()).collect();
    let (metadata, prices) = tokio::join!(get_metadata(&mints), get_prices(&mints));
    let metadata = metadata.unwrap_or_default();
    let prices = prices.unwrap_or_default();

    for token in tokens.iter_mut() {
        if let Some(info) = metadata.get(&token.mint) {
            token.symbol = info.symbol.clone();
            token.name = info.name.clone();
            if info.decimals > 0 {
                token.decimals = info.decimals;
                if let Ok(raw) = token.amount.parse::<u64>() {
                    token.ui_amount = raw as f64 / 10f64.powi(info.decimals as i32);
                }
            }
            token.logo_uri = info.logo_uri.clone();
        }
        if let Some(price) = prices.get(&token.mint).copied() {
            token.price_usd = Some(price);
            token.value_usd = Some(price * token.ui_amount);
        }
    }
}
