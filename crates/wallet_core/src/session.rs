use aegis_solana::{Keypair, Pubkey, Signer, SolanaRpc};
use models::WalletFile;
use storage::WalletStorage;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::WalletError;

pub(crate) struct WalletSession {
    pub public_key: String,
    pub keypair: Keypair,
}

pub struct WalletService {
    pub(crate) storage: WalletStorage,
    pub(crate) session: Arc<Mutex<Option<WalletSession>>>,
    pub(crate) cached_wallet: Mutex<Option<WalletFile>>,
    pub(crate) rpc: SolanaRpc,
}

impl WalletService {
    pub fn new(data_dir: impl AsRef<Path>, rpc_url: Option<&str>) -> Self {
        Self {
            storage: WalletStorage::new(data_dir),
            session: Arc::new(Mutex::new(None)),
            cached_wallet: Mutex::new(None),
            rpc: SolanaRpc::new(rpc_url),
        }
    }

    pub fn wallet_exists(&self) -> bool {
        self.storage.exists()
    }

    pub fn is_unlocked(&self) -> bool {
        self.session.lock().unwrap().is_some()
    }

    pub fn get_public_key(&self) -> Option<String> {
        if let Some(session) = self.session.lock().unwrap().as_ref() {
            return Some(session.public_key.clone());
        }
        self.storage.load().ok().map(|w| w.public_key)
    }

    pub fn lock(&self) {
        let mut session = self.session.lock().unwrap();
        *session = None;
        *self.cached_wallet.lock().unwrap() = None;
    }

    pub(crate) fn signing_keypair(&self) -> Result<Keypair, WalletError> {
        let session = self.session.lock().unwrap();
        let session = session.as_ref().ok_or(WalletError::Locked)?;
        Keypair::try_from(session.keypair.to_bytes().as_slice()).map_err(|_| WalletError::Locked)
    }

    pub(crate) fn require_pubkey(&self) -> Result<Pubkey, WalletError> {
        let session = self.session.lock().unwrap();
        let session = session.as_ref().ok_or(WalletError::Locked)?;
        Ok(session.keypair.pubkey())
    }
}
