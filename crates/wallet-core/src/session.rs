use taurvia_solana::{configure_jupiter_api_key, Keypair, Pubkey, Signer, SolanaRpc};
use models::{AppSettings, Network, RuntimeConfig, WalletFile};
use storage::{AppConfigStore, FileWalletStore};
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::WalletError;

pub(crate) struct WalletSession {
    pub public_key: String,
    pub keypair: Keypair,
}

pub struct WalletService {
    pub(crate) storage: FileWalletStore,
    pub(crate) config_store: AppConfigStore,
    pub(crate) settings: Mutex<AppSettings>,
    pub(crate) session: Arc<Mutex<Option<WalletSession>>>,
    pub(crate) cached_wallet: Mutex<Option<WalletFile>>,
    pub(crate) rpc: Mutex<SolanaRpc>,
}

impl WalletService {
    /// Desktop/mobile convenience: filesystem wallet store + resolved RuntimeConfig.
    pub fn new(data_dir: impl AsRef<Path>, _legacy_rpc_url: Option<&str>) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        let config_store = AppConfigStore::new(&data_dir);
        let settings = config_store.load().unwrap_or_default();
        // Prefer explicit constructor arg only if settings/env did not resolve a custom URL
        // and caller passed a URL (tests). Otherwise RuntimeConfig::resolve handles defaults.
        let mut runtime = RuntimeConfig::resolve(&settings);
        if let Some(url) = _legacy_rpc_url.filter(|u| !u.is_empty()) {
            // Tests pass localhost; honor when settings have no override.
            if settings.rpc_url.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true)
                && std::env::var("TAURVIA_RPC_URL").is_err()
            {
                runtime.rpc_url = url.to_string();
            }
        }
        Self::from_parts(settings, runtime, config_store, FileWalletStore::new(&data_dir))
    }

    pub fn from_parts(
        settings: AppSettings,
        runtime: RuntimeConfig,
        config_store: AppConfigStore,
        storage: FileWalletStore,
    ) -> Self {
        configure_jupiter_api_key(runtime.jupiter_api_key.clone());
        Self {
            storage,
            config_store,
            settings: Mutex::new(settings),
            session: Arc::new(Mutex::new(None)),
            cached_wallet: Mutex::new(None),
            rpc: Mutex::new(SolanaRpc::new(Some(&runtime.rpc_url))),
        }
    }

    pub fn get_settings(&self) -> AppSettings {
        self.settings.lock().unwrap().clone()
    }

    pub fn update_settings(&self, settings: AppSettings) -> Result<RuntimeConfig, WalletError> {
        let prev = self.settings.lock().unwrap().clone();
        self.config_store.save(&settings)?;
        *self.settings.lock().unwrap() = settings.clone();
        let runtime = RuntimeConfig::resolve(&settings);
        // Skip RPC / Jupiter rebuild for UI-only prefs (layout, auto-lock, explorer, …).
        let connectivity_changed =
            prev.rpc_url != settings.rpc_url || prev.jupiter_api_key != settings.jupiter_api_key;
        if connectivity_changed {
            configure_jupiter_api_key(runtime.jupiter_api_key.clone());
            *self.rpc.lock().unwrap() = SolanaRpc::new(Some(&runtime.rpc_url));
        }
        Ok(runtime)
    }

    /// Network id from the wallet file (`solana-mainnet`, …). Default when none exists.
    pub fn wallet_network(&self) -> String {
        if let Some(wallet) = self.cached_wallet.lock().unwrap().as_ref() {
            return wallet.network.clone();
        }
        self.storage
            .load()
            .map(|w| w.network)
            .unwrap_or_else(|_| Network::SolanaMainnet.as_str().to_string())
    }

    pub fn runtime_config(&self) -> RuntimeConfig {
        RuntimeConfig::resolve(&self.settings.lock().unwrap())
    }

    pub(crate) fn rpc_handle(&self) -> SolanaRpc {
        self.rpc.lock().unwrap().clone()
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
