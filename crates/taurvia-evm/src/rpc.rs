use anyhow::{anyhow, Context, Result};
use futures::stream::{self, StreamExt};
use models::{NetworkDescriptor, SendPreview, SendResult, TokenBalance, WalletSnapshot};
use std::str::FromStr;
use std::time::Duration;

use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol;
use alloy::sol_types::SolCall;

use crate::derive::{validate_address, EvmSigner};
use crate::tokens::{curated_tokens, f64_to_u256, token_balance, u256_to_f64};

sol! {
    function balanceOf(address account) external view returns (uint256);
    function transfer(address to, uint256 amount) external returns (bool);
}

const MARKET_DATA_BUDGET: Duration = Duration::from_secs(4);
const NATIVE_MINT: &str = "eth";

pub struct EvmRpc {
    pub(crate) rpc_url: String,
    pub(crate) descriptor: NetworkDescriptor,
}

impl EvmRpc {
    pub fn new(rpc_url: &str, descriptor: NetworkDescriptor) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            descriptor,
        }
    }

    fn provider(&self) -> Result<impl Provider + Clone> {
        let url = self
            .rpc_url
            .parse()
            .map_err(|e| anyhow!("invalid evm rpc url: {e}"))?;
        Ok(ProviderBuilder::new().connect_http(url))
    }

    pub async fn snapshot(&self, address: &str) -> Result<WalletSnapshot> {
        let owner = Address::from_str(address).context("invalid stored evm address")?;
        let provider = self.provider()?;
        let native_fut = provider.get_balance(owner);
        let tokens_fut = self.token_balances_inner(provider.clone(), owner);
        let price_id = self.descriptor.coingecko_id.unwrap_or("ethereum");
        let price_fut = taurvia_chain::native_price_usd(price_id);

        let (native, market) = tokio::join!(
            native_fut,
            tokio::time::timeout(MARKET_DATA_BUDGET, async {
                tokio::join!(tokens_fut, price_fut)
            })
        );
        let native = native.context("eth_getBalance failed")?;
        let native_balance = u256_to_f64(native, 18);
        let (tokens, price) = market.unwrap_or((Ok(Vec::new()), Err(anyhow!("price timeout"))));

        let tokens = tokens.unwrap_or_default();
        let native_price_usd = price.ok();
        let native_value_usd = native_price_usd.map(|p| p * native_balance);
        let tokens_value: f64 = tokens.iter().filter_map(|t| t.value_usd).sum();

        Ok(WalletSnapshot {
            exists: true,
            unlocked: true,
            network: self.descriptor.id.to_string(),
            public_key: Some(address.to_string()),
            native_balance: Some(native_balance),
            native_symbol: self.descriptor.native_symbol.to_string(),
            native_price_usd,
            native_value_usd,
            total_portfolio_usd: Some(native_value_usd.unwrap_or(0.0) + tokens_value),
            tokens: Some(tokens),
            chains: Vec::new(),
            account_name: String::new(),
            import_kind: models::ImportKind::Mnemonic,
            enabled_networks: Vec::new(),
            can_reveal_mnemonic: false,
        })
    }

    async fn token_balances_inner(
        &self,
        provider: impl Provider + Clone,
        owner: Address,
    ) -> Result<Vec<TokenBalance>> {
        let curated = curated_tokens(self.descriptor.id);
        if curated.is_empty() {
            return Ok(Vec::new());
        }
        let contracts: Vec<String> = curated.iter().map(|t| t.address.to_string()).collect();
        let prices_fut = taurvia_chain::token_prices_usd("ethereum", &contracts);
        let balances_fut = async {
            stream::iter(curated.iter().copied())
                .map(|token| {
                    let provider = provider.clone();
                    async move {
                        let contract = Address::from_str(token.address)?;
                        let call = balanceOfCall { account: owner };
                        let tx = TransactionRequest::default()
                            .with_to(contract)
                            .with_input(call.abi_encode());
                        let bytes = provider.call(tx).await.context("balanceOf")?;
                        let raw = balanceOfCall::abi_decode_returns(&bytes)
                            .context("decode balanceOf")?;
                        Ok::<_, anyhow::Error>((token, raw))
                    }
                })
                .buffer_unordered(5)
                .collect::<Vec<_>>()
                .await
        };
        let (prices, results) = tokio::join!(prices_fut, balances_fut);
        let prices = prices.unwrap_or_default();

        let mut out = Vec::new();
        for item in results.into_iter().flatten() {
            let (token, raw) = item;
            let price = prices.get(&token.address.to_lowercase()).copied();
            if let Some(bal) = token_balance(&token, raw, price) {
                out.push(bal);
            }
        }
        Ok(out)
    }

    pub async fn preview_send(
        &self,
        from: &str,
        to: &str,
        amount: f64,
        asset: Option<&str>,
    ) -> Result<SendPreview> {
        validate_address(to)?;
        let provider = self.provider()?;
        let from_addr = Address::from_str(from).context("invalid stored evm address")?;
        let to_addr = Address::from_str(to).context("invalid recipient")?;
        let (token_symbol, tx) = self.build_tx(from_addr, to_addr, amount, asset)?;
        let gas = provider
            .estimate_gas(tx.clone())
            .await
            .context("eth_estimateGas")?;
        let fees = provider
            .estimate_eip1559_fees()
            .await
            .context("eip1559 fees")?;
        let fee_wei = U256::from(gas).saturating_mul(U256::from(fees.max_fee_per_gas));
        let estimated_fee = u256_to_f64(fee_wei, 18);
        Ok(SendPreview {
            from: from.to_string(),
            to: to.to_string(),
            token: token_symbol,
            amount: format!("{amount}"),
            network_name: self.descriptor.name.to_string(),
            estimated_fee,
            fee_symbol: self.descriptor.native_symbol.to_string(),
            creates_token_account: false,
        })
    }

    pub async fn send(
        &self,
        signer: &EvmSigner,
        to: &str,
        amount: f64,
        asset: Option<&str>,
    ) -> Result<SendResult> {
        validate_address(to)?;
        let pk = hex::encode(signer.secret_bytes());
        let local: PrivateKeySigner = pk.parse().context("evm signer")?;
        let url = self
            .rpc_url
            .parse()
            .map_err(|e| anyhow!("invalid evm rpc url: {e}"))?;
        let wallet = alloy::network::EthereumWallet::from(local);
        let provider = ProviderBuilder::new().wallet(wallet).connect_http(url);
        let from = Address::from_str(&signer.address)?;
        let to_addr = Address::from_str(to).context("invalid recipient")?;
        let (_, mut tx) = self.build_tx(from, to_addr, amount, asset)?;
        tx.set_chain_id(self.descriptor.eip155_chain_id.unwrap_or(1));
        let pending = provider
            .send_transaction(tx)
            .await
            .context("eth_sendTransaction")?;
        let hash = *pending.tx_hash();
        Ok(SendResult {
            txid: format!("{hash:#x}"),
            status: "submitted".into(),
        })
    }

    fn build_tx(
        &self,
        from: Address,
        to: Address,
        amount: f64,
        asset: Option<&str>,
    ) -> Result<(String, TransactionRequest)> {
        let native = asset
            .map(|a| a.eq_ignore_ascii_case(NATIVE_MINT) || a.eq_ignore_ascii_case("native"))
            .unwrap_or(true)
            || asset.map(|a| a.is_empty()).unwrap_or(true);
        if native || asset == Some("") {
            let wei = f64_to_u256(amount, 18);
            let tx = TransactionRequest::default()
                .with_from(from)
                .with_to(to)
                .with_value(wei);
            return Ok((self.descriptor.native_symbol.to_string(), tx));
        }
        let asset = asset.unwrap();
        let token = curated_tokens(self.descriptor.id)
            .iter()
            .find(|t| t.address.eq_ignore_ascii_case(asset))
            .ok_or_else(|| anyhow!("unknown token"))?;
        let raw = f64_to_u256(amount, token.decimals);
        let call = transferCall { to, amount: raw };
        let contract = Address::from_str(token.address)?;
        let tx = TransactionRequest::default()
            .with_from(from)
            .with_to(contract)
            .with_input(call.abi_encode());
        Ok((token.symbol.to_string(), tx))
    }
}
