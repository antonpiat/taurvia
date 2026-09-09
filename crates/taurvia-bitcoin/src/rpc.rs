use anyhow::{anyhow, bail, Context, Result};
use bitcoin::absolute::LockTime;
use bitcoin::consensus::encode::serialize_hex;
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::Message;
use bitcoin::sighash::{EcdsaSighashType, SighashCache};
use bitcoin::{
    Address, Amount, CompressedPublicKey, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut,
    Txid, Witness,
};
use models::{ActivityItem, NetworkDescriptor, SendPreview, SendResult, WalletSnapshot};
use serde::Deserialize;
use std::str::FromStr;
use std::time::Duration;

use crate::derive::{secp, validate_address, BtcSigner};

const SATS_PER_BTC: f64 = 100_000_000.0;
const MARKET_DATA_BUDGET: Duration = Duration::from_secs(4);
/// P2WPKH vbytes: ~10.5 + 68/in + 31/out
const OVERHEAD_VBYTES: f64 = 10.5;
const INPUT_VBYTES: f64 = 68.0;
const OUTPUT_VBYTES: f64 = 31.0;

#[derive(Clone, Deserialize)]
struct EsploraUtxo {
    txid: String,
    vout: u32,
    value: u64,
    status: EsploraStatus,
}

#[derive(Clone, Deserialize)]
struct EsploraStatus {
    confirmed: bool,
}

#[derive(Deserialize)]
struct EsploraTx {
    txid: String,
    status: EsploraTxStatus,
    vin: Vec<EsploraVin>,
    vout: Vec<EsploraVout>,
}

#[derive(Deserialize)]
struct EsploraTxStatus {
    block_time: Option<i64>,
    confirmed: bool,
}

#[derive(Deserialize)]
struct EsploraVin {
    prevout: Option<EsploraVout>,
}

#[derive(Deserialize)]
struct EsploraVout {
    scriptpubkey_address: Option<String>,
    value: u64,
}

pub struct BtcRpc {
    esplora: String,
    descriptor: NetworkDescriptor,
}

impl BtcRpc {
    pub fn new(esplora: &str, descriptor: NetworkDescriptor) -> Self {
        Self {
            esplora: esplora.trim_end_matches('/').to_string(),
            descriptor,
        }
    }

    pub async fn snapshot(&self, address: &str) -> Result<WalletSnapshot> {
        let price_id = self.descriptor.coingecko_id.unwrap_or("bitcoin");
        let (utxos, price) = tokio::join!(
            self.utxos(address),
            tokio::time::timeout(
                MARKET_DATA_BUDGET,
                taurvia_chain::native_price_usd(price_id),
            ),
        );
        let utxos = utxos.context("failed to fetch Bitcoin UTXOs")?;
        let sats: u64 = utxos.iter().map(|u| u.value).sum();
        let native_balance = sats as f64 / SATS_PER_BTC;
        let native_price_usd = price.ok().and_then(|r| r.ok());
        let native_value_usd = native_price_usd.map(|p| p * native_balance);
        Ok(WalletSnapshot {
            exists: true,
            unlocked: true,
            network: self.descriptor.id.to_string(),
            public_key: Some(address.to_string()),
            native_balance: Some(native_balance),
            native_symbol: self.descriptor.native_symbol.to_string(),
            native_price_usd,
            native_value_usd,
            total_portfolio_usd: native_value_usd,
            tokens: Some(Vec::new()),
            chains: Vec::new(),
            account_name: String::new(),
            import_kind: models::ImportKind::Mnemonic,
            enabled_networks: Vec::new(),
            can_reveal_mnemonic: false,
        })
    }

    pub async fn activity(&self, address: &str, limit: usize) -> Result<Vec<ActivityItem>> {
        let url = format!("{}/address/{}/txs", self.esplora, address);
        let txs: Vec<EsploraTx> = taurvia_chain::http_client()
            .get(url)
            .send()
            .await
            .context("esplora txs")?
            .error_for_status()
            .context("esplora txs HTTP")?
            .json()
            .await
            .context("esplora txs json")?;
        let me = address.to_lowercase();
        Ok(txs
            .into_iter()
            .take(limit.clamp(1, 25))
            .map(|tx| {
                let incoming: u64 = tx
                    .vout
                    .iter()
                    .filter(|o| {
                        o.scriptpubkey_address
                            .as_deref()
                            .map(|a| a.eq_ignore_ascii_case(&me))
                            .unwrap_or(false)
                    })
                    .map(|o| o.value)
                    .sum();
                let outgoing: u64 = tx
                    .vin
                    .iter()
                    .filter_map(|i| i.prevout.as_ref())
                    .filter(|o| {
                        o.scriptpubkey_address
                            .as_deref()
                            .map(|a| a.eq_ignore_ascii_case(&me))
                            .unwrap_or(false)
                    })
                    .map(|o| o.value)
                    .sum();
                let (direction, amount_sats) = if outgoing > incoming {
                    ("out", outgoing - incoming)
                } else if incoming > outgoing {
                    ("in", incoming - outgoing)
                } else {
                    ("unknown", 0)
                };
                let amount = amount_sats as f64 / SATS_PER_BTC;
                ActivityItem {
                    txid: tx.txid,
                    timestamp: tx.status.block_time,
                    status: if tx.status.confirmed {
                        "confirmed".into()
                    } else {
                        "pending".into()
                    },
                    direction: direction.into(),
                    amount: if amount_sats == 0 { None } else { Some(amount) },
                    amount_symbol: Some("BTC".into()),
                    description: if direction == "in" {
                        format!("Received {amount:.8} BTC")
                    } else if direction == "out" {
                        format!("Sent {amount:.8} BTC")
                    } else {
                        "Transaction".into()
                    },
                }
            })
            .collect())
    }

    pub async fn preview_send(&self, from: &str, to: &str, amount_btc: f64) -> Result<SendPreview> {
        let (_selected, fee_sats, _change, _send_sats) =
            self.select_coins(from, to, amount_btc, None).await?;
        Ok(SendPreview {
            from: from.to_string(),
            to: to.to_string(),
            token: "BTC".into(),
            amount: format!("{amount_btc}"),
            network_name: self.descriptor.name.to_string(),
            estimated_fee: fee_sats as f64 / SATS_PER_BTC,
            fee_symbol: "BTC".into(),
            creates_token_account: false,
        })
    }

    pub async fn send(&self, signer: &BtcSigner, to: &str, amount_btc: f64) -> Result<SendResult> {
        self.send_with_memo(signer, to, amount_btc, None).await
    }

    pub async fn send_with_memo(
        &self,
        signer: &BtcSigner,
        to: &str,
        amount_btc: f64,
        memo: Option<&str>,
    ) -> Result<SendResult> {
        let (tx, _) = self.build_signed(signer, to, amount_btc, memo).await?;
        let hex = serialize_hex(&tx);
        let url = format!("{}/tx", self.esplora);
        let txid = taurvia_chain::http_client()
            .post(url)
            .header("Content-Type", "text/plain")
            .body(hex)
            .send()
            .await
            .context("esplora broadcast")?
            .error_for_status()
            .context("esplora broadcast HTTP")?
            .text()
            .await
            .context("esplora broadcast body")?;
        Ok(SendResult {
            txid: txid.trim().to_string(),
            status: "submitted".into(),
        })
    }

    async fn utxos(&self, address: &str) -> Result<Vec<EsploraUtxo>> {
        let url = format!("{}/address/{address}/utxo", self.esplora);
        taurvia_chain::http_client()
            .get(url)
            .send()
            .await
            .context("esplora utxo")?
            .error_for_status()
            .context("esplora utxo HTTP")?
            .json()
            .await
            .context("esplora utxo json")
    }

    async fn fee_rate_sat_vb(&self) -> Result<f64> {
        let url = format!("{}/fee-estimates", self.esplora);
        let estimates: serde_json::Map<String, serde_json::Value> = taurvia_chain::http_client()
            .get(url)
            .send()
            .await
            .context("esplora fees")?
            .json()
            .await
            .unwrap_or_default();
        let rate = estimates
            .get("3")
            .or_else(|| estimates.get("2"))
            .or_else(|| estimates.get("1"))
            .and_then(|v| v.as_f64())
            .unwrap_or(5.0);
        Ok(rate.max(1.0))
    }

    async fn select_coins(
        &self,
        from: &str,
        to: &str,
        amount_btc: f64,
        memo: Option<&str>,
    ) -> Result<(Vec<EsploraUtxo>, u64, u64, u64)> {
        validate_address(to, self.descriptor.is_testnet)?;
        if amount_btc <= 0.0 {
            bail!("amount must be positive");
        }
        let send_sats = (amount_btc * SATS_PER_BTC).round() as u64;
        let (utxos, fee_rate) = tokio::join!(self.utxos(from), self.fee_rate_sat_vb());
        let mut utxos = utxos?;
        let fee_rate = fee_rate?;
        utxos.retain(|u| u.status.confirmed);
        utxos.sort_by_key(|u| std::cmp::Reverse(u.value));

        let extra_outputs = 1.0 + if memo.is_some() { 1.0 } else { 0.0 };
        let memo_vbytes = memo.map(|m| 11.0 + m.len() as f64).unwrap_or(0.0);

        let mut selected = Vec::new();
        let mut total = 0u64;
        let mut fee = 0u64;
        for utxo in utxos {
            selected.push(utxo);
            total = selected.iter().map(|u| u.value).sum();
            let vbytes = OVERHEAD_VBYTES
                + INPUT_VBYTES * selected.len() as f64
                + OUTPUT_VBYTES * extra_outputs
                + memo_vbytes;
            fee = (vbytes * fee_rate).ceil() as u64;
            if total >= send_sats.saturating_add(fee) {
                break;
            }
        }
        if total < send_sats.saturating_add(fee) {
            bail!("insufficient Bitcoin balance");
        }
        let change = total - send_sats - fee;
        Ok((selected, fee, change, send_sats))
    }

    async fn build_signed(
        &self,
        signer: &BtcSigner,
        to: &str,
        amount_btc: f64,
        memo: Option<&str>,
    ) -> Result<(Transaction, u64)> {
        let (selected, fee, change, send_sats) = self
            .select_coins(&signer.address, to, amount_btc, memo)
            .await?;

        let dest: Address = Address::from_str(to)
            .map_err(|e| anyhow!("invalid recipient: {e}"))?
            .require_network(signer.network)
            .map_err(|e| anyhow!("recipient network mismatch: {e}"))?;
        let change_addr: Address = Address::from_str(&signer.address)
            .map_err(|e| anyhow!("{e}"))?
            .assume_checked();

        let mut outputs = vec![TxOut {
            value: Amount::from_sat(send_sats),
            script_pubkey: dest.script_pubkey(),
        }];
        if change > 546 {
            outputs.push(TxOut {
                value: Amount::from_sat(change),
                script_pubkey: change_addr.script_pubkey(),
            });
        }
        if let Some(memo) = memo {
            if memo.len() > 80 {
                bail!("Thorchain memo is too long for Bitcoin OP_RETURN");
            }
            let mut bytes = bitcoin::script::PushBytesBuf::new();
            bytes
                .extend_from_slice(memo.as_bytes())
                .map_err(|_| anyhow!("invalid OP_RETURN memo"))?;
            outputs.push(TxOut {
                value: Amount::ZERO,
                script_pubkey: ScriptBuf::new_op_return(bytes),
            });
        }

        let tx = Transaction {
            version: bitcoin::transaction::Version::TWO,
            lock_time: LockTime::ZERO,
            input: selected
                .iter()
                .map(|u| {
                    let txid = Txid::from_str(&u.txid).expect("txid");
                    TxIn {
                        previous_output: OutPoint { txid, vout: u.vout },
                        script_sig: ScriptBuf::new(),
                        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                        witness: Witness::new(),
                    }
                })
                .collect(),
            output: outputs,
        };

        let secp = secp();
        let privkey = signer.private_key()?;
        let compressed =
            CompressedPublicKey::from_private_key(secp, &privkey).map_err(|e| anyhow!("{e}"))?;
        let prev_script = Address::p2wpkh(&compressed, signer.network).script_pubkey();

        let mut cache = SighashCache::new(tx);
        for (index, utxo) in selected.iter().enumerate() {
            let sighash = cache
                .p2wpkh_signature_hash(
                    index,
                    &prev_script,
                    Amount::from_sat(utxo.value),
                    EcdsaSighashType::All,
                )
                .map_err(|e| anyhow!("sighash: {e}"))?;
            let msg = Message::from_digest(sighash.to_byte_array());
            let sig = secp.sign_ecdsa(&msg, &privkey.inner);
            let mut sig_bytes = sig.serialize_der().to_vec();
            sig_bytes.push(EcdsaSighashType::All as u8);
            cache
                .witness_mut(index)
                .ok_or_else(|| anyhow!("missing witness"))?
                .push(sig_bytes);
            cache
                .witness_mut(index)
                .ok_or_else(|| anyhow!("missing witness"))?
                .push(compressed.to_bytes());
        }
        Ok((cache.into_transaction(), fee))
    }
}
