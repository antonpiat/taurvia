use anyhow::{anyhow, bail, Result};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, CompressedPublicKey, Network, PrivateKey};
use std::str::FromStr;
use std::sync::OnceLock;
use zeroize::Zeroizing;

const BITCOIN_MAINNET_PATH: &str = "m/84'/0'/0'/0/0";
const BITCOIN_TESTNET_PATH: &str = "m/84'/1'/0'/0/0";

pub(crate) fn secp() -> &'static Secp256k1<bitcoin::secp256k1::All> {
    static SECP: OnceLock<Secp256k1<bitcoin::secp256k1::All>> = OnceLock::new();
    SECP.get_or_init(Secp256k1::new)
}

#[derive(Clone)]
pub struct BtcSigner {
    pub address: String,
    pub network: Network,
    secret: Zeroizing<[u8; 32]>,
}

impl BtcSigner {
    pub fn private_key(&self) -> Result<PrivateKey> {
        let sk = bitcoin::secp256k1::SecretKey::from_slice(self.secret.as_slice())
            .map_err(|e| anyhow!("invalid bitcoin key: {e}"))?;
        Ok(PrivateKey::new(sk, self.network))
    }

    pub fn from_secret(secret: [u8; 32], testnet: bool) -> Result<Self> {
        signer_from_secret(secret, testnet)
    }
}

pub fn from_wif(input: &str) -> Result<(BtcSigner, BtcSigner)> {
    let pk = PrivateKey::from_wif(input.trim()).map_err(|e| anyhow!("invalid Bitcoin WIF: {e}"))?;
    let mut secret = [0u8; 32];
    secret.copy_from_slice(&pk.inner.secret_bytes());
    let mainnet = signer_from_secret(secret, false)?;
    let testnet = signer_from_secret(secret, true)?;
    Ok((mainnet, testnet))
}

fn signer_from_secret(secret: [u8; 32], testnet: bool) -> Result<BtcSigner> {
    let network = if testnet {
        Network::Testnet
    } else {
        Network::Bitcoin
    };
    let secp = secp();
    let privkey = {
        let sk = bitcoin::secp256k1::SecretKey::from_slice(secret.as_slice())
            .map_err(|e| anyhow!("invalid bitcoin key: {e}"))?;
        PrivateKey::new(sk, network)
    };
    let compressed = CompressedPublicKey::from_private_key(secp, &privkey)
        .map_err(|e| anyhow!("bitcoin pubkey: {e}"))?;
    let address = Address::p2wpkh(&compressed, network);
    Ok(BtcSigner {
        address: address.to_string(),
        network,
        secret: Zeroizing::new(secret),
    })
}

pub fn derive_from_seed(seed: &[u8], testnet: bool) -> Result<BtcSigner> {
    let path_str = if testnet {
        BITCOIN_TESTNET_PATH
    } else {
        BITCOIN_MAINNET_PATH
    };
    let path: bip32::DerivationPath = path_str
        .parse()
        .map_err(|e| anyhow!("invalid bitcoin path: {e}"))?;
    let xprv = bip32::XPrv::derive_from_path(seed, &path)
        .map_err(|e| anyhow!("bitcoin derivation: {e}"))?;
    let mut secret = Zeroizing::new([0u8; 32]);
    secret.copy_from_slice(xprv.private_key().to_bytes().as_slice());
    signer_from_secret(*secret, testnet)
}

pub fn validate_address(address: &str, testnet: bool) -> Result<()> {
    let expected = if testnet {
        Network::Testnet
    } else {
        Network::Bitcoin
    };
    let parsed = Address::from_str(address).map_err(|e| anyhow!("invalid Bitcoin address: {e}"))?;
    let checked = parsed
        .require_network(expected)
        .map_err(|e| anyhow!("invalid Bitcoin address: {e}"))?;
    let s = checked.to_string().to_ascii_lowercase();
    let prefix = if testnet { "tb1q" } else { "bc1q" };
    if !s.starts_with(prefix) {
        bail!(
            "invalid Bitcoin address (Native SegWit {}… required)",
            if testnet { "tb1q" } else { "bc1q" }
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use taurvia_hd::seed_from_mnemonic;

    #[test]
    fn bip84_abandon_mainnet() {
        let seed = seed_from_mnemonic(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )
        .unwrap();
        let signer = derive_from_seed(seed.as_slice(), false).unwrap();
        let pk = signer.private_key().unwrap();
        let compressed = bitcoin::CompressedPublicKey::from_private_key(secp(), &pk).unwrap();
        assert_eq!(
            format!("{compressed}"),
            "0330d54fd0dd420a6e5f8d3624f5f3482cae350f79d5f0753bf5beef9c2d91af3c"
        );
        assert!(signer.address.starts_with("bc1q"));
        assert_eq!(
            signer.address,
            Address::p2wpkh(&compressed, Network::Bitcoin).to_string()
        );
    }
}
