use aegis_solana::{
    derive_keypair_from_mnemonic, generate_mnemonic, keypair_from_base64, keypair_to_base64,
    lamports_to_sol, Keypair, Pubkey, Signer, SolanaRpc,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use crypto::{
    decrypt, derive_key, encrypt, generate_salt, CIPHER_NAME, KDF_NAME, KEY_LEN, NONCE_LEN,
};
use models::{
    ActivityItem, CryptoEnvelope, EncryptedPayload, Network, SendPreview, SendResult,
    TokenBalance, DEFAULT_DERIVATION_PATH, WALLET_FILE_VERSION,
};
use models::WalletFile;
use storage::WalletStorage;
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("wallet already exists")]
    AlreadyExists,
    #[error("wallet not found")]
    NotFound,
    #[error("wallet is locked")]
    Locked,
    #[error("invalid password")]
    InvalidPassword,
    #[error("invalid mnemonic")]
    InvalidMnemonic,
    #[error("storage error: {0}")]
    Storage(#[from] storage::StorageError),
    #[error("crypto error: {0}")]
    Crypto(#[from] crypto::CryptoError),
    #[error("operation failed: {0}")]
    Operation(#[from] anyhow::Error),
}

struct WalletSession {
    public_key: String,
    keypair: Keypair,
}

pub struct WalletService {
    storage: WalletStorage,
    session: Arc<Mutex<Option<WalletSession>>>,
    rpc: SolanaRpc,
}

impl WalletService {
    pub fn new(data_dir: impl AsRef<Path>, rpc_url: Option<&str>) -> Self {
        Self {
            storage: WalletStorage::new(data_dir),
            session: Arc::new(Mutex::new(None)),
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

    pub fn generate_mnemonic(&self) -> Result<String, WalletError> {
        generate_mnemonic().map_err(WalletError::Operation)
    }

    pub fn create_wallet(&self, mnemonic: &str, password: &str) -> Result<WalletFile, WalletError> {
        if self.storage.exists() {
            return Err(WalletError::AlreadyExists);
        }
        self.save_wallet_from_mnemonic(mnemonic, password)
    }

    pub fn import_wallet(&self, mnemonic: &str, password: &str) -> Result<WalletFile, WalletError> {
        if self.storage.exists() {
            return Err(WalletError::AlreadyExists);
        }
        self.save_wallet_from_mnemonic(mnemonic, password)
    }

    pub fn unlock(&self, password: &str) -> Result<String, WalletError> {
        let wallet = self.storage.load()?;
        let payload = self.decrypt_payload(&wallet, password)?;
        let keypair = keypair_from_base64(&payload.private_key).map_err(WalletError::Operation)?;
        let public_key = keypair.pubkey().to_string();
        let mut session = self.session.lock().unwrap();
        *session = Some(WalletSession {
            public_key: public_key.clone(),
            keypair,
        });
        Ok(public_key)
    }

    pub fn lock(&self) {
        let mut session = self.session.lock().unwrap();
        *session = None;
    }

    pub fn reveal_mnemonic(&self, password: &str) -> Result<String, WalletError> {
        let wallet = self.storage.load()?;
        let payload = self.decrypt_payload(&wallet, password)?;
        Ok(payload.mnemonic)
    }

    pub fn get_sol_balance(&self) -> Result<f64, WalletError> {
        let pubkey = self.require_pubkey()?;
        let lamports = self.rpc.get_balance(&pubkey).map_err(WalletError::Operation)?;
        Ok(lamports_to_sol(lamports))
    }

    pub fn get_token_balances(&self) -> Result<Vec<TokenBalance>, WalletError> {
        let pubkey = self.require_pubkey()?;
        self.rpc
            .get_token_balances(&pubkey)
            .map_err(WalletError::Operation)
    }

    pub fn get_activity(&self, limit: usize) -> Result<Vec<ActivityItem>, WalletError> {
        let pubkey = self.require_pubkey()?;
        self.rpc
            .get_activity(&pubkey, limit)
            .map_err(WalletError::Operation)
    }

    pub fn preview_sol_send(&self, to: &str, amount_sol: f64) -> Result<SendPreview, WalletError> {
        let keypair = self.signing_keypair()?;
        self.rpc
            .preview_sol_send(&keypair, to, amount_sol)
            .map_err(WalletError::Operation)
    }

    pub fn preview_spl_send(
        &self,
        mint: &str,
        to: &str,
        amount: f64,
    ) -> Result<SendPreview, WalletError> {
        let keypair = self.signing_keypair()?;
        self.rpc
            .preview_spl_send(&keypair, mint, to, amount)
            .map_err(WalletError::Operation)
    }

    pub fn send_sol(
        &self,
        password: &str,
        to: &str,
        amount_sol: f64,
    ) -> Result<SendResult, WalletError> {
        self.verify_password(password)?;
        let keypair = self.signing_keypair()?;
        self.rpc
            .send_sol(&keypair, to, amount_sol)
            .map_err(WalletError::Operation)
    }

    pub fn send_spl(
        &self,
        password: &str,
        mint: &str,
        to: &str,
        amount: f64,
    ) -> Result<SendResult, WalletError> {
        self.verify_password(password)?;
        let keypair = self.signing_keypair()?;
        self.rpc
            .send_spl(&keypair, mint, to, amount)
            .map_err(WalletError::Operation)
    }

    fn save_wallet_from_mnemonic(
        &self,
        mnemonic: &str,
        password: &str,
    ) -> Result<WalletFile, WalletError> {
        let keypair = derive_keypair_from_mnemonic(mnemonic).map_err(|_| WalletError::InvalidMnemonic)?;
        let payload = EncryptedPayload {
            mnemonic: mnemonic.to_string(),
            private_key: keypair_to_base64(&keypair),
            derivation_path: DEFAULT_DERIVATION_PATH.to_string(),
        };
        let wallet = self.encrypt_wallet_file(&keypair, &payload, password)?;
        self.storage.save(&wallet)?;
        Ok(wallet)
    }

    fn encrypt_wallet_file(
        &self,
        keypair: &Keypair,
        payload: &EncryptedPayload,
        password: &str,
    ) -> Result<WalletFile, WalletError> {
        let salt = generate_salt();
        let derived = derive_key(password, &salt)?;
        let plaintext = serde_json::to_vec(payload).map_err(|e| {
            WalletError::Operation(anyhow::anyhow!("failed to serialize payload: {e}"))
        })?;
        let (nonce, ciphertext) = encrypt(&plaintext, derived.as_bytes())?;

        Ok(models::WalletFile {
            version: WALLET_FILE_VERSION,
            wallet_id: Uuid::new_v4().to_string(),
            network: Network::SolanaMainnet.as_str().to_string(),
            public_key: keypair.pubkey().to_string(),
            created_at: Utc::now().to_rfc3339(),
            crypto: CryptoEnvelope {
                kdf: KDF_NAME.into(),
                salt: BASE64.encode(salt),
                cipher: CIPHER_NAME.into(),
                nonce: BASE64.encode(nonce),
                ciphertext: BASE64.encode(ciphertext),
            },
        })
    }

    fn decrypt_payload(
        &self,
        wallet: &WalletFile,
        password: &str,
    ) -> Result<EncryptedPayload, WalletError> {
        let salt = BASE64
            .decode(&wallet.crypto.salt)
            .map_err(|_| WalletError::InvalidPassword)?;
        let nonce = BASE64
            .decode(&wallet.crypto.nonce)
            .map_err(|_| WalletError::InvalidPassword)?;
        let ciphertext = BASE64
            .decode(&wallet.crypto.ciphertext)
            .map_err(|_| WalletError::InvalidPassword)?;

        if nonce.len() != NONCE_LEN {
            return Err(WalletError::InvalidPassword);
        }

        let derived = derive_key(password, &salt)?;
        let key: [u8; KEY_LEN] = *derived.as_bytes();
        let plaintext = decrypt(&nonce, &ciphertext, &key)?;
        let payload: EncryptedPayload = serde_json::from_slice(&plaintext).map_err(|_| {
            WalletError::InvalidPassword
        })?;
        Ok(payload)
    }

    fn verify_password(&self, password: &str) -> Result<(), WalletError> {
        let wallet = self.storage.load()?;
        self.decrypt_payload(&wallet, password)?;
        Ok(())
    }

    fn signing_keypair(&self) -> Result<Keypair, WalletError> {
        let session = self.session.lock().unwrap();
        let session = session.as_ref().ok_or(WalletError::Locked)?;
        Keypair::try_from(session.keypair.to_bytes().as_slice()).map_err(|_| WalletError::Locked)
    }

    fn require_pubkey(&self) -> Result<Pubkey, WalletError> {
        let session = self.session.lock().unwrap();
        let session = session.as_ref().ok_or(WalletError::Locked)?;
        Ok(session.keypair.pubkey())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_import_unlock_flow() {
        let dir = tempfile::tempdir().unwrap();
        let service = WalletService::new(dir.path(), Some("http://localhost:8899"));
        let mnemonic = service.generate_mnemonic().unwrap();
        service.create_wallet(&mnemonic, "password123").unwrap();
        assert!(service.wallet_exists());
        let pubkey = service.unlock("password123").unwrap();
        assert!(!pubkey.is_empty());
        let revealed = service.reveal_mnemonic("password123").unwrap();
        assert_eq!(revealed, mnemonic);
        service.lock();
        assert!(!service.is_unlocked());
    }
}
