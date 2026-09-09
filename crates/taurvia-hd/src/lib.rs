//! BIP39 mnemonic generate / validate / seed. Chain crates own path + address encoding.
//! Seed bytes never cross Tauri IPC.

use anyhow::{Context, Result};
use bip39::Mnemonic;
use zeroize::Zeroizing;

pub const SEED_LEN: usize = 64;

pub fn generate_mnemonic() -> Result<String> {
    let mut entropy = [0u8; 16];
    rand::fill(&mut entropy);
    let mnemonic = Mnemonic::from_entropy(&entropy).context("failed to generate mnemonic")?;
    Ok(mnemonic.to_string())
}

pub fn validate_mnemonic(mnemonic: &str) -> Result<()> {
    Mnemonic::parse(mnemonic)
        .map(|_| ())
        .map_err(|_| anyhow::anyhow!("invalid mnemonic phrase"))
}

/// BIP39 seed (PBKDF2, empty passphrase). Caller must drop / zeroize.
pub fn seed_from_mnemonic(mnemonic: &str) -> Result<Zeroizing<[u8; SEED_LEN]>> {
    let mnemonic =
        Mnemonic::parse(mnemonic).map_err(|_| anyhow::anyhow!("invalid mnemonic phrase"))?;
    let seed = mnemonic.to_seed("");
    Ok(Zeroizing::new(seed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_and_validate() {
        let phrase = generate_mnemonic().unwrap();
        validate_mnemonic(&phrase).unwrap();
        assert_eq!(seed_from_mnemonic(&phrase).unwrap().len(), SEED_LEN);
    }

    #[test]
    fn reject_garbage() {
        assert!(validate_mnemonic("not a phrase").is_err());
    }
}
