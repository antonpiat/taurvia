use crate::transfer::{build_sol_transfer, build_spl_transfer};
use anyhow::{anyhow, Context, Result};
use base64::Engine;
use models::{ActivityItem, SendPreview, SendResult, TokenBalance};
use rayon::prelude::*;
use solana_account_decoder::UiAccountData;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcSendTransactionConfig, RpcTransactionConfig};
use solana_commitment_config::CommitmentConfig;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_program_pack::Pack;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
use solana_transaction_status_client_types::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiMessage,
    UiTransactionEncoding,
};
use spl_token_interface::state::{Account as TokenAccount, Mint};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

const DEFAULT_RPC_URL: &str = "https://api.mainnet-beta.solana.com";
const DEFAULT_HISTORY_LIMIT: usize = 20;
const RPC_TIMEOUT: Duration = Duration::from_secs(30);

pub struct SolanaRpc {
    rpc_url: String,
    client: RpcClient,
}

impl SolanaRpc {
    pub fn new(rpc_url: Option<&str>) -> Self {
        let url = rpc_url
            .filter(|u| !u.is_empty())
            .unwrap_or(DEFAULT_RPC_URL)
            .to_string();
        let client = Self::client_for_url(&url);
        Self {
            rpc_url: url,
            client,
        }
    }

    pub fn from_env() -> Self {
        Self::new(std::env::var("AEGIS_RPC_URL").ok().as_deref())
    }

    pub fn get_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        self.client
            .get_balance(pubkey)
            .context("failed to fetch SOL balance")
    }

    pub fn get_balances_parallel(&self, owner: &Pubkey) -> Result<(u64, Vec<TokenBalance>)> {
        let url = self.rpc_url.clone();
        let owner = *owner;
        std::thread::scope(|scope| {
            let sol_handle = scope.spawn(|| Self::client_for_url(&url).get_balance(&owner));
            let tokens_url = url.clone();
            let tokens_handle = scope.spawn(move || {
                Self::new(Some(&tokens_url)).get_token_balances(&owner)
            });
            let sol = sol_handle.join().unwrap()?;
            let tokens = tokens_handle.join().unwrap()?;
            Ok((sol, tokens))
        })
    }

    pub fn get_token_balances(&self, owner: &Pubkey) -> Result<Vec<TokenBalance>> {
        let accounts = self
            .client
            .get_token_accounts_by_owner(
                owner,
                solana_client::rpc_request::TokenAccountsFilter::ProgramId(spl_token_interface::id()),
            )
            .context("failed to fetch token accounts")?;

        let mut holdings = Vec::new();
        let mut unique_mints = Vec::new();
        let mut seen_mints = HashMap::new();

        for account in accounts {
            let data = match account.account.data {
                UiAccountData::Binary(data, _) => base64::engine::general_purpose::STANDARD
                    .decode(data)
                    .context("failed to decode token account data")?,
                _ => continue,
            };

            let token_account =
                TokenAccount::unpack(&data).context("failed to unpack token account")?;
            if token_account.amount == 0 {
                continue;
            }

            if !seen_mints.contains_key(&token_account.mint) {
                seen_mints.insert(token_account.mint, ());
                unique_mints.push(token_account.mint);
            }

            holdings.push((token_account.mint, token_account.amount));
        }

        let decimals_by_mint = self.fetch_mint_decimals(&unique_mints)?;

        let mut balances = Vec::with_capacity(holdings.len());
        for (mint, amount) in holdings {
            let mint_str = mint.to_string();
            let decimals = decimals_by_mint.get(&mint).copied().unwrap_or(0);
            let short = shorten_mint(&mint_str);
            balances.push(TokenBalance {
                mint: mint_str,
                symbol: short.clone(),
                name: format!("SPL {short}"),
                amount: amount.to_string(),
                decimals,
                ui_amount: amount as f64 / 10f64.powi(decimals as i32),
            });
        }

        Ok(balances)
    }

    pub fn preview_sol_send(
        &self,
        from: &Keypair,
        to: &str,
        amount_sol: f64,
    ) -> Result<SendPreview> {
        let to_pubkey = Pubkey::from_str(to).context("invalid recipient address")?;
        let lamports = sol_to_lamports(amount_sol)?;
        let blockhash = self.client.get_latest_blockhash()?;
        let tx = build_sol_transfer(from, &to_pubkey, lamports, blockhash)?;
        let fee = self.estimate_fee(&tx)?;
        Ok(SendPreview {
            from: from.pubkey().to_string(),
            to: to_pubkey.to_string(),
            token: "SOL".into(),
            amount: format!("{amount_sol} SOL"),
            estimated_fee_lamports: fee,
            estimated_fee_sol: crate::lamports_to_sol(fee),
        })
    }

    pub fn preview_spl_send(
        &self,
        from: &Keypair,
        mint: &str,
        to: &str,
        amount: f64,
    ) -> Result<SendPreview> {
        let mint_pubkey = Pubkey::from_str(mint).context("invalid mint address")?;
        let to_pubkey = Pubkey::from_str(to).context("invalid recipient address")?;
        let decimals = self
            .mint_decimals(&mint_pubkey)
            .context("failed to fetch mint decimals")?;
        let raw_amount = (amount * 10f64.powi(decimals as i32)).round() as u64;
        let blockhash = self.client.get_latest_blockhash()?;
        let tx = build_spl_transfer(from, &mint_pubkey, &to_pubkey, raw_amount, blockhash)?;
        let fee = self.estimate_fee(&tx)?;
        Ok(SendPreview {
            from: from.pubkey().to_string(),
            to: to_pubkey.to_string(),
            token: shorten_mint(mint),
            amount: format!("{amount}"),
            estimated_fee_lamports: fee,
            estimated_fee_sol: crate::lamports_to_sol(fee),
        })
    }

    pub fn send_sol(&self, from: &Keypair, to: &str, amount_sol: f64) -> Result<SendResult> {
        let to_pubkey = Pubkey::from_str(to).context("invalid recipient address")?;
        let lamports = sol_to_lamports(amount_sol)?;
        let blockhash = self.client.get_latest_blockhash()?;
        let tx = build_sol_transfer(from, &to_pubkey, lamports, blockhash)?;
        self.send_and_confirm(tx)
    }

    pub fn send_spl(
        &self,
        from: &Keypair,
        mint: &str,
        to: &str,
        amount: f64,
    ) -> Result<SendResult> {
        let mint_pubkey = Pubkey::from_str(mint).context("invalid mint address")?;
        let to_pubkey = Pubkey::from_str(to).context("invalid recipient address")?;
        let decimals = self
            .mint_decimals(&mint_pubkey)
            .context("failed to fetch mint decimals")?;
        let raw_amount = (amount * 10f64.powi(decimals as i32)).round() as u64;
        let blockhash = self.client.get_latest_blockhash()?;
        let tx = build_spl_transfer(from, &mint_pubkey, &to_pubkey, raw_amount, blockhash)?;
        self.send_and_confirm(tx)
    }

    pub fn get_activity(&self, owner: &Pubkey, limit: usize) -> Result<Vec<ActivityItem>> {
        let signatures = self
            .client
            .get_signatures_for_address_with_config(
                owner,
                solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config {
                    limit: Some(limit.min(DEFAULT_HISTORY_LIMIT)),
                    ..Default::default()
                },
            )
            .context("failed to fetch transaction signatures")?;

        let rpc_url = self.rpc_url.clone();
        let owner_str = owner.to_string();

        Ok(signatures
            .into_par_iter()
            .map(|sig_info| {
                build_activity_item(&Self::client_for_url(&rpc_url), sig_info, &owner_str)
            })
            .collect())
    }

    fn client_for_url(url: &str) -> RpcClient {
        RpcClient::new_with_timeout_and_commitment(url, RPC_TIMEOUT, CommitmentConfig::confirmed())
    }

    fn mint_decimals(&self, mint: &Pubkey) -> Result<u8> {
        self.fetch_mint_decimals(&[*mint])
            .map(|map| map.get(mint).copied().unwrap_or(0))
    }

    fn fetch_mint_decimals(&self, mints: &[Pubkey]) -> Result<HashMap<Pubkey, u8>> {
        if mints.is_empty() {
            return Ok(HashMap::new());
        }

        let accounts = self
            .client
            .get_multiple_accounts(mints)
            .context("failed to fetch mint accounts")?;

        let mut decimals_by_mint = HashMap::with_capacity(mints.len());
        for (mint, account) in mints.iter().zip(accounts) {
            let Some(account) = account else {
                continue;
            };
            if let Ok(mint_state) = Mint::unpack(&account.data) {
                decimals_by_mint.insert(*mint, mint_state.decimals);
            }
        }
        Ok(decimals_by_mint)
    }

    fn estimate_fee(&self, tx: &Transaction) -> Result<u64> {
        let simulation = self
            .client
            .simulate_transaction(tx)
            .context("failed to simulate transaction")?;
        if let Some(err) = simulation.value.err {
            return Err(anyhow!("simulation failed: {err:?}"));
        }
        Ok(simulation.value.units_consumed.unwrap_or(5000) * 5000 / 1_000_000 + 5000)
    }

    fn send_and_confirm(&self, tx: Transaction) -> Result<SendResult> {
        let blockhash = tx.message.recent_blockhash;
        let signature = self
            .client
            .send_transaction_with_config(
                &tx,
                RpcSendTransactionConfig {
                    skip_preflight: false,
                    ..Default::default()
                },
            )
            .context("failed to send transaction")?;

        self.client
            .confirm_transaction_with_spinner(
                &signature,
                &blockhash,
                CommitmentConfig::confirmed(),
            )
            .context("failed to confirm transaction")?;

        Ok(SendResult {
            signature: signature.to_string(),
            status: "confirmed".into(),
        })
    }
}

fn build_activity_item(
    client: &RpcClient,
    sig_info: solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature,
    owner_str: &str,
) -> ActivityItem {
    let signature = sig_info.signature;
    let status = if sig_info.err.is_some() {
        "failed"
    } else {
        "confirmed"
    }
    .to_string();

    let timestamp = sig_info.block_time;
    let mut amount_sol = None;
    let mut description = "Transaction".to_string();
    let mut direction = "unknown".to_string();

    if let Ok(sig) = Signature::from_str(&signature) {
        if let Ok(tx) = client.get_transaction_with_config(
            &sig,
            RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::JsonParsed),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            },
        ) {
            if let Some(ref meta) = tx.transaction.meta {
                let pre_balances = &meta.pre_balances;
                let post_balances = &meta.post_balances;
                if let Some(idx) = find_account_index(&tx, owner_str) {
                    if idx < pre_balances.len() && idx < post_balances.len() {
                        let delta = post_balances[idx] as i64 - pre_balances[idx] as i64;
                        if delta != 0 {
                            amount_sol = Some(crate::lamports_to_sol(delta.unsigned_abs()));
                            direction = if delta < 0 { "out".into() } else { "in".into() };
                            description = if delta < 0 {
                                format!(
                                    "Sent {:.6} SOL",
                                    crate::lamports_to_sol(delta.unsigned_abs())
                                )
                            } else {
                                format!(
                                    "Received {:.6} SOL",
                                    crate::lamports_to_sol(delta.unsigned_abs())
                                )
                            };
                        }
                    }
                }
            }
        }
    }

    ActivityItem {
        signature,
        timestamp,
        status,
        direction,
        amount_sol,
        description,
    }
}

fn sol_to_lamports(amount_sol: f64) -> Result<u64> {
    if amount_sol <= 0.0 {
        return Err(anyhow!("amount must be positive"));
    }
    let lamports = (amount_sol * LAMPORTS_PER_SOL as f64).round() as u64;
    if lamports == 0 {
        return Err(anyhow!("amount too small"));
    }
    Ok(lamports)
}

fn shorten_mint(mint: &str) -> String {
    if mint.len() <= 10 {
        mint.to_string()
    } else {
        format!("{}...{}", &mint[..4], &mint[mint.len() - 4..])
    }
}

fn find_account_index(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
    owner: &str,
) -> Option<usize> {
    match &tx.transaction.transaction {
        EncodedTransaction::Json(ui_tx) => match &ui_tx.message {
            UiMessage::Parsed(parsed) => parsed
                .account_keys
                .iter()
                .position(|key| key.pubkey == owner),
            UiMessage::Raw(raw) => raw.account_keys.iter().position(|key| key == owner),
        },
        _ => None,
    }
}
