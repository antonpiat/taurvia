use models::{
    normalize_network_id, require_network, AppSettings, ChainFamily, ImportKind, RuntimeConfig,
    WalletFile, DEFAULT_NETWORK_ID,
};
use std::path::Path;
use std::sync::{Arc, Mutex};
use storage::{AppConfigStore, DeviceSecretStore, FileWalletStore};
use taurvia_solana::{configure_jupiter_api_key, Keypair, Pubkey, Signer, SolanaRpc};

use crate::WalletError;
use zeroize::Zeroize;

pub(crate) struct FamilyKeyring {
    pub solana: Option<Keypair>,
    pub evm: Option<taurvia_evm::EvmSigner>,
    pub bitcoin: Option<taurvia_bitcoin::BtcSigner>,
    pub bitcoin_testnet: Option<taurvia_bitcoin::BtcSigner>,
}

impl FamilyKeyring {
    pub fn from_mnemonic(mnemonic: &str) -> Result<Self, WalletError> {
        let seed =
            taurvia_hd::seed_from_mnemonic(mnemonic).map_err(|_| WalletError::InvalidMnemonic)?;
        let solana = taurvia_solana::derive_keypair_from_seed(seed.as_slice())
            .map_err(|_| WalletError::InvalidMnemonic)?;
        let evm = taurvia_evm::derive_from_seed(seed.as_slice()).map_err(WalletError::Operation)?;
        let bitcoin = taurvia_bitcoin::derive_from_seed(seed.as_slice(), false)
            .map_err(WalletError::Operation)?;
        let bitcoin_testnet = taurvia_bitcoin::derive_from_seed(seed.as_slice(), true)
            .map_err(WalletError::Operation)?;
        Ok(Self {
            solana: Some(solana),
            evm: Some(evm),
            bitcoin: Some(bitcoin),
            bitcoin_testnet: Some(bitcoin_testnet),
        })
    }

    pub fn from_solana_and_mnemonic(solana: Keypair, mnemonic: &str) -> Result<Self, WalletError> {
        let seed =
            taurvia_hd::seed_from_mnemonic(mnemonic).map_err(|_| WalletError::InvalidMnemonic)?;
        let evm = taurvia_evm::derive_from_seed(seed.as_slice()).map_err(WalletError::Operation)?;
        let bitcoin = taurvia_bitcoin::derive_from_seed(seed.as_slice(), false)
            .map_err(WalletError::Operation)?;
        let bitcoin_testnet = taurvia_bitcoin::derive_from_seed(seed.as_slice(), true)
            .map_err(WalletError::Operation)?;
        Ok(Self {
            solana: Some(solana),
            evm: Some(evm),
            bitcoin: Some(bitcoin),
            bitcoin_testnet: Some(bitcoin_testnet),
        })
    }

    pub fn from_solana_key(solana: Keypair) -> Self {
        Self {
            solana: Some(solana),
            evm: None,
            bitcoin: None,
            bitcoin_testnet: None,
        }
    }

    pub fn from_evm_key(evm: taurvia_evm::EvmSigner) -> Self {
        Self {
            solana: None,
            evm: Some(evm),
            bitcoin: None,
            bitcoin_testnet: None,
        }
    }

    pub fn from_btc_keys(
        bitcoin: taurvia_bitcoin::BtcSigner,
        bitcoin_testnet: taurvia_bitcoin::BtcSigner,
    ) -> Self {
        Self {
            solana: None,
            evm: None,
            bitcoin: Some(bitcoin),
            bitcoin_testnet: Some(bitcoin_testnet),
        }
    }

    pub fn has_family(&self, family: ChainFamily) -> bool {
        match family {
            ChainFamily::Solana => self.solana.is_some(),
            ChainFamily::Evm => self.evm.is_some(),
            ChainFamily::Bitcoin => self.bitcoin.is_some() || self.bitcoin_testnet.is_some(),
            ChainFamily::Sui => false,
        }
    }

    pub fn require_solana(&self) -> Result<&Keypair, WalletError> {
        self.solana
            .as_ref()
            .ok_or_else(|| WalletError::Operation(anyhow::anyhow!("this wallet has no Solana key")))
    }

    pub fn require_evm(&self) -> Result<&taurvia_evm::EvmSigner, WalletError> {
        self.evm.as_ref().ok_or_else(|| {
            WalletError::Operation(anyhow::anyhow!("this wallet has no Ethereum key"))
        })
    }

    pub fn address(&self, family: ChainFamily, testnet: bool) -> Result<String, WalletError> {
        match family {
            ChainFamily::Solana => Ok(self.require_solana()?.pubkey().to_string()),
            ChainFamily::Evm => Ok(self.require_evm()?.address.clone()),
            ChainFamily::Bitcoin => Ok(self.require_btc(testnet)?.address.clone()),
            ChainFamily::Sui => Err(WalletError::Operation(anyhow::anyhow!(
                "Sui is not enabled yet"
            ))),
        }
    }

    pub fn require_btc(&self, testnet: bool) -> Result<&taurvia_bitcoin::BtcSigner, WalletError> {
        let signer = if testnet {
            self.bitcoin_testnet.as_ref()
        } else {
            self.bitcoin.as_ref()
        };
        signer.ok_or_else(|| {
            WalletError::Operation(anyhow::anyhow!("this wallet has no Bitcoin key"))
        })
    }

    pub fn addresses(&self) -> models::WalletAddresses {
        let mut addresses = models::WalletAddresses::default();
        if let Some(kp) = self.solana.as_ref() {
            addresses.set(ChainFamily::Solana, kp.pubkey().to_string());
        }
        if let Some(evm) = self.evm.as_ref() {
            addresses.set(ChainFamily::Evm, evm.address.clone());
        }
        if let Some(btc) = self.bitcoin.as_ref() {
            addresses.set(ChainFamily::Bitcoin, btc.address.clone());
        }
        addresses
    }

    pub fn primary_address(&self) -> String {
        self.solana
            .as_ref()
            .map(|k| k.pubkey().to_string())
            .or_else(|| self.evm.as_ref().map(|e| e.address.clone()))
            .or_else(|| self.bitcoin.as_ref().map(|b| b.address.clone()))
            .or_else(|| self.bitcoin_testnet.as_ref().map(|b| b.address.clone()))
            .unwrap_or_default()
    }
}

impl Drop for FamilyKeyring {
    fn drop(&mut self) {
        if let Some(solana) = self.solana.as_ref() {
            let mut bytes = solana.to_bytes();
            bytes.zeroize();
        }
    }
}

pub(crate) struct WalletSession {
    pub keyring: FamilyKeyring,
}

pub struct WalletService {
    pub(crate) storage: FileWalletStore,
    pub(crate) config_store: AppConfigStore,
    pub(crate) settings: Mutex<AppSettings>,
    pub(crate) session: Arc<Mutex<Option<WalletSession>>>,
    pub(crate) cached_wallet: Mutex<Option<WalletFile>>,
    pub(crate) rpc: Mutex<SolanaRpc>,
    pub(crate) evm_rpc_url: Mutex<String>,
    pub(crate) btc_esplora: Mutex<String>,
    pub(crate) device_secrets: DeviceSecretStore,
}

impl WalletService {
    /// Desktop/mobile convenience: filesystem wallet store + OS device-secret store.
    pub fn new(data_dir: impl AsRef<Path>, _legacy_rpc_url: Option<&str>) -> Self {
        Self::with_device_secrets(data_dir, _legacy_rpc_url, DeviceSecretStore::os())
    }

    /// Same as `new`, but injects a device-secret backend (use memory in tests).
    pub fn with_device_secrets(
        data_dir: impl AsRef<Path>,
        _legacy_rpc_url: Option<&str>,
        device_secrets: DeviceSecretStore,
    ) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        let config_store = AppConfigStore::new(&data_dir);
        let settings = config_store.load().unwrap_or_default();
        let mut runtime = RuntimeConfig::resolve(&settings);
        if let Some(url) = _legacy_rpc_url.filter(|u| !u.is_empty()) {
            if settings
                .rpc_url
                .as_ref()
                .map(|s| s.trim().is_empty())
                .unwrap_or(true)
                && std::env::var("TAURVIA_RPC_URL").is_err()
            {
                runtime.rpc_url = url.to_string();
            }
        }
        Self::from_parts(
            settings,
            runtime,
            config_store,
            FileWalletStore::new(&data_dir),
            device_secrets,
        )
    }

    pub fn from_parts(
        settings: AppSettings,
        runtime: RuntimeConfig,
        config_store: AppConfigStore,
        storage: FileWalletStore,
        device_secrets: DeviceSecretStore,
    ) -> Self {
        configure_jupiter_api_key(runtime.jupiter_api_key.clone());
        let (sol_rpc, evm_url, btc_url) = split_runtime(&settings, &runtime);
        Self {
            storage,
            config_store,
            settings: Mutex::new(settings),
            session: Arc::new(Mutex::new(None)),
            cached_wallet: Mutex::new(None),
            rpc: Mutex::new(SolanaRpc::new(Some(&sol_rpc))),
            evm_rpc_url: Mutex::new(evm_url),
            btc_esplora: Mutex::new(btc_url),
            device_secrets,
        }
    }

    pub fn get_settings(&self) -> AppSettings {
        self.settings.lock().unwrap().clone()
    }

    pub fn update_settings(&self, settings: AppSettings) -> Result<RuntimeConfig, WalletError> {
        let prev = self.settings.lock().unwrap().clone();
        let mut settings = settings;
        settings.network = normalize_network_id(&settings.network).to_string();
        if self.storage.exists() {
            settings.network = normalize_network_id(&self.wallet_network()).to_string();
            settings.enabled_networks = self.enabled_network_ids();
        }
        self.config_store.save(&settings)?;
        *self.settings.lock().unwrap() = settings.clone();
        let runtime = RuntimeConfig::resolve(&settings);
        let connectivity_changed = prev.rpc_url != settings.rpc_url
            || prev.rpc_urls != settings.rpc_urls
            || prev.jupiter_api_key != settings.jupiter_api_key
            || prev.zerox_api_key != settings.zerox_api_key
            || prev.network != settings.network;
        if connectivity_changed {
            configure_jupiter_api_key(runtime.jupiter_api_key.clone());
            let (sol_rpc, evm_url, btc_url) = split_runtime(&settings, &runtime);
            *self.rpc.lock().unwrap() = SolanaRpc::new(Some(&sol_rpc));
            *self.evm_rpc_url.lock().unwrap() = evm_url;
            *self.btc_esplora.lock().unwrap() = btc_url;
        }
        Ok(runtime)
    }

    pub fn wallet_network(&self) -> String {
        if let Some(wallet) = self.cached_wallet.lock().unwrap().as_ref() {
            return normalize_network_id(&wallet.network).to_string();
        }
        self.storage
            .load()
            .map(|w| normalize_network_id(&w.network).to_string())
            .unwrap_or_else(|_| DEFAULT_NETWORK_ID.to_string())
    }

    pub fn import_kind(&self) -> ImportKind {
        self.cached_or_disk()
            .map(|w| w.import_kind)
            .unwrap_or(ImportKind::Mnemonic)
    }

    pub fn account_name(&self) -> String {
        self.cached_or_disk()
            .map(|w| w.account_name)
            .unwrap_or_else(|| models::DEFAULT_ACCOUNT_NAME.to_string())
    }

    pub fn enabled_network_ids(&self) -> Vec<String> {
        if let Some(wallet) = self.cached_or_disk() {
            if !wallet.enabled_networks.is_empty() {
                return wallet
                    .enabled_networks
                    .into_iter()
                    .map(|id| normalize_network_id(&id).to_string())
                    .collect();
            }
            return wallet.import_kind.default_enabled_networks();
        }
        self.get_settings().enabled_networks
    }

    pub(crate) fn cached_or_disk(&self) -> Option<WalletFile> {
        self.cached_wallet
            .lock()
            .unwrap()
            .clone()
            .or_else(|| self.storage.load().ok())
    }

    fn persist_wallet(&self, wallet: WalletFile) -> Result<(), WalletError> {
        self.storage.save(&wallet)?;
        *self.cached_wallet.lock().unwrap() = Some(wallet);
        Ok(())
    }

    pub fn set_account_name(&self, name: &str) -> Result<(), WalletError> {
        let mut wallet = self.cached_or_disk().ok_or(WalletError::NotFound)?;
        let name = name.trim();
        wallet.account_name = if name.is_empty() {
            models::DEFAULT_ACCOUNT_NAME.to_string()
        } else {
            name.chars().take(32).collect()
        };
        self.persist_wallet(wallet)
    }

    pub fn set_enabled_networks(&self, networks: &[String]) -> Result<RuntimeConfig, WalletError> {
        let mut wallet = self.cached_or_disk().ok_or(WalletError::NotFound)?;
        let kind = wallet.import_kind;
        let mut cleaned = Vec::new();
        for id in networks {
            let id = normalize_network_id(id);
            let desc = require_network(id);
            if !desc.enabled || desc.is_testnet {
                continue;
            }
            if let Some(family) = kind.family() {
                if desc.family != family {
                    return Err(WalletError::Operation(anyhow::anyhow!(
                        "this wallet can only use {}",
                        match family {
                            ChainFamily::Solana => "Solana",
                            ChainFamily::Evm => "Ethereum",
                            ChainFamily::Bitcoin => "Bitcoin",
                            ChainFamily::Sui => "Sui",
                        }
                    )));
                }
            }
            if !cleaned.iter().any(|existing: &String| existing == id) {
                cleaned.push(id.to_string());
            }
        }
        if cleaned.is_empty() {
            return Err(WalletError::Operation(anyhow::anyhow!(
                "keep at least one network enabled"
            )));
        }
        wallet.enabled_networks = cleaned.clone();
        let last = normalize_network_id(&wallet.network);
        let last_desc = require_network(last);
        let last_ok = cleaned.iter().any(|id| {
            let d = require_network(id);
            d.family == last_desc.family
        });
        if !last_ok {
            wallet.network = cleaned[0].clone();
        }
        self.persist_wallet(wallet)?;
        let mut settings = self.get_settings();
        settings.enabled_networks = cleaned;
        settings.network = self.wallet_network();
        self.update_settings(settings)
    }

    fn family_available(&self, family: ChainFamily) -> bool {
        if let Some(session) = self.session.lock().unwrap().as_ref() {
            return session.keyring.has_family(family);
        }
        match self.import_kind().family() {
            None => true,
            Some(owned) => owned == family,
        }
    }

    pub fn change_network(&self, network: &str) -> Result<RuntimeConfig, WalletError> {
        let id = normalize_network_id(network);
        let desc = require_network(id);
        if !desc.enabled {
            return Err(WalletError::Operation(anyhow::anyhow!(
                "{} is not enabled yet",
                desc.name
            )));
        }
        if !self.family_available(desc.family) {
            return Err(WalletError::Operation(anyhow::anyhow!(
                "this wallet has no {} key",
                match desc.family {
                    ChainFamily::Solana => "Solana",
                    ChainFamily::Evm => "Ethereum",
                    ChainFamily::Bitcoin => "Bitcoin",
                    ChainFamily::Sui => "Sui",
                }
            )));
        }
        let enabled = self.enabled_network_ids();
        let family_on = enabled
            .iter()
            .any(|eid| require_network(eid).family == desc.family)
            || enabled.iter().any(|eid| normalize_network_id(eid) == id);
        if !family_on && !desc.is_testnet {
            return Err(WalletError::Operation(anyhow::anyhow!(
                "{} is not activated",
                desc.name
            )));
        }
        if desc.is_testnet {
            let main = models::mainnet_id_for_family(desc.family);
            if !enabled.iter().any(|eid| normalize_network_id(eid) == main) {
                return Err(WalletError::Operation(anyhow::anyhow!(
                    "activate {} before using the testnet",
                    desc.name
                )));
            }
        }

        let mut wallet = self.cached_or_disk().ok_or(WalletError::NotFound)?;
        wallet.network = id.to_string();
        self.persist_wallet(wallet)?;

        let mut settings = self.get_settings();
        settings.network = id.to_string();
        self.update_settings(settings)
    }

    pub(crate) fn rpc_handle(&self) -> SolanaRpc {
        self.rpc.lock().unwrap().clone()
    }

    pub(crate) fn active_descriptor(&self) -> &'static models::NetworkDescriptor {
        require_network(&self.wallet_network())
    }

    pub(crate) fn endpoint_for(&self, network_id: &str) -> String {
        let id = normalize_network_id(network_id);
        let settings = self.get_settings();
        if let Some(url) = settings
            .rpc_urls
            .get(id)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            return url;
        }
        if require_network(id).family == ChainFamily::Solana {
            if let Some(url) = settings
                .rpc_url
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
            {
                return url;
            }
        }
        models::env_rpc_override(require_network(id).family)
            .unwrap_or_else(|| models::managed_rpc_url(id).to_string())
    }

    pub fn wallet_exists(&self) -> bool {
        self.storage.exists()
    }

    pub fn is_unlocked(&self) -> bool {
        self.session.lock().unwrap().is_some()
    }

    pub fn get_public_key(&self) -> Option<String> {
        let desc = self.active_descriptor();
        if let Some(session) = self.session.lock().unwrap().as_ref() {
            return session.keyring.address(desc.family, desc.is_testnet).ok();
        }
        let wallet = self.cached_or_disk()?;
        if desc.family == ChainFamily::Bitcoin && desc.is_testnet {
            // Stored address is mainnet; do not show it on Bitcoin testnet while locked.
            return None;
        }
        if let Some(addr) = wallet.addresses.get(desc.family) {
            return Some(addr.to_string());
        }
        if desc.family == ChainFamily::Solana {
            return Some(wallet.public_key);
        }
        None
    }

    pub fn lock(&self) {
        let mut session = self.session.lock().unwrap();
        *session = None;
        *self.cached_wallet.lock().unwrap() = None;
    }

    pub(crate) fn signing_keypair(&self) -> Result<Keypair, WalletError> {
        let session = self.session.lock().unwrap();
        let session = session.as_ref().ok_or(WalletError::Locked)?;
        let kp = session.keyring.require_solana()?;
        Keypair::try_from(kp.to_bytes().as_slice()).map_err(|_| WalletError::Locked)
    }

    pub(crate) fn require_pubkey(&self) -> Result<Pubkey, WalletError> {
        let session = self.session.lock().unwrap();
        let session = session.as_ref().ok_or(WalletError::Locked)?;
        Ok(session.keyring.require_solana()?.pubkey())
    }

    pub(crate) fn with_session<T>(
        &self,
        f: impl FnOnce(&FamilyKeyring) -> T,
    ) -> Result<T, WalletError> {
        let session = self.session.lock().unwrap();
        let session = session.as_ref().ok_or(WalletError::Locked)?;
        Ok(f(&session.keyring))
    }

    pub(crate) fn snapshot_descriptors(&self) -> Vec<&'static models::NetworkDescriptor> {
        let last = require_network(&self.wallet_network());
        let mut out = Vec::new();
        for id in self.enabled_network_ids() {
            let desc = require_network(&id);
            if !desc.enabled {
                continue;
            }
            if desc.family == last.family {
                if !out
                    .iter()
                    .any(|d: &&models::NetworkDescriptor| d.id == last.id)
                {
                    out.push(last);
                }
            } else if !out
                .iter()
                .any(|d: &&models::NetworkDescriptor| d.family == desc.family)
            {
                out.push(desc);
            }
        }
        if out.is_empty() {
            out.push(last);
        }
        out
    }
}

fn split_runtime(settings: &AppSettings, runtime: &RuntimeConfig) -> (String, String, String) {
    let desc = require_network(&settings.network);
    let sol = if desc.family == ChainFamily::Solana {
        runtime.rpc_url.clone()
    } else {
        settings
            .rpc_urls
            .get(models::NETWORK_SOLANA_MAINNET)
            .cloned()
            .or_else(|| settings.rpc_url.clone())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| models::MANAGED_DEFAULT_RPC_URL.to_string())
    };
    let evm = if desc.family == ChainFamily::Evm {
        runtime.rpc_url.clone()
    } else {
        require_network(models::NETWORK_ETHEREUM_MAINNET)
            .default_rpc
            .to_string()
    };
    let btc = if desc.family == ChainFamily::Bitcoin {
        runtime.rpc_url.clone()
    } else {
        require_network(models::NETWORK_BITCOIN_MAINNET)
            .default_rpc
            .to_string()
    };
    (sol, evm, btc)
}
