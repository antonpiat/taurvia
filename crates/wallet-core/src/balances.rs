use aegis_solana::lamports_to_sol;
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
                tokens: None,
            });
        }

        let pubkey = self.require_pubkey()?;
        let (lamports, tokens) = self
            .rpc
            .get_balances_parallel(&pubkey)
            .await
            .map_err(WalletError::Operation)?;

        Ok(WalletSnapshot {
            exists: true,
            unlocked: true,
            public_key,
            sol_balance: Some(lamports_to_sol(lamports)),
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
        self.rpc
            .get_token_balances(&pubkey)
            .await
            .map_err(WalletError::Operation)
    }

    pub async fn get_activity(&self, limit: usize) -> Result<Vec<ActivityItem>, WalletError> {
        let pubkey = self.require_pubkey()?;
        self.rpc
            .get_activity(&pubkey, limit)
            .await
            .map_err(WalletError::Operation)
    }
}
