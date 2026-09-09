use anyhow::{anyhow, bail, Result};
use bip32::{DerivationPath, XPrv};
use k256::elliptic_curve::sec1::ToSec1Point;
use sha3::{Digest, Keccak256};
use zeroize::Zeroizing;

const EVM_DERIVATION_PATH: &str = "m/44'/60'/0'/0/0";

#[derive(Clone)]
pub struct EvmSigner {
    pub address: String,
    secret: Zeroizing<[u8; 32]>,
}

impl EvmSigner {
    pub fn secret_bytes(&self) -> &[u8; 32] {
        &self.secret
    }
}

pub fn derive_from_seed(seed: &[u8]) -> Result<EvmSigner> {
    let path: DerivationPath = EVM_DERIVATION_PATH
        .parse()
        .map_err(|e| anyhow!("invalid evm derivation path: {e}"))?;
    let xprv =
        XPrv::derive_from_path(seed, &path).map_err(|e| anyhow!("evm derivation failed: {e}"))?;
    let secret_key = xprv.private_key();
    let mut secret = Zeroizing::new([0u8; 32]);
    secret.copy_from_slice(secret_key.to_bytes().as_slice());
    from_secret(*secret)
}

pub fn from_hex(input: &str) -> Result<EvmSigner> {
    let trimmed = input.trim();
    let hex = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("invalid Ethereum private key (expected 0x + 64 hex characters)");
    }
    let bytes = hex::decode(hex).map_err(|e| anyhow!("invalid Ethereum private key: {e}"))?;
    let mut secret = [0u8; 32];
    secret.copy_from_slice(&bytes);
    from_secret(secret)
}

fn from_secret(secret: [u8; 32]) -> Result<EvmSigner> {
    let address = address_from_secret(&secret)?;
    Ok(EvmSigner {
        address,
        secret: Zeroizing::new(secret),
    })
}

pub fn validate_address(address: &str) -> Result<()> {
    let rest = address
        .strip_prefix("0x")
        .or_else(|| address.strip_prefix("0X"))
        .ok_or_else(|| anyhow!("invalid Ethereum address"))?;
    if rest.len() != 40 || !rest.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("invalid Ethereum address");
    }
    Ok(())
}

fn address_from_secret(secret: &[u8; 32]) -> Result<String> {
    let sk = k256::SecretKey::from_slice(secret.as_slice())
        .map_err(|e| anyhow!("invalid secp256k1 key: {e}"))?;
    let pk = sk.public_key();
    let uncompressed = pk.to_sec1_point(false);
    let bytes = uncompressed.as_bytes();
    // skip 0x04 prefix
    let hash = Keccak256::digest(&bytes[1..]);
    let addr = &hash[12..];
    Ok(eip55(addr))
}

fn eip55(addr: &[u8]) -> String {
    let hex = hex::encode(addr);
    let hash = Keccak256::digest(hex.as_bytes());
    let mut out = String::from("0x");
    for (i, ch) in hex.chars().enumerate() {
        let hash_byte = hash[i / 2];
        let nibble = if i % 2 == 0 {
            hash_byte >> 4
        } else {
            hash_byte & 0x0f
        };
        if ch.is_ascii_hexdigit() && ch.is_ascii_alphabetic() && nibble >= 8 {
            out.push(ch.to_ascii_uppercase());
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use taurvia_hd::seed_from_mnemonic;

    #[test]
    fn metamask_vector_abandon() {
        let seed = seed_from_mnemonic(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )
        .unwrap();
        let signer = derive_from_seed(seed.as_slice()).unwrap();
        assert_eq!(
            signer.address.to_lowercase(),
            "0x9858effd232b4033e47d90003d41ec34ecaeda94"
        );
    }
}
