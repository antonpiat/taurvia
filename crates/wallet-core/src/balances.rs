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

        let mut sol = None;
        let mut evm = None;
        let mut btc = None;
        for d in self.snapshot_descriptors() {
            if !self.family_available_unlocked(d.family) {
                continue;
            }
            match d.family {
                ChainFamily::Solana => sol = Some(d),
                ChainFamily::Evm => evm = Some(d),
                ChainFamily::Bitcoin => btc = Some(d),
                ChainFamily::Sui => {}
            }
        }

        let (sol_r, evm_r, btc_r) = tokio::join!(
            async {
                match sol {
                    Some(d) => Some(self.chain_snapshot(d).await),
                    None => None,
                }
            },
            async {
                match evm {
                    Some(d) => Some(self.chain_snapshot(d).await),
                    None => None,
                }
            },
            async {
                match btc {
                    Some(d) => Some(self.chain_snapshot(d).await),
                    None => None,
                }
            },
        );

        // Option<Result<T,E>>: skip missing families, omit chains whose RPC failed (do not invent a 0).
        let mut chains = Vec::new();
        for chain in [sol_r, evm_r, btc_r].into_iter().flatten().flatten() {
            chains.push(chain);
        }

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

    fn family_available_unlocked(&self, family: ChainFamily) -> bool {
        self.with_session(|k| k.has_family(family)).unwrap_or(false)
    }

    async fn chain_snapshot(
        &self,
        desc: &'static models::NetworkDescriptor,
    ) -> Result<ChainSnapshot, WalletError> {
        match desc.family {
            ChainFamily::Solana => self.solana_chain_snapshot(desc).await,
            ChainFamily::Evm => self.evm_chain_snapshot(desc).await,
            ChainFamily::Bitcoin => self.bitcoin_chain_snapshot(desc).await,
            ChainFamily::Sui => Err(WalletError::Operation(anyhow::anyhow!(
                "Sui is not enabled yet"
            ))),
        }
    }

    async fn evm_chain_snapshot(
        &self,
        desc: &'static models::NetworkDescriptor,
    ) -> Result<ChainSnapshot, WalletError> {
        let url = self.endpoint_for(desc.id);
        let rpc = taurvia_evm::EvmRpc::new(&url, *desc);
        let address = self.with_session(|k| k.require_evm().map(|e| e.address.clone()))??;
        let snap = rpc
            .snapshot(&address)
            .await
            .map_err(WalletError::Operation)?;
        Ok(chain_from_legacy(snap))
    }

    async fn bitcoin_chain_snapshot(
        &self,
        desc: &'static models::NetworkDescriptor,
    ) -> Result<ChainSnapshot, WalletError> {
        let url = self.endpoint_for(desc.id);
        let rpc = taurvia_bitcoin::BtcRpc::new(&url, *desc);
        let address =
            self.with_session(|k| k.require_btc(desc.is_testnet).map(|s| s.address.clone()))??;
        let snap = rpc
            .snapshot(&address)
            .await
            .map_err(WalletError::Operation)?;
        Ok(chain_from_legacy(snap))
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
        match desc.family {
            ChainFamily::Solana => {
                let pubkey = self.require_pubkey()?;
                self.rpc_handle()
                    .get_activity(&pubkey, limit)
                    .await
                    .map_err(WalletError::Operation)
            }
            ChainFamily::Evm => {
                let address =
                    self.with_session(|k| k.require_evm().map(|e| e.address.clone()))??;
                taurvia_evm::activity(*desc, &address, limit)
                    .await
                    .map_err(WalletError::Operation)
            }
            ChainFamily::Bitcoin => {
                let url = self.endpoint_for(desc.id);
                let rpc = taurvia_bitcoin::BtcRpc::new(&url, *desc);
                let address = self.with_session(|k| {
                    k.require_btc(desc.is_testnet).map(|s| s.address.clone())
                })??;
                rpc.activity(&address, limit)
                    .await
                    .map_err(WalletError::Operation)
            }
            ChainFamily::Sui => Ok(Vec::new()),
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
