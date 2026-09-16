use models::{ActivityItem, ChainFamily, ChainSnapshot, TokenBalance, TokenInfo, WalletSnapshot};
use std::collections::HashMap;
use std::time::Duration;
use taurvia_solana::{get_metadata, get_prices, lamports_to_sol, WRAPPED_SOL_MINT};

use crate::session::WalletService;
use crate::WalletError;

const MARKET_DATA_BUDGET: Duration = Duration::from_secs(4);

impl WalletService {
    pub async fn get_snapshot(&self) -> Result<WalletSnapshot, WalletError> {
        let exists = self.wallet_exists();
        let network = self.wallet_network();
        let desc = models::require_network(&network);
        let account_name = self.account_name();
        let import_kind = self.import_kind();
        let enabled_networks = self.enabled_network_ids();
        let can_reveal_mnemonic = exists && import_kind.has_mnemonic();

        let mut snap = WalletSnapshot::empty(network.clone(), desc.native_symbol.to_string());
        snap.exists = exists;
        snap.account_name = account_name;
        snap.import_kind = import_kind;
        snap.enabled_networks = enabled_networks.clone();
        snap.can_reveal_mnemonic = can_reveal_mnemonic;

        if !exists {
            return Ok(snap);
        }

        let unlocked = self.is_unlocked();
        let public_key = self.get_public_key();
        snap.unlocked = unlocked;
        snap.public_key = public_key.clone();

        if !unlocked {
            return Ok(snap);
        }

        let descriptors = self.snapshot_descriptors();
        let descriptors = match self.with_session(|k| {
            descriptors
                .into_iter()
                .filter(|d| k.has_family(d.family))
                .collect::<Vec<_>>()
        }) {
            Ok(d) => d,
            Err(_) => return Ok(snap),
        };

        // Keep activated chains on the dashboard even when an RPC is down (do not invent a 0).
        let chains: Vec<_> =
            futures::future::join_all(descriptors.iter().copied().map(|d| self.chain_snapshot(d)))
                .await
                .into_iter()
                .zip(descriptors.iter().copied())
                .map(|(result, desc)| match result {
                    Ok(chain) => chain,
                    Err(_) => self.pending_chain(desc),
                })
                .collect();

        let total: f64 = chains.iter().filter_map(|c| c.total_usd).sum();
        let active = chains
            .iter()
            .find(|c| c.network == network)
            .cloned()
            .or_else(|| chains.first().cloned());

        snap.total_portfolio_usd = Some(total);
        snap.chains = chains;
        if let Some(active) = active {
            snap.network = active.network.clone();
            snap.public_key = active.public_key.clone();
            snap.native_balance = active.native_balance;
            snap.native_symbol = active.native_symbol.clone();
            snap.native_price_usd = active.native_price_usd;
            snap.native_value_usd = active.native_value_usd;
            snap.tokens = active.tokens.clone();
        }
        Ok(snap)
    }

    async fn chain_snapshot(
        &self,
        desc: &'static models::NetworkDescriptor,
    ) -> Result<ChainSnapshot, WalletError> {
        if desc.family == ChainFamily::Solana {
            return self.solana_chain_snapshot(desc).await;
        }
        let url = self.endpoint_for(desc.id);
        let address = self.with_session(|k| k.address(desc.family, desc.is_testnet))??;
        let snap = match desc.family {
            ChainFamily::Evm => {
                taurvia_evm::EvmRpc::new(&url, *desc)
                    .snapshot(&address)
                    .await
            }
            ChainFamily::Bitcoin => {
                taurvia_bitcoin::BtcRpc::new(&url, *desc)
                    .snapshot(&address)
                    .await
            }
            ChainFamily::Sui => {
                taurvia_sui::SuiRpc::new(&url, *desc)
                    .snapshot(&address)
                    .await
            }
            ChainFamily::Solana => unreachable!(),
        }
        .map_err(WalletError::Operation)?;
        Ok(chain_from_legacy(snap))
    }

    fn pending_chain(&self, desc: &'static models::NetworkDescriptor) -> ChainSnapshot {
        let public_key = self
            .with_session(|k| k.address(desc.family, desc.is_testnet).ok())
            .ok()
            .flatten();
        ChainSnapshot {
            network: desc.id.to_string(),
            public_key,
            native_balance: None,
            native_symbol: desc.native_symbol.to_string(),
            native_price_usd: None,
            native_value_usd: None,
            total_usd: None,
            tokens: None,
        }
    }

    async fn solana_chain_snapshot(
        &self,
        desc: &'static models::NetworkDescriptor,
    ) -> Result<ChainSnapshot, WalletError> {
        let pubkey = self.require_pubkey()?;
        let rpc = if desc.id == self.wallet_network() {
            self.rpc_handle()
        } else {
            taurvia_solana::SolanaRpc::new(Some(&self.endpoint_for(desc.id)))
        };
        let (lamports, mut tokens) = rpc
            .get_balances_parallel(&pubkey)
            .await
            .map_err(WalletError::Operation)?;

        apply_local_metadata(&mut tokens);
        let mut mints: Vec<String> = tokens.iter().map(|token| token.mint.clone()).collect();
        if !mints.iter().any(|mint| mint == WRAPPED_SOL_MINT) {
            mints.push(WRAPPED_SOL_MINT.to_string());
        }
        let enrichment = tokio::time::timeout(MARKET_DATA_BUDGET, async {
            tokio::join!(get_metadata(&mints), get_prices(&mints))
        })
        .await;

        let mut native_price_usd = None;
        if let Ok((metadata, prices)) = enrichment {
            let prices = prices.unwrap_or_default();
            native_price_usd = prices.get(WRAPPED_SOL_MINT).copied();
            apply_remote_enrichment(&mut tokens, metadata.unwrap_or_default(), prices);
        }

        let native_balance = lamports_to_sol(lamports);
        let native_value_usd = native_price_usd.map(|price| price * native_balance);
        let tokens_value: f64 = tokens.iter().filter_map(|token| token.value_usd).sum();
        Ok(ChainSnapshot {
            network: desc.id.to_string(),
            public_key: Some(pubkey.to_string()),
            native_balance: Some(native_balance),
            native_symbol: "SOL".into(),
            native_price_usd,
            native_value_usd,
            total_usd: Some(native_value_usd.unwrap_or(0.0) + tokens_value),
            tokens: Some(tokens),
        })
    }

    pub async fn get_activity(&self, limit: usize) -> Result<Vec<ActivityItem>, WalletError> {
        let desc = self.active_descriptor();
        if desc.family == ChainFamily::Solana {
            let pubkey = self.require_pubkey()?;
            return self
                .rpc_handle()
                .get_activity(&pubkey, limit)
                .await
                .map_err(WalletError::Operation);
        }
        let address = self.with_session(|k| k.address(desc.family, desc.is_testnet))??;
        match desc.family {
            ChainFamily::Evm => taurvia_evm::activity(*desc, &address, limit)
                .await
                .map_err(WalletError::Operation),
            ChainFamily::Bitcoin => {
                let url = self.endpoint_for(desc.id);
                taurvia_bitcoin::BtcRpc::new(&url, *desc)
                    .activity(&address, limit)
                    .await
                    .map_err(WalletError::Operation)
            }
            ChainFamily::Sui => {
                let url = self.endpoint_for(desc.id);
                taurvia_sui::SuiRpc::new(&url, *desc)
                    .activity(&address, limit)
                    .await
                    .map_err(WalletError::Operation)
            }
            ChainFamily::Solana => unreachable!(),
        }
    }
}

fn chain_from_legacy(snap: WalletSnapshot) -> ChainSnapshot {
    ChainSnapshot {
        network: snap.network,
        public_key: snap.public_key,
        native_balance: snap.native_balance,
        native_symbol: snap.native_symbol,
        native_price_usd: snap.native_price_usd,
        native_value_usd: snap.native_value_usd,
        total_usd: snap.total_portfolio_usd,
        tokens: snap.tokens,
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
