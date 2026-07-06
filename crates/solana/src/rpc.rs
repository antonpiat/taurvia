use crate::transfer::{build_sol_transfer, build_spl_transfer};
use anyhow::{anyhow, Context, Result};
use base64::Engine;
use models::{ActivityItem, SendPreview, SendResult, TokenBalance};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcSendTransactionConfig, RpcTransactionConfig};
use solana_commitment_config::CommitmentConfig;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
use solana_program_pack::Pack;
use solana_transaction_status_client_types::UiTransactionEncoding;
use spl_token_interface::state::Account as TokenAccount;
use std::str::FromStr;
use std::time::Duration;

const DEFAULT_RPC_URL: &str = "https://api.mainnet-beta.solana.com";
const DEFAULT_HISTORY_LIMIT: usize = 20;

pub struct SolanaRpc {
    client: RpcClient,
}

impl SolanaRpc {
    pub fn new(rpc_url: Option<&str>) -> Self {
        let url = rpc_url
            .filter(|u| !u.is_empty())
            .unwrap_or(DEFAULT_RPC_URL)
            .to_string();
        let client = RpcClient::new_with_timeout_and_commitment(
            url,
            Duration::from_secs(30),
            CommitmentConfig::confirmed(),
        );
        Self { client }
    }

    pub fn from_env() -> Self {
        Self::new(std::env::var("AEGIS_RPC_URL").ok().as_deref())
    }

    pub fn get_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        self.client
            .get_balance(pubkey)
            .context("failed to fetch SOL balance")
    }

    pub fn get_token_balances(&self, owner: &Pubkey) -> Result<Vec<TokenBalance>> {
        let accounts = self
            .client
            .get_token_accounts_by_owner(
                owner,
                solana_client::rpc_request::TokenAccountsFilter::ProgramId(spl_token_interface::id()),
            )
            .context("failed to fetch token accounts")?;

        let mut balances = Vec::new();
        for account in accounts {
            let data = match account.account.data {
                solana_account_decoder::UiAccountData::Binary(data, _) => base64::engine::general_purpose::STANDARD
                    .decode(data)
                    .context("failed to decode token account data")?,
                _ => continue,
            };

            let token_account = TokenAccount::unpack(&data)
                .context("failed to unpack token account")?;
            if token_account.amount == 0 {
                continue;
            }

            let mint = token_account.mint.to_string();
            let decimals = self
                .client
                .get_token_supply(&token_account.mint)
                .map(|s| s.decimals)
                .unwrap_or(0);
            let ui_amount = token_account.amount as f64 / 10f64.powi(decimals as i32);

            balances.push(TokenBalance {
                mint: mint.clone(),
                symbol: shorten_mint(&mint),
                name: format!("SPL {}", shorten_mint(&mint)),
                amount: token_account.amount.to_string(),
                decimals,
                ui_amount,
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
            .client
            .get_token_supply(&mint_pubkey)
            .map(|s| s.decimals)
            .unwrap_or(0);
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
            .client
            .get_token_supply(&mint_pubkey)
            .map(|s| s.decimals)
            .unwrap_or(0);
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

        let mut items = Vec::new();
        for sig_info in signatures {
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
                if let Ok(tx) = self.client.get_transaction_with_config(
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
                        if let Some(idx) = find_account_index(&tx, owner) {
                            if idx < pre_balances.len() && idx < post_balances.len() {
                                let delta =
                                    post_balances[idx] as i64 - pre_balances[idx] as i64;
                                if delta != 0 {
                                    amount_sol = Some(crate::lamports_to_sol(delta.unsigned_abs()));
                                    direction = if delta < 0 {
                                        "out".into()
                                    } else {
                                        "in".into()
                                    };
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

            items.push(ActivityItem {
                signature,
                timestamp,
                status,
                direction,
                amount_sol,
                description,
            });
        }

        Ok(items)
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
                &self.client.get_latest_blockhash()?,
                CommitmentConfig::confirmed(),
            )
            .context("failed to confirm transaction")?;

        Ok(SendResult {
            signature: signature.to_string(),
            status: "confirmed".into(),
        })
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
    tx: &solana_transaction_status_client_types::EncodedConfirmedTransactionWithStatusMeta,
    owner: &Pubkey,
) -> Option<usize> {
    use solana_transaction_status_client_types::{EncodedTransaction, UiMessage};
    match &tx.transaction.transaction {
        EncodedTransaction::Json(ui_tx) => match &ui_tx.message {
            UiMessage::Parsed(parsed) => parsed
                .account_keys
                .iter()
                .position(|key| key.pubkey == owner.to_string()),
            UiMessage::Raw(raw) => raw
                .account_keys
                .iter()
                .position(|key| key == &owner.to_string()),
        },
        _ => None,
    }
}
