use taurvia_solana::{
    derive_keypair_from_mnemonic, generate_mnemonic, keypair_from_base64, keypair_to_base64,
    validate_mnemonic, Keypair, Signer,
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

    pub fn validate_mnemonic(&self, mnemonic: &str) -> Result<(), WalletError> {
        validate_mnemonic(mnemonic).map_err(|_| WalletError::InvalidMnemonic)
    }

    pub fn create_wallet(&self, mnemonic: &str, password: &str) -> Result<WalletFile, WalletError> {
        if self.storage.exists() {
            return Err(WalletError::AlreadyExists);
        }
        Self::require_password_strength(password)?;
        self.save_wallet_from_mnemonic(mnemonic, password)
    }

    pub fn import_wallet(&self, mnemonic: &str, password: &str) -> Result<WalletFile, WalletError> {
        if self.storage.exists() {
            return Err(WalletError::AlreadyExists);
        }
        Self::require_password_strength(password)?;
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
            mnemonic: payload.mnemonic.clone(),
        });
        *self.cached_wallet.lock().unwrap() = Some(wallet.clone());

        // Keep settings.network aligned with the wallet file.
        let network = Network::parse(&wallet.network);
        let mut settings = self.get_settings();
        if settings.network != network {
            settings.network = network;
            let _ = self.update_settings(settings);
        }

        Ok(public_key)
    }

    /// Returns the recovery phrase for the unlocked session. Requires unlock (no password re-prompt).
    pub fn reveal_mnemonic(&self) -> Result<String, WalletError> {
        let session = self.session.lock().unwrap();
        let session = session.as_ref().ok_or(WalletError::Locked)?;
        Ok(session.mnemonic.clone())
    }

    pub fn remove_wallet(&self, password: &str) -> Result<(), WalletError> {
        self.verify_password(password)?;
        self.lock();
        self.storage.delete()?;
        Ok(())
    }

    pub fn change_password(&self, old_password: &str, new_password: &str) -> Result<(), WalletError> {
        Self::require_password_strength(new_password)?;
        let wallet = self
            .cached_wallet
            .lock()
            .unwrap()
            .clone()
            .or_else(|| self.storage.load().ok())
            .ok_or(WalletError::NotFound)?;
        let payload = self.decrypt_payload(&wallet, old_password)?;
        let keypair =
            keypair_from_base64(&payload.private_key).map_err(WalletError::Operation)?;
        let updated = self.reencrypt_wallet_file(&wallet, &keypair, &payload, new_password)?;
        self.storage.save(&updated)?;
        *self.cached_wallet.lock().unwrap() = Some(updated);
        Ok(())
    }

    pub fn export_wallet(&self, password: &str) -> Result<String, WalletError> {
        let wallet = self
            .cached_wallet
            .lock()
            .unwrap()
            .clone()
            .or_else(|| self.storage.load().ok())
            .ok_or(WalletError::NotFound)?;
        self.decrypt_payload(&wallet, password)?;
        serde_json::to_string_pretty(&wallet).map_err(|e| {
            WalletError::Operation(anyhow::anyhow!("failed to serialize wallet file: {e}"))
        })
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
        let network = self.get_settings().network;
        let wallet = self.encrypt_wallet_file(&keypair, &payload, password, network)?;
        self.storage.save(&wallet)?;
        *self.cached_wallet.lock().unwrap() = Some(wallet.clone());
        Ok(wallet)
    }

    /// Phantom-style: 8+ chars with upper, lower, digit, and special character.
    fn require_password_strength(password: &str) -> Result<(), WalletError> {
        let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
        let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_ascii_alphanumeric());
        if password.len() < 8 || !has_upper || !has_lower || !has_digit || !has_special {
            return Err(WalletError::Operation(anyhow::anyhow!(
                "password must contain at least 8 characters including 1 uppercase letter, 1 lowercase letter, 1 number, and 1 special character"
            )));
        }
        Ok(())
    }

    fn encrypt_wallet_file(
        &self,
        keypair: &Keypair,
        payload: &EncryptedPayload,
        password: &str,
        network: Network,
    ) -> Result<WalletFile, WalletError> {
        self.reencrypt_wallet_file(
            &WalletFile {
                version: WALLET_FILE_VERSION,
                wallet_id: Uuid::new_v4().to_string(),
                network: network.as_str().to_string(),
                public_key: keypair.pubkey().to_string(),
                created_at: Utc::now().to_rfc3339(),
                crypto: CryptoEnvelope {
                    kdf: KDF_NAME.into(),
                    salt: String::new(),
                    cipher: CIPHER_NAME.into(),
                    nonce: String::new(),
                    ciphertext: String::new(),
                },
            },
            keypair,
            payload,
            password,
        )
    }

    fn reencrypt_wallet_file(
        &self,
        existing: &WalletFile,
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
            wallet_id: existing.wallet_id.clone(),
            network: existing.network.clone(),
            public_key: keypair.pubkey().to_string(),
            created_at: existing.created_at.clone(),
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
        if wallet.version != WALLET_FILE_VERSION {
            return Err(WalletError::Operation(anyhow::anyhow!(
                "unsupported wallet file version: {}",
                wallet.version
            )));
        }
        if wallet.crypto.kdf != KDF_NAME || wallet.crypto.cipher != CIPHER_NAME {
            return Err(WalletError::Operation(anyhow::anyhow!(
                "unsupported wallet encryption algorithm"
            )));
        }

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
