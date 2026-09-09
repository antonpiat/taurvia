use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use crypto::{
    decrypt, derive_wallet_key, encrypt, generate_device_secret, generate_salt, CIPHER_NAME,
    KDF_NAME, KEY_LEN, NONCE_LEN,
};
use models::{
    CryptoEnvelope, EncryptedPayload, ImportKind, WalletFile, WalletProtection,
    DEFAULT_ACCOUNT_NAME, DEFAULT_DERIVATION_PATH, MIN_WALLET_FILE_VERSION, WALLET_FILE_VERSION,
};
use storage::DeviceSecretError;
use taurvia_solana::{keypair_from_base64, keypair_from_secret_input, keypair_to_base64};
use uuid::Uuid;
use zeroize::Zeroize;

use crate::session::{FamilyKeyring, WalletService, WalletSession};
use crate::WalletError;

impl WalletService {
    pub fn generate_mnemonic(&self) -> Result<String, WalletError> {
        taurvia_hd::generate_mnemonic().map_err(WalletError::Operation)
    }

    pub fn validate_mnemonic(&self, mnemonic: &str) -> Result<(), WalletError> {
        taurvia_hd::validate_mnemonic(mnemonic).map_err(|_| WalletError::InvalidMnemonic)
    }

    pub fn create_wallet(
        &self,
        mnemonic: &str,
        password: &str,
        account_name: &str,
    ) -> Result<WalletFile, WalletError> {
        if self.storage.exists() {
            return Err(WalletError::AlreadyExists);
        }
        Self::require_password_strength(password)?;
        self.save_wallet_from_mnemonic(mnemonic, password, account_name)
    }

    pub fn import_wallet(
        &self,
        mnemonic: &str,
        password: &str,
        account_name: &str,
    ) -> Result<WalletFile, WalletError> {
        if self.storage.exists() {
            return Err(WalletError::AlreadyExists);
        }
        Self::require_password_strength(password)?;
        self.save_wallet_from_mnemonic(mnemonic, password, account_name)
    }

    pub fn import_private_key(
        &self,
        secret: &str,
        password: &str,
        account_name: &str,
    ) -> Result<WalletFile, WalletError> {
        if self.storage.exists() {
            return Err(WalletError::AlreadyExists);
        }
        Self::require_password_strength(password)?;
        let trimmed = secret.trim();
        if trimmed.is_empty() {
            return Err(WalletError::Operation(anyhow::anyhow!(
                "private key is required"
            )));
        }

        let (keyring, kind, stored_secret) = detect_and_parse_key(trimmed)?;
        let payload = EncryptedPayload {
            mnemonic: String::new(),
            private_key: stored_secret,
            derivation_path: DEFAULT_DERIVATION_PATH.to_string(),
        };
        let network =
            models::mainnet_id_for_family(kind.family().unwrap_or(models::ChainFamily::Solana))
                .to_string();
        let wallet = self.encrypt_wallet_file(
            &keyring,
            &payload,
            password,
            &network,
            account_name,
            kind,
            kind.default_enabled_networks(),
        )?;
        self.storage.save(&wallet)?;
        *self.cached_wallet.lock().unwrap() = Some(wallet.clone());
        Ok(wallet)
    }

    /// Restore from an exported wallet JSON + password (password-only backups).
    /// Device-bound backups require the OS keychain secret for the same wallet_id.
    pub fn import_wallet_backup(
        &self,
        wallet_json: &str,
        password: &str,
    ) -> Result<WalletFile, WalletError> {
        if self.storage.exists() {
            return Err(WalletError::AlreadyExists);
        }
        let wallet: WalletFile = serde_json::from_str(wallet_json).map_err(|e| {
            WalletError::Operation(anyhow::anyhow!("invalid wallet backup JSON: {e}"))
        })?;
        let _payload = self.decrypt_payload(&wallet, password)?;
        self.storage.save(&wallet)?;
        *self.cached_wallet.lock().unwrap() = Some(wallet.clone());
        let _ = self.unlock(password)?;
        Ok(wallet)
    }

    pub fn unlock(&self, password: &str) -> Result<String, WalletError> {
        let mut wallet = self.storage.load()?;
        let payload = self.decrypt_payload(&wallet, password)?;
        let keyring = keyring_from_payload(&wallet, &payload)?;
        let desc = models::require_network(&wallet.network);
        let public_key = keyring
            .address(desc.family, desc.is_testnet)
            .unwrap_or_else(|_| keyring.primary_address());

        let addresses = keyring.addresses();
        let needs_upgrade = wallet.version < WALLET_FILE_VERSION
            || wallet.account_name.is_empty()
            || wallet.enabled_networks.is_empty();
        if needs_upgrade {
            wallet.version = WALLET_FILE_VERSION;
            wallet.addresses = addresses;
            if wallet.account_name.is_empty() {
                wallet.account_name = DEFAULT_ACCOUNT_NAME.to_string();
            }
            if wallet.enabled_networks.is_empty() {
                wallet.enabled_networks = wallet.import_kind.default_enabled_networks();
            }
            if !payload.mnemonic.is_empty() {
                wallet.import_kind = ImportKind::Mnemonic;
            }
            self.storage.save(&wallet)?;
        }

        let mut session = self.session.lock().unwrap();
        *session = Some(WalletSession { keyring });
        *self.cached_wallet.lock().unwrap() = Some(wallet.clone());

        let network = models::normalize_network_id(&wallet.network).to_string();
        let mut settings = self.get_settings();
        if settings.network != network || settings.enabled_networks != wallet.enabled_networks {
            settings.network = network;
            settings.enabled_networks = wallet.enabled_networks.clone();
            let _ = self.update_settings(settings);
        }

        Ok(public_key)
    }

    /// Re-decrypt from disk with password (+ device secret if bound). Ephemeral — not stored in session.
    pub fn reveal_mnemonic(&self, password: &str) -> Result<String, WalletError> {
        if !self.is_unlocked() {
            return Err(WalletError::Locked);
        }
        let wallet = self.cached_or_disk().ok_or(WalletError::NotFound)?;
        if !wallet.import_kind.has_mnemonic() {
            return Err(WalletError::Operation(anyhow::anyhow!(
                "this wallet has no recovery phrase"
            )));
        }
        let payload = self.decrypt_payload(&wallet, password)?;
        if payload.mnemonic.is_empty() {
            return Err(WalletError::Operation(anyhow::anyhow!(
                "this wallet has no recovery phrase"
            )));
        }
        Ok(payload.mnemonic)
    }

    /// Delete the local wallet without the password (forgot-password / factory reset).
    /// Funds are only recoverable via recovery phrase or a portable backup.
    pub fn reset_local_wallet(&self) -> Result<(), WalletError> {
        let wallet = self.cached_or_disk().ok_or(WalletError::NotFound)?;
        self.wipe_local_wallet(&wallet.wallet_id)
    }

    fn wipe_local_wallet(&self, wallet_id: &str) -> Result<(), WalletError> {
        self.lock();
        self.storage.delete()?;
        let _ = self.device_secrets.delete(wallet_id);
        Ok(())
    }

    pub fn change_password(
        &self,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), WalletError> {
        Self::require_password_strength(new_password)?;
        let wallet = self.cached_or_disk().ok_or(WalletError::NotFound)?;
        let payload = self.decrypt_payload(&wallet, old_password)?;
        let device_secret = self.resolve_device_secret_for_write(&wallet)?;
        let updated = self.reencrypt_wallet_file(
            &wallet,
            &payload,
            new_password,
            wallet.protection,
            device_secret.as_ref(),
        )?;
        self.storage.save(&updated)?;
        *self.cached_wallet.lock().unwrap() = Some(updated);
        Ok(())
    }

    pub fn export_wallet(&self, password: &str) -> Result<String, WalletError> {
        let wallet = self.cached_or_disk().ok_or(WalletError::NotFound)?;
        self.decrypt_payload(&wallet, password)?;
        serde_json::to_string_pretty(&wallet).map_err(|e| {
            WalletError::Operation(anyhow::anyhow!("failed to serialize wallet file: {e}"))
        })
    }

    /// Enable Enhanced device protection (re-encrypt with OS-bound secret).
    pub fn enable_device_protection(&self, password: &str) -> Result<WalletFile, WalletError> {
        let wallet = self.cached_or_disk().ok_or(WalletError::NotFound)?;
        if wallet.protection.is_device_bound() {
            return Ok(wallet);
        }
        let payload = self.decrypt_payload(&wallet, password)?;
        let mut secret = generate_device_secret();
        self.device_secrets
            .set(&wallet.wallet_id, &secret)
            .map_err(Self::map_device_secret_error)?;
        let updated = self.reencrypt_wallet_file(
            &wallet,
            &payload,
            password,
            WalletProtection::PasswordDevice,
            Some(&secret),
        );
        secret.zeroize();
        let updated = updated?;
        self.storage.save(&updated)?;
        *self.cached_wallet.lock().unwrap() = Some(updated.clone());
        Ok(updated)
    }

    /// Disable Enhanced device protection (password-only portable encryption).
    pub fn disable_device_protection(&self, password: &str) -> Result<WalletFile, WalletError> {
        let wallet = self.cached_or_disk().ok_or(WalletError::NotFound)?;
        if !wallet.protection.is_device_bound() {
            return Ok(wallet);
        }
        let payload = self.decrypt_payload(&wallet, password)?;
        let updated = self.reencrypt_wallet_file(
            &wallet,
            &payload,
            password,
            WalletProtection::Password,
            None,
        )?;
        self.storage.save(&updated)?;
        let _ = self.device_secrets.delete(&wallet.wallet_id);
        *self.cached_wallet.lock().unwrap() = Some(updated.clone());
        Ok(updated)
    }

    pub fn device_protection_enabled(&self) -> bool {
        if let Some(wallet) = self.cached_wallet.lock().unwrap().as_ref() {
            return wallet.protection.is_device_bound();
        }
        self.storage
            .load()
            .map(|w| w.protection.is_device_bound())
            .unwrap_or(false)
    }

    fn save_wallet_from_mnemonic(
        &self,
        mnemonic: &str,
        password: &str,
        account_name: &str,
    ) -> Result<WalletFile, WalletError> {
        let keyring =
            FamilyKeyring::from_mnemonic(mnemonic).map_err(|_| WalletError::InvalidMnemonic)?;
        let private_key = keypair_to_base64(keyring.require_solana()?);
        let payload = EncryptedPayload {
            mnemonic: mnemonic.to_string(),
            private_key,
            derivation_path: DEFAULT_DERIVATION_PATH.to_string(),
        };
        let mut network = models::normalize_network_id(&self.get_settings().network).to_string();
        let enabled = ImportKind::Mnemonic.default_enabled_networks();
        if !enabled.iter().any(|id| id == &network) {
            network = models::NETWORK_SOLANA_MAINNET.to_string();
        }
        let wallet = self.encrypt_wallet_file(
            &keyring,
            &payload,
            password,
            &network,
            account_name,
            ImportKind::Mnemonic,
            enabled,
        )?;
        self.storage.save(&wallet)?;
        *self.cached_wallet.lock().unwrap() = Some(wallet.clone());
        Ok(wallet)
    }

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

    #[allow(clippy::too_many_arguments)]
    fn encrypt_wallet_file(
        &self,
        keyring: &FamilyKeyring,
        payload: &EncryptedPayload,
        password: &str,
        network: &str,
        account_name: &str,
        import_kind: ImportKind,
        enabled_networks: Vec<String>,
    ) -> Result<WalletFile, WalletError> {
        let name = account_name.trim();
        let account_name = if name.is_empty() {
            DEFAULT_ACCOUNT_NAME.to_string()
        } else {
            name.chars().take(32).collect()
        };
        let addresses = keyring.addresses();
        self.reencrypt_wallet_file(
            &WalletFile {
                version: WALLET_FILE_VERSION,
                wallet_id: Uuid::new_v4().to_string(),
                network: network.to_string(),
                public_key: keyring.primary_address(),
                created_at: Utc::now().to_rfc3339(),
                protection: WalletProtection::Password,
                addresses,
                account_name,
                import_kind,
                enabled_networks,
                crypto: CryptoEnvelope {
                    kdf: KDF_NAME.into(),
                    salt: String::new(),
                    cipher: CIPHER_NAME.into(),
                    nonce: String::new(),
                    ciphertext: String::new(),
                },
            },
            payload,
            password,
            WalletProtection::Password,
            None,
        )
    }

    fn reencrypt_wallet_file(
        &self,
        existing: &WalletFile,
        payload: &EncryptedPayload,
        password: &str,
        protection: WalletProtection,
        device_secret: Option<&[u8; KEY_LEN]>,
    ) -> Result<WalletFile, WalletError> {
        if protection.is_device_bound() && device_secret.is_none() {
            return Err(WalletError::DeviceSecretMissing);
        }
        let salt = generate_salt();
        let derived = derive_wallet_key(password, &salt, device_secret)?;
        let plaintext = serde_json::to_vec(payload).map_err(|e| {
            WalletError::Operation(anyhow::anyhow!("failed to serialize payload: {e}"))
        })?;
        let (nonce, ciphertext) = encrypt(&plaintext, derived.as_bytes())?;

        Ok(WalletFile {
            version: WALLET_FILE_VERSION,
            wallet_id: existing.wallet_id.clone(),
            network: existing.network.clone(),
            public_key: existing.public_key.clone(),
            created_at: existing.created_at.clone(),
            protection,
            addresses: existing.addresses.clone(),
            account_name: existing.account_name.clone(),
            import_kind: existing.import_kind,
            enabled_networks: existing.enabled_networks.clone(),
            crypto: CryptoEnvelope {
                kdf: KDF_NAME.into(),
                salt: BASE64.encode(salt),
                cipher: CIPHER_NAME.into(),
                nonce: BASE64.encode(nonce),
                ciphertext: BASE64.encode(ciphertext),
            },
        })
    }

    fn resolve_device_secret_for_read(
        &self,
        wallet: &WalletFile,
    ) -> Result<Option<[u8; KEY_LEN]>, WalletError> {
        if !wallet.protection.is_device_bound() {
            return Ok(None);
        }
        match self.device_secrets.get(&wallet.wallet_id) {
            Ok(secret) => Ok(Some(secret)),
            Err(DeviceSecretError::NotFound) => Err(WalletError::DeviceSecretMissing),
            Err(e) => Err(Self::map_device_secret_error(e)),
        }
    }

    fn resolve_device_secret_for_write(
        &self,
        wallet: &WalletFile,
    ) -> Result<Option<[u8; KEY_LEN]>, WalletError> {
        self.resolve_device_secret_for_read(wallet)
    }

    pub(crate) fn decrypt_payload(
        &self,
        wallet: &WalletFile,
        password: &str,
    ) -> Result<EncryptedPayload, WalletError> {
        if wallet.version < MIN_WALLET_FILE_VERSION || wallet.version > WALLET_FILE_VERSION {
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

        let device_secret = self.resolve_device_secret_for_read(wallet)?;
        let derived = derive_wallet_key(password, &salt, device_secret.as_ref())?;
        let key: [u8; KEY_LEN] = *derived.as_bytes();
        let plaintext = decrypt(&nonce, &ciphertext, &key)?;
        let payload: EncryptedPayload =
            serde_json::from_slice(&plaintext).map_err(|_| WalletError::InvalidPassword)?;
        Ok(payload)
    }

    pub(crate) fn verify_password(&self, password: &str) -> Result<(), WalletError> {
        let wallet = self.cached_or_disk().ok_or(WalletError::NotFound)?;
        self.decrypt_payload(&wallet, password)?;
        Ok(())
    }

    fn map_device_secret_error(err: DeviceSecretError) -> WalletError {
        match err {
            DeviceSecretError::NotFound => WalletError::DeviceSecretMissing,
            DeviceSecretError::Unavailable(msg) => WalletError::Operation(anyhow::anyhow!(
                "Enhanced device protection is unavailable on this system: {msg}"
            )),
            DeviceSecretError::Invalid => WalletError::DeviceSecretMissing,
        }
    }
}

fn keyring_from_payload(
    wallet: &WalletFile,
    payload: &EncryptedPayload,
) -> Result<FamilyKeyring, WalletError> {
    if !payload.mnemonic.is_empty() {
        if payload.private_key.is_empty() {
            return FamilyKeyring::from_mnemonic(&payload.mnemonic);
        }
        let keypair = keypair_from_base64(&payload.private_key).map_err(WalletError::Operation)?;
        return FamilyKeyring::from_solana_and_mnemonic(keypair, &payload.mnemonic);
    }
    match wallet.import_kind {
        ImportKind::Mnemonic => Err(WalletError::InvalidMnemonic),
        ImportKind::SolanaKey => {
            let keypair =
                keypair_from_secret_input(&payload.private_key).map_err(WalletError::Operation)?;
            Ok(FamilyKeyring::from_solana_key(keypair))
        }
        ImportKind::EvmKey => {
            let signer =
                taurvia_evm::from_hex(&payload.private_key).map_err(WalletError::Operation)?;
            Ok(FamilyKeyring::from_evm_key(signer))
        }
        ImportKind::BitcoinKey => {
            let (main, test) =
                taurvia_bitcoin::from_wif(&payload.private_key).map_err(WalletError::Operation)?;
            Ok(FamilyKeyring::from_btc_keys(main, test))
        }
    }
}

fn detect_and_parse_key(secret: &str) -> Result<(FamilyKeyring, ImportKind, String), WalletError> {
    let trimmed = secret.trim();
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        let signer = taurvia_evm::from_hex(trimmed).map_err(WalletError::Operation)?;
        let hex = trimmed[2..].to_ascii_lowercase();
        return Ok((
            FamilyKeyring::from_evm_key(signer),
            ImportKind::EvmKey,
            format!("0x{hex}"),
        ));
    }
    if let Ok((main, test)) = taurvia_bitcoin::from_wif(trimmed) {
        return Ok((
            FamilyKeyring::from_btc_keys(main, test),
            ImportKind::BitcoinKey,
            trimmed.to_string(),
        ));
    }
    if let Ok(kp) = keypair_from_secret_input(trimmed) {
        let stored = keypair_to_base64(&kp);
        return Ok((
            FamilyKeyring::from_solana_key(kp),
            ImportKind::SolanaKey,
            stored,
        ));
    }
    Err(WalletError::Operation(anyhow::anyhow!(
        "unrecognized private key (Solana base58/JSON, Ethereum 0x hex, or Bitcoin WIF)"
    )))
}
