use models::{ChainFamily, SwapQuote, SwapResult, TokenInfo};
use taurvia_solana::{normalize_mint, resolve_mint, search_tokens, ui_amount_to_raw};

use crate::session::WalletService;
use crate::WalletError;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AssetFamily {
    Solana,
    Evm,
    Bitcoin,
}

impl WalletService {
    fn classify_asset(asset: &str) -> Result<AssetFamily, WalletError> {
        let a = asset.trim();
        if taurvia_bitcoin::is_btc(a) {
            return Ok(AssetFamily::Bitcoin);
        }
        if taurvia_bitcoin::is_eth_native(a)
            || (a.starts_with("0x") && a.len() == 42)
            || (a.starts_with("0X") && a.len() == 42)
        {
            return Ok(AssetFamily::Evm);
        }
        Ok(AssetFamily::Solana)
    }

    fn require_swap_backend(&self, family: AssetFamily) -> Result<(), WalletError> {
        let kind = self.import_kind();
        if let Some(owned) = kind.family() {
            let needed = match family {
                AssetFamily::Solana => ChainFamily::Solana,
                AssetFamily::Evm => ChainFamily::Evm,
                AssetFamily::Bitcoin => ChainFamily::Bitcoin,
            };
            if owned != needed {
                return Err(WalletError::Operation(anyhow::anyhow!(
                    "this wallet cannot swap on that chain"
                )));
            }
        }
        let mainnet = match family {
            AssetFamily::Solana => models::NETWORK_SOLANA_MAINNET,
            AssetFamily::Evm => models::NETWORK_ETHEREUM_MAINNET,
            AssetFamily::Bitcoin => models::NETWORK_BITCOIN_MAINNET,
        };
        let enabled = self.enabled_network_ids();
        if !enabled
            .iter()
            .any(|id| models::normalize_network_id(id) == mainnet)
        {
            return Err(WalletError::Operation(anyhow::anyhow!(
                "activate {} mainnet to swap",
                match family {
                    AssetFamily::Solana => "Solana",
                    AssetFamily::Evm => "Ethereum",
                    AssetFamily::Bitcoin => "Bitcoin",
                }
            )));
        }
        let desc = models::require_network(mainnet);
        if desc.is_testnet || !desc.features.swap {
            return Err(WalletError::Operation(anyhow::anyhow!(
                "Swap is available on mainnet only"
            )));
        }
        Ok(())
    }

    pub async fn resolve_token(&self, mint: &str) -> Result<TokenInfo, WalletError> {
        let family = Self::classify_asset(mint)?;
        self.require_swap_backend(family)?;
        match family {
            AssetFamily::Solana => {
                let mint = normalize_mint(mint).map_err(WalletError::Operation)?;
                resolve_mint(&mint).await.map_err(WalletError::Operation)
            }
            AssetFamily::Evm => {
                if taurvia_bitcoin::is_eth_native(mint) {
                    return Ok(TokenInfo {
                        mint: "eth".into(),
                        symbol: "ETH".into(),
                        name: "Ethereum".into(),
                        decimals: 18,
                        logo_uri: None,
                    });
                }
                taurvia_evm::resolve_curated(models::NETWORK_ETHEREUM_MAINNET, mint)
                    .map(taurvia_evm::token_info)
                    .ok_or_else(|| {
                        WalletError::Operation(anyhow::anyhow!("unknown Ethereum token"))
                    })
            }
            AssetFamily::Bitcoin => Ok(TokenInfo {
                mint: "btc".into(),
                symbol: "BTC".into(),
                name: "Bitcoin".into(),
                decimals: 8,
                logo_uri: None,
            }),
        }
    }

    pub async fn search_tokens(&self, query: &str) -> Result<Vec<TokenInfo>, WalletError> {
        let q = query.trim();
        if q.is_empty() {
            return Ok(Vec::new());
        }
        let family = models::require_network(&self.wallet_network()).family;
        match family {
            models::ChainFamily::Solana => {
                self.require_swap_backend(AssetFamily::Solana)?;
                search_tokens(q).await.map_err(WalletError::Operation)
            }
            models::ChainFamily::Evm => {
                self.require_swap_backend(AssetFamily::Evm)?;
                let q = q.to_ascii_lowercase();
                Ok(
                    taurvia_evm::curated_tokens(models::NETWORK_ETHEREUM_MAINNET)
                        .iter()
                        .filter(|t| {
                            t.symbol.to_ascii_lowercase().contains(&q)
                                || t.name.to_ascii_lowercase().contains(&q)
                                || t.address.to_ascii_lowercase().contains(&q)
                        })
                        .map(taurvia_evm::token_info)
                        .collect(),
                )
            }
            models::ChainFamily::Bitcoin => Ok(Vec::new()),
            models::ChainFamily::Sui => Err(WalletError::Operation(anyhow::anyhow!(
                "token search is not available on this chain"
            ))),
        }
    }

    pub async fn preview_swap(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount_ui: f64,
        slippage_bps: u16,
    ) -> Result<SwapQuote, WalletError> {
        self.dispatch_quote(input_mint, output_mint, amount_ui, slippage_bps)
            .await
    }

    pub async fn execute_swap(
        &self,
        password: &str,
        input_mint: &str,
        output_mint: &str,
        amount_ui: f64,
        slippage_bps: u16,
    ) -> Result<SwapResult, WalletError> {
        self.verify_password(password)?;
        self.dispatch_execute(input_mint, output_mint, amount_ui, slippage_bps)
            .await
    }

    async fn dispatch_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount_ui: f64,
        slippage_bps: u16,
    ) -> Result<SwapQuote, WalletError> {
        let from = Self::classify_asset(input_mint)?;
        let to = Self::classify_asset(output_mint)?;
        self.require_swap_backend(from)?;
        match (from, to) {
            (AssetFamily::Solana, AssetFamily::Solana) => {
                self.jupiter_quote(input_mint, output_mint, amount_ui, slippage_bps)
                    .await
            }
            (AssetFamily::Evm, AssetFamily::Evm) => {
                self.zerox_quote(input_mint, output_mint, amount_ui, slippage_bps)
                    .await
            }
            (AssetFamily::Bitcoin, AssetFamily::Evm)
            | (AssetFamily::Bitcoin, AssetFamily::Solana)
            | (AssetFamily::Evm, AssetFamily::Bitcoin)
            | (AssetFamily::Solana, AssetFamily::Bitcoin) => {
                self.thorchain_quote(input_mint, output_mint, amount_ui, slippage_bps)
                    .await
            }
            (AssetFamily::Solana, AssetFamily::Evm) | (AssetFamily::Evm, AssetFamily::Solana) => {
                Err(WalletError::Operation(anyhow::anyhow!(
                    "same-chain swaps only for Solana (Jupiter) and Ethereum (0x); Thorchain is used when Bitcoin is one side"
                )))
            }
            (AssetFamily::Bitcoin, AssetFamily::Bitcoin) => Err(WalletError::Operation(
                anyhow::anyhow!("Bitcoin has no same-chain swap"),
            )),
        }
    }

    async fn dispatch_execute(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount_ui: f64,
        slippage_bps: u16,
    ) -> Result<SwapResult, WalletError> {
        let from = Self::classify_asset(input_mint)?;
        let to = Self::classify_asset(output_mint)?;
        self.require_swap_backend(from)?;
        match (from, to) {
            (AssetFamily::Solana, AssetFamily::Solana) => {
                self.jupiter_execute(input_mint, output_mint, amount_ui, slippage_bps)
                    .await
            }
            (AssetFamily::Evm, AssetFamily::Evm) => {
                self.zerox_execute(input_mint, output_mint, amount_ui, slippage_bps)
                    .await
            }
            (AssetFamily::Bitcoin, _) => {
                self.thorchain_execute_btc(input_mint, output_mint, amount_ui, slippage_bps)
                    .await
            }
            _ => Err(WalletError::Operation(anyhow::anyhow!(
                "this pair is quote-only in this version; swap Bitcoin as the source"
            ))),
        }
    }

    async fn jupiter_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount_ui: f64,
        slippage_bps: u16,
    ) -> Result<SwapQuote, WalletError> {
        let _ = self.require_pubkey()?;
        let input_mint = normalize_mint(input_mint).map_err(WalletError::Operation)?;
        let output_mint = normalize_mint(output_mint).map_err(WalletError::Operation)?;
        let amount_raw = ui_amount_to_raw(&input_mint, amount_ui)
            .await
            .map_err(WalletError::Operation)?;
        self.solana_mainnet_rpc()
            .quote_swap(&input_mint, &output_mint, amount_raw, slippage_bps)
            .await
            .map_err(WalletError::Operation)
    }

    async fn jupiter_execute(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount_ui: f64,
        slippage_bps: u16,
    ) -> Result<SwapResult, WalletError> {
        let keypair = self.signing_keypair()?;
        let input_mint = normalize_mint(input_mint).map_err(WalletError::Operation)?;
        let output_mint = normalize_mint(output_mint).map_err(WalletError::Operation)?;
        let amount_raw = ui_amount_to_raw(&input_mint, amount_ui)
            .await
            .map_err(WalletError::Operation)?;
        self.solana_mainnet_rpc()
            .execute_swap(
                &keypair,
                &input_mint,
                &output_mint,
                amount_raw,
                slippage_bps,
            )
            .await
            .map_err(WalletError::Operation)
    }

    fn solana_mainnet_rpc(&self) -> taurvia_solana::SolanaRpc {
        taurvia_solana::SolanaRpc::new(Some(&self.endpoint_for(models::NETWORK_SOLANA_MAINNET)))
    }

    fn zerox_api_key(&self) -> Option<String> {
        let from_settings = self
            .get_settings()
            .zerox_api_key
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        from_settings.or_else(|| {
            std::env::var("TAURVIA_0X_API_KEY")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
    }

    async fn zerox_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount_ui: f64,
        slippage_bps: u16,
    ) -> Result<SwapQuote, WalletError> {
        let url = self.endpoint_for(models::NETWORK_ETHEREUM_MAINNET);
        let desc = *models::require_network(models::NETWORK_ETHEREUM_MAINNET);
        let rpc = taurvia_evm::EvmRpc::new(&url, desc);
        let taker = self.with_session(|k| k.require_evm().map(|e| e.address.clone()))??;
        let key = self.zerox_api_key();
        rpc.quote_swap(
            &taker,
            input_mint,
            output_mint,
            amount_ui,
            slippage_bps,
            key.as_deref(),
        )
        .await
        .map_err(WalletError::Operation)
    }

    async fn zerox_execute(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount_ui: f64,
        slippage_bps: u16,
    ) -> Result<SwapResult, WalletError> {
        let url = self.endpoint_for(models::NETWORK_ETHEREUM_MAINNET);
        let desc = *models::require_network(models::NETWORK_ETHEREUM_MAINNET);
        let rpc = taurvia_evm::EvmRpc::new(&url, desc);
        let signer = self.with_session(|k| k.require_evm().cloned())??;
        let key = self.zerox_api_key();
        rpc.execute_swap(
            &signer,
            input_mint,
            output_mint,
            amount_ui,
            slippage_bps,
            key.as_deref(),
        )
        .await
        .map_err(WalletError::Operation)
    }

    async fn thorchain_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount_ui: f64,
        slippage_bps: u16,
    ) -> Result<SwapQuote, WalletError> {
        let from_asset = taurvia_bitcoin::thor_asset(input_mint).map_err(WalletError::Operation)?;
        let to_asset = taurvia_bitcoin::thor_asset(output_mint).map_err(WalletError::Operation)?;
        let dest = self.thorchain_destination(output_mint)?;
        let amount_1e8 = (amount_ui * 1e8).round() as u64;
        let quote =
            taurvia_bitcoin::thorchain_quote(from_asset, to_asset, amount_1e8, &dest, slippage_bps)
                .await
                .map_err(WalletError::Operation)?;
        let in_sym = if taurvia_bitcoin::is_btc(input_mint) {
            "BTC"
        } else if taurvia_bitcoin::is_eth_native(input_mint) {
            "ETH"
        } else {
            "SOL"
        };
        let out_sym = if taurvia_bitcoin::is_btc(output_mint) {
            "BTC"
        } else if taurvia_bitcoin::is_eth_native(output_mint) {
            "ETH"
        } else {
            "SOL"
        };
        taurvia_bitcoin::quote_to_swap(
            input_mint,
            output_mint,
            in_sym,
            out_sym,
            amount_ui,
            slippage_bps,
            &quote,
        )
        .map_err(WalletError::Operation)
    }

    async fn thorchain_execute_btc(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount_ui: f64,
        slippage_bps: u16,
    ) -> Result<SwapResult, WalletError> {
        if !taurvia_bitcoin::is_btc(input_mint) {
            return Err(WalletError::Operation(anyhow::anyhow!(
                "only Bitcoin → other Thorchain swaps are signed in this version"
            )));
        }
        let dest = self.thorchain_destination(output_mint)?;
        let from_asset = taurvia_bitcoin::thor_asset(input_mint).map_err(WalletError::Operation)?;
        let to_asset = taurvia_bitcoin::thor_asset(output_mint).map_err(WalletError::Operation)?;
        let amount_1e8 = (amount_ui * 1e8).round() as u64;
        let quote =
            taurvia_bitcoin::thorchain_quote(from_asset, to_asset, amount_1e8, &dest, slippage_bps)
                .await
                .map_err(WalletError::Operation)?;
        let url = self.endpoint_for(models::NETWORK_BITCOIN_MAINNET);
        let desc = *models::require_network(models::NETWORK_BITCOIN_MAINNET);
        let rpc = taurvia_bitcoin::BtcRpc::new(&url, desc);
        let signer = self.with_session(|k| k.require_btc(false).cloned())??;
        let result = rpc
            .send_with_memo(
                &signer,
                &quote.inbound_address,
                amount_ui,
                Some(&quote.memo),
            )
            .await
            .map_err(WalletError::Operation)?;
        Ok(SwapResult {
            signature: result.txid,
            status: result.status,
        })
    }

    fn thorchain_destination(&self, output_mint: &str) -> Result<String, WalletError> {
        if taurvia_bitcoin::is_eth_native(output_mint) {
            return self.with_session(|k| k.require_evm().map(|e| e.address.clone()))?;
        }
        if taurvia_bitcoin::is_sol_native(output_mint) {
            return Ok(self.require_pubkey()?.to_string());
        }
        if taurvia_bitcoin::is_btc(output_mint) {
            return self.with_session(|k| k.require_btc(false).map(|s| s.address.clone()))?;
        }
        Err(WalletError::Operation(anyhow::anyhow!(
            "unsupported Thorchain destination"
        )))
    }
}
