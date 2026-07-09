use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use bip39::Mnemonic;
use models::DEFAULT_DERIVATION_PATH;
use solana_derivation_path::DerivationPath;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::keypair::keypair_from_seed_and_derivation_path;

pub fn generate_mnemonic() -> Result<String> {
    let mut entropy = [0u8; 16];
    rand::fill(&mut entropy);
    let mnemonic = Mnemonic::from_entropy(&entropy).context("failed to generate mnemonic")?;
    Ok(mnemonic.to_string())
}

pub fn derive_keypair_from_mnemonic(mnemonic: &str) -> Result<Keypair> {
    let mnemonic =
        Mnemonic::parse(mnemonic).map_err(|_| anyhow!("invalid mnemonic phrase"))?;
    let seed = mnemonic.to_seed("");
    let path = DerivationPath::from_absolute_path_str(DEFAULT_DERIVATION_PATH)
        .map_err(|e| anyhow!("invalid derivation path: {e}"))?;
    keypair_from_seed_and_derivation_path(&seed, Some(path))
        .map_err(|e| anyhow!("key derivation failed: {e}"))
}

pub fn keypair_to_base64(keypair: &Keypair) -> String {
    BASE64.encode(keypair.to_bytes())
}

pub fn keypair_from_base64(encoded: &str) -> Result<Keypair> {
    let bytes = BASE64
        .decode(encoded)
        .context("invalid private key encoding")?;
    Keypair::try_from(bytes.as_slice()).map_err(|e| anyhow!("invalid keypair: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signer::Signer;

    #[test]
    fn mnemonic_round_trip() {
        let phrase = generate_mnemonic().unwrap();
        let kp1 = derive_keypair_from_mnemonic(&phrase).unwrap();
        let kp2 = derive_keypair_from_mnemonic(&phrase).unwrap();
        assert_eq!(kp1.pubkey(), kp2.pubkey());
    }

    #[test]
    fn keypair_base64_round_trip() {
        let phrase = generate_mnemonic().unwrap();
        let kp = derive_keypair_from_mnemonic(&phrase).unwrap();
        let encoded = keypair_to_base64(&kp);
        let restored = keypair_from_base64(&encoded).unwrap();
        assert_eq!(kp.pubkey(), restored.pubkey());
    }
}
