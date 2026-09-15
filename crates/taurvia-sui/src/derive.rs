use anyhow::{anyhow, bail, Result};
use bech32::{FromBase32, ToBase32, Variant};
use blake2::{digest::consts::U32, Blake2b, Digest};
use ed25519_dalek::{Signer as _, SigningKey, VerifyingKey};
use hmac::{Hmac, Mac};
use sha2::Sha512;
use zeroize::Zeroizing;

type HmacSha512 = Hmac<Sha512>;
type Blake2b256 = Blake2b<U32>;

/// Sui Wallet / Suiet default Ed25519 path (all hardened, SLIP-0010).
const SUI_DERIVATION_PATH: [u32; 5] = [
    44 | 0x8000_0000,
    784 | 0x8000_0000,
    0x8000_0000,
    0x8000_0000,
    0x8000_0000,
];

const ED25519_FLAG: u8 = 0x00;

#[derive(Clone)]
pub struct SuiSigner {
    pub address: String,
    secret: Zeroizing<[u8; 32]>,
}

impl SuiSigner {
    pub fn secret_bytes(&self) -> &[u8; 32] {
        &self.secret
    }

    pub fn public_key_bytes(&self) -> [u8; 32] {
        VerifyingKey::from(&SigningKey::from_bytes(&self.secret)).to_bytes()
    }

    pub fn sign_intent_digest(&self, digest: &[u8; 32]) -> [u8; 64] {
        let key = SigningKey::from_bytes(&self.secret);
        key.sign(digest).to_bytes()
    }

    pub fn to_suiprivkey(&self) -> Result<String> {
        encode_suiprivkey(&self.secret)
    }
}

pub fn derive_from_seed(seed: &[u8]) -> Result<SuiSigner> {
    let (mut key, mut chain) = slip10_master(seed)?;
    for index in SUI_DERIVATION_PATH {
        let (next_key, next_chain) = slip10_child(&key, &chain, index)?;
        key = next_key;
        chain = next_chain;
    }
    from_secret_bytes(key)
}

pub fn from_secret(input: &str) -> Result<SuiSigner> {
    let trimmed = input.trim();
    if trimmed.to_ascii_lowercase().starts_with("suiprivkey1") {
        return from_suiprivkey(trimmed);
    }
    let hex = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("invalid Sui private key (expected suiprivkey1… or 64 hex characters)");
    }
    let bytes = hex::decode(hex).map_err(|e| anyhow!("invalid Sui private key: {e}"))?;
    let mut secret = [0u8; 32];
    secret.copy_from_slice(&bytes);
    from_secret_bytes(secret)
}

fn from_suiprivkey(encoded: &str) -> Result<SuiSigner> {
    let (hrp, data, variant) =
        bech32::decode(encoded).map_err(|e| anyhow!("invalid suiprivkey: {e}"))?;
    if hrp != "suiprivkey" || variant != Variant::Bech32 {
        bail!("invalid suiprivkey encoding");
    }
    let bytes = Vec::<u8>::from_base32(&data).map_err(|e| anyhow!("invalid suiprivkey: {e}"))?;
    if bytes.len() != 33 {
        bail!("invalid suiprivkey length");
    }
    if bytes[0] != ED25519_FLAG {
        bail!("only Ed25519 Sui keys are supported");
    }
    let mut secret = [0u8; 32];
    secret.copy_from_slice(&bytes[1..]);
    from_secret_bytes(secret)
}

fn encode_suiprivkey(secret: &[u8; 32]) -> Result<String> {
    let mut payload = Vec::with_capacity(33);
    payload.push(ED25519_FLAG);
    payload.extend_from_slice(secret);
    bech32::encode("suiprivkey", payload.to_base32(), Variant::Bech32)
        .map_err(|e| anyhow!("encode suiprivkey: {e}"))
}

fn from_secret_bytes(secret: [u8; 32]) -> Result<SuiSigner> {
    let address = address_from_secret(&secret);
    Ok(SuiSigner {
        address,
        secret: Zeroizing::new(secret),
    })
}

pub fn validate_address(address: &str) -> Result<()> {
    if !is_sui_address(address) {
        bail!("invalid Sui address");
    }
    Ok(())
}

pub fn is_sui_address(address: &str) -> bool {
    let rest = address
        .strip_prefix("0x")
        .or_else(|| address.strip_prefix("0X"));
    match rest {
        Some(hex) => hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()),
        None => false,
    }
}

fn address_from_secret(secret: &[u8; 32]) -> String {
    let verifying = VerifyingKey::from(&SigningKey::from_bytes(secret));
    address_from_pubkey(&verifying.to_bytes())
}

fn address_from_pubkey(pubkey: &[u8; 32]) -> String {
    let mut hasher = Blake2b256::new();
    hasher.update([ED25519_FLAG]);
    hasher.update(pubkey);
    format!("0x{}", hex::encode(hasher.finalize()))
}

pub fn intent_digest(tx_bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Blake2b256::new();
    // Intent: [scope=TransactionData, version=V0, app=Sui]
    hasher.update([0u8, 0u8, 0u8]);
    hasher.update(tx_bytes);
    hasher.finalize().into()
}

fn slip10_master(seed: &[u8]) -> Result<([u8; 32], [u8; 32])> {
    let mut mac =
        HmacSha512::new_from_slice(b"ed25519 seed").map_err(|e| anyhow!("slip10 hmac: {e}"))?;
    mac.update(seed);
    Ok(split_i(hmac_bytes(mac)))
}

fn slip10_child(key: &[u8; 32], chain: &[u8; 32], index: u32) -> Result<([u8; 32], [u8; 32])> {
    let mut data = [0u8; 37];
    data[0] = 0x00;
    data[1..33].copy_from_slice(key);
    data[33..37].copy_from_slice(&index.to_be_bytes());
    let mut mac = HmacSha512::new_from_slice(chain).map_err(|e| anyhow!("slip10 hmac: {e}"))?;
    mac.update(&data);
    Ok(split_i(hmac_bytes(mac)))
}

fn hmac_bytes(mac: HmacSha512) -> [u8; 64] {
    let result = mac.finalize().into_bytes();
    let mut i = [0u8; 64];
    i.copy_from_slice(&result);
    i
}

fn split_i(i: [u8; 64]) -> ([u8; 32], [u8; 32]) {
    let mut key = [0u8; 32];
    let mut chain = [0u8; 32];
    key.copy_from_slice(&i[..32]);
    chain.copy_from_slice(&i[32..]);
    (key, chain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use taurvia_hd::seed_from_mnemonic;

    #[test]
    fn slip10_abandon_sui_wallet() {
        let seed = seed_from_mnemonic(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )
        .unwrap();
        let signer = derive_from_seed(seed.as_slice()).unwrap();
        assert_eq!(
            hex::encode(signer.secret_bytes()),
            "8869cb07178bf67e08d7c4abdf45487dbf379c9a452fcec2836854bf4a3d29b0"
        );
        assert_eq!(
            signer.address,
            "0x5e93a736d04fbb25737aa40bee40171ef79f65fae833749e3c089fe7cc2161f1"
        );
        let encoded = signer.to_suiprivkey().unwrap();
        let round = from_secret(&encoded).unwrap();
        assert_eq!(round.address, signer.address);
    }

    #[test]
    fn rejects_evm_length_address() {
        assert!(validate_address("0x9858effd232b4033e47d90003d41ec34ecaeda94").is_err());
    }
}
