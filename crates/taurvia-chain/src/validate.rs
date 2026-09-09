use anyhow::{bail, Result};
use models::ChainFamily;

/// Reject obvious cross-family addresses in Rust (UI copy is not enough).
pub fn validate_recipient(family: ChainFamily, address: &str) -> Result<()> {
    let address = address.trim();
    if address.is_empty() {
        bail!("recipient address is required");
    }

    let looks_evm = is_evm_hex(address);
    let looks_btc = is_bitcoin_bech32(address);
    let looks_solana = !looks_evm && !looks_btc && looks_base58(address);

    match family {
        ChainFamily::Solana => {
            if looks_evm {
                bail!(
                    "this looks like an Ethereum address; switch network or paste a Solana address"
                );
            }
            if looks_btc {
                bail!(
                    "this looks like a Bitcoin address; switch network or paste a Solana address"
                );
            }
            if !looks_solana {
                bail!("invalid Solana address");
            }
            Ok(())
        }
        ChainFamily::Evm => {
            if looks_btc {
                bail!("this looks like a Bitcoin address; switch network or paste an Ethereum address");
            }
            if !looks_evm {
                bail!("invalid Ethereum address");
            }
            Ok(())
        }
        ChainFamily::Bitcoin => {
            if looks_evm {
                bail!("this looks like an Ethereum address; switch network or paste a Bitcoin address");
            }
            if !looks_btc {
                bail!("invalid Bitcoin address (Native SegWit bc1/tb1 required)");
            }
            Ok(())
        }
        ChainFamily::Sui => {
            if looks_btc {
                bail!("this looks like a Bitcoin address; switch network or paste a Sui address");
            }
            if !looks_evm {
                bail!("invalid Sui address");
            }
            Ok(())
        }
    }
}

fn is_evm_hex(address: &str) -> bool {
    let rest = address
        .strip_prefix("0x")
        .or_else(|| address.strip_prefix("0X"));
    match rest {
        Some(hex) => hex.len() == 40 && hex.chars().all(|c| c.is_ascii_hexdigit()),
        None => false,
    }
}

fn is_bitcoin_bech32(address: &str) -> bool {
    let lower = address.to_ascii_lowercase();
    (lower.starts_with("bc1") || lower.starts_with("tb1"))
        && lower.len() >= 14
        && lower.chars().all(|c| matches!(c, 'a'..='z' | '0'..='9'))
}

fn looks_base58(address: &str) -> bool {
    let len = address.len();
    (32..=44).contains(&len)
        && address
            .chars()
            .all(|c| c.is_ascii_alphanumeric() && !matches!(c, '0' | 'O' | 'I' | 'l'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solana_rejects_evm() {
        let err = validate_recipient(
            ChainFamily::Solana,
            "0x9858EfFD960e39d77Fbc6d5ebb9d8881e807bf3f",
        )
        .unwrap_err();
        assert!(err.to_string().contains("Ethereum"));
    }

    #[test]
    fn evm_rejects_bitcoin() {
        let err = validate_recipient(
            ChainFamily::Evm,
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
        )
        .unwrap_err();
        assert!(err.to_string().contains("Bitcoin"));
    }

    #[test]
    fn bitcoin_accepts_bc1() {
        validate_recipient(
            ChainFamily::Bitcoin,
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
        )
        .unwrap();
    }
}
