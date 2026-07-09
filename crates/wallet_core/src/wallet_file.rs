use aegis_solana::{
    derive_keypair_from_mnemonic, generate_mnemonic, keypair_from_base64, keypair_to_base64,
    Keypair, Signer,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use crypto::{
    decrypt, derive_key, encrypt, generate_salt, CIPHER_NAME, KDF_NAME, KEY_LEN, NONCE_LEN,
};
use models::{
    CryptoEnvelope, EncryptedPayload, Network, WalletFile, DEFAULT_DERIVATION_PATH,
    WALLET_FILE_VERSION,
};
use uuid::Uuid;

use crate::session::{WalletService, WalletSession};
use crate::WalletError;

impl WalletService {
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
        *self.cached_wallet.lock().unwrap() = Some(wallet);
        Ok(public_key)
    }

    pub fn reveal_mnemonic(&self, password: &str) -> Result<String, WalletError> {
        let wallet = self.storage.load()?;
        let payload = self.decrypt_payload(&wallet, password)?;
        Ok(payload.mnemonic)
    }

    fn save_wallet_from_mnemonic(
        &self,
        mnemonic: &str,
        password: &str,
    ) -> Result<WalletFile, WalletError> {
        let keypair =
            derive_keypair_from_mnemonic(mnemonic).map_err(|_| WalletError::InvalidMnemonic)?;
        let payload = EncryptedPayload {
            mnemonic: mnemonic.to_string(),
            private_key: keypair_to_base64(&keypair),
            derivation_path: DEFAULT_DERIVATION_PATH.to_string(),
        };
        let wallet = self.encrypt_wallet_file(&keypair, &payload, password)?;
        self.storage.save(&wallet)?;
        *self.cached_wallet.lock().unwrap() = Some(wallet.clone());
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

        Ok(WalletFile {
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

    pub(crate) fn decrypt_payload(
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
        let payload: EncryptedPayload =
            serde_json::from_slice(&plaintext).map_err(|_| WalletError::InvalidPassword)?;
        Ok(payload)
    }

    pub(crate) fn verify_password(&self, password: &str) -> Result<(), WalletError> {
        let wallet = self
            .cached_wallet
            .lock()
            .unwrap()
            .clone()
            .or_else(|| self.storage.load().ok())
            .ok_or(WalletError::NotFound)?;
        self.decrypt_payload(&wallet, password)?;
        Ok(())
    }
}
