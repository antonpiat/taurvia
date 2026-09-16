use serde::{Deserialize, Serialize};
use specta::Type;

pub const NETWORK_SOLANA_MAINNET: &str = "solana-mainnet";
pub const NETWORK_SOLANA_DEVNET: &str = "solana-devnet";
pub const NETWORK_ETHEREUM_MAINNET: &str = "ethereum-mainnet";
pub const NETWORK_ETHEREUM_SEPOLIA: &str = "ethereum-sepolia";
pub const NETWORK_BITCOIN_MAINNET: &str = "bitcoin-mainnet";
pub const NETWORK_BITCOIN_TESTNET: &str = "bitcoin-testnet";
pub const NETWORK_SUI_MAINNET: &str = "sui-mainnet";
pub const NETWORK_SUI_TESTNET: &str = "sui-testnet";
pub const DEFAULT_NETWORK_ID: &str = NETWORK_SOLANA_MAINNET;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "kebab-case")]
pub enum ChainFamily {
    Solana,
    Evm,
    Bitcoin,
    Sui,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ChainFeatures {
    pub tokens: bool,
    pub swap: bool,
    pub utxo: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct NetworkDescriptor {
    pub id: &'static str,
    pub family: ChainFamily,
    pub name: &'static str,
    pub native_symbol: &'static str,
    pub is_testnet: bool,
    pub eip155_chain_id: Option<u64>,
    pub default_rpc: &'static str,
    pub explorer_tx: &'static str,
    pub explorer_address: &'static str,
    pub explorer_api: Option<&'static str>,
    pub features: ChainFeatures,
    pub enabled: bool,
    pub coingecko_id: Option<&'static str>,
}

impl NetworkDescriptor {
    pub fn to_info(self) -> NetworkInfo {
        NetworkInfo {
            id: self.id.to_string(),
            family: self.family,
            name: self.name.to_string(),
            native_symbol: self.native_symbol.to_string(),
            is_testnet: self.is_testnet,
            eip155_chain_id: self.eip155_chain_id,
            default_rpc: self.default_rpc.to_string(),
            explorer_tx: self.explorer_tx.to_string(),
            explorer_address: self.explorer_address.to_string(),
            features: self.features,
            enabled: self.enabled,
        }
    }
}

/// Specta/UI copy of a descriptor (owned strings).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NetworkInfo {
    pub id: String,
    pub family: ChainFamily,
    pub name: String,
    pub native_symbol: String,
    pub is_testnet: bool,
    pub eip155_chain_id: Option<u64>,
    pub default_rpc: String,
    pub explorer_tx: String,
    pub explorer_address: String,
    pub features: ChainFeatures,
    pub enabled: bool,
}

const SOLANA_FEATURES: ChainFeatures = ChainFeatures {
    tokens: true,
    swap: true,
    utxo: false,
};

const EVM_FEATURES: ChainFeatures = ChainFeatures {
    tokens: true,
    swap: true,
    utxo: false,
};

const BITCOIN_FEATURES: ChainFeatures = ChainFeatures {
    tokens: false,
    swap: true,
    utxo: true,
};

const SUI_FEATURES: ChainFeatures = ChainFeatures {
    tokens: true,
    swap: false,
    utxo: false,
};

/// Static network table. **EVM L2s (Polygon, Base) are extra rows** with `ChainFamily::Evm`.
/// A new VM needs a family crate + `ChainFamily` variant — not a descriptor-only change.
pub static NETWORKS: &[NetworkDescriptor] = &[
    NetworkDescriptor {
        id: NETWORK_SOLANA_MAINNET,
        family: ChainFamily::Solana,
        name: "Solana",
        native_symbol: "SOL",
        is_testnet: false,
        eip155_chain_id: None,
        default_rpc: "https://api.mainnet-beta.solana.com",
        explorer_tx: "https://solscan.io/tx/{txid}",
        explorer_address: "https://solscan.io/account/{address}",
        explorer_api: None,
        features: SOLANA_FEATURES,
        enabled: true,
        coingecko_id: Some("solana"),
    },
    NetworkDescriptor {
        id: NETWORK_SOLANA_DEVNET,
        family: ChainFamily::Solana,
        name: "Solana Devnet",
        native_symbol: "SOL",
        is_testnet: true,
        eip155_chain_id: None,
        default_rpc: "https://api.devnet.solana.com",
        explorer_tx: "https://solscan.io/tx/{txid}?cluster=devnet",
        explorer_address: "https://solscan.io/account/{address}?cluster=devnet",
        explorer_api: None,
        features: ChainFeatures {
            tokens: true,
            swap: false,
            utxo: false,
        },
        enabled: true,
        coingecko_id: Some("solana"),
    },
    NetworkDescriptor {
        id: NETWORK_ETHEREUM_MAINNET,
        family: ChainFamily::Evm,
        name: "Ethereum",
        native_symbol: "ETH",
        is_testnet: false,
        eip155_chain_id: Some(1),
        default_rpc: "https://ethereum-rpc.publicnode.com",
        explorer_tx: "https://etherscan.io/tx/{txid}",
        explorer_address: "https://etherscan.io/address/{address}",
        explorer_api: Some("https://api.etherscan.io/api"),
        features: EVM_FEATURES,
        enabled: true,
        coingecko_id: Some("ethereum"),
    },
    NetworkDescriptor {
        id: NETWORK_ETHEREUM_SEPOLIA,
        family: ChainFamily::Evm,
        name: "Ethereum Sepolia",
        native_symbol: "ETH",
        is_testnet: true,
        eip155_chain_id: Some(11155111),
        default_rpc: "https://ethereum-sepolia-rpc.publicnode.com",
        explorer_tx: "https://sepolia.etherscan.io/tx/{txid}",
        explorer_address: "https://sepolia.etherscan.io/address/{address}",
        explorer_api: Some("https://api-sepolia.etherscan.io/api"),
        features: EVM_FEATURES,
        enabled: true,
        coingecko_id: Some("ethereum"),
    },
    NetworkDescriptor {
        id: NETWORK_BITCOIN_MAINNET,
        family: ChainFamily::Bitcoin,
        name: "Bitcoin",
        native_symbol: "BTC",
        is_testnet: false,
        eip155_chain_id: None,
        default_rpc: "https://blockstream.info/api",
        explorer_tx: "https://mempool.space/tx/{txid}",
        explorer_address: "https://mempool.space/address/{address}",
        explorer_api: Some("https://blockstream.info/api"),
        features: BITCOIN_FEATURES,
        enabled: true,
        coingecko_id: Some("bitcoin"),
    },
    NetworkDescriptor {
        id: NETWORK_BITCOIN_TESTNET,
        family: ChainFamily::Bitcoin,
        name: "Bitcoin Testnet",
        native_symbol: "BTC",
        is_testnet: true,
        eip155_chain_id: None,
        default_rpc: "https://blockstream.info/testnet/api",
        explorer_tx: "https://mempool.space/testnet/tx/{txid}",
        explorer_address: "https://mempool.space/testnet/address/{address}",
        explorer_api: Some("https://blockstream.info/testnet/api"),
        features: BITCOIN_FEATURES,
        enabled: true,
        coingecko_id: Some("bitcoin"),
    },
    NetworkDescriptor {
        id: "polygon-mainnet",
        family: ChainFamily::Evm,
        name: "Polygon",
        native_symbol: "POL",
        is_testnet: false,
        eip155_chain_id: Some(137),
        default_rpc: "https://polygon-bor-rpc.publicnode.com",
        explorer_tx: "https://polygonscan.com/tx/{txid}",
        explorer_address: "https://polygonscan.com/address/{address}",
        explorer_api: Some("https://api.polygonscan.com/api"),
        features: EVM_FEATURES,
        enabled: false,
        coingecko_id: Some("matic-network"),
    },
    NetworkDescriptor {
        id: "polygon-amoy",
        family: ChainFamily::Evm,
        name: "Polygon Amoy",
        native_symbol: "POL",
        is_testnet: true,
        eip155_chain_id: Some(80002),
        default_rpc: "https://rpc-amoy.polygon.technology",
        explorer_tx: "https://amoy.polygonscan.com/tx/{txid}",
        explorer_address: "https://amoy.polygonscan.com/address/{address}",
        explorer_api: Some("https://api-amoy.polygonscan.com/api"),
        features: EVM_FEATURES,
        enabled: false,
        coingecko_id: Some("matic-network"),
    },
    NetworkDescriptor {
        id: "base-mainnet",
        family: ChainFamily::Evm,
        name: "Base",
        native_symbol: "ETH",
        is_testnet: false,
        eip155_chain_id: Some(8453),
        default_rpc: "https://base-rpc.publicnode.com",
        explorer_tx: "https://basescan.org/tx/{txid}",
        explorer_address: "https://basescan.org/address/{address}",
        explorer_api: Some("https://api.basescan.org/api"),
        features: EVM_FEATURES,
        enabled: false,
        coingecko_id: Some("ethereum"),
    },
    NetworkDescriptor {
        id: "base-sepolia",
        family: ChainFamily::Evm,
        name: "Base Sepolia",
        native_symbol: "ETH",
        is_testnet: true,
        eip155_chain_id: Some(84532),
        default_rpc: "https://sepolia.base.org",
        explorer_tx: "https://sepolia.basescan.org/tx/{txid}",
        explorer_address: "https://sepolia.basescan.org/address/{address}",
        explorer_api: Some("https://api-sepolia.basescan.org/api"),
        features: EVM_FEATURES,
        enabled: false,
        coingecko_id: Some("ethereum"),
    },
    NetworkDescriptor {
        id: NETWORK_SUI_MAINNET,
        family: ChainFamily::Sui,
        name: "Sui",
        native_symbol: "SUI",
        is_testnet: false,
        eip155_chain_id: None,
        default_rpc: "https://sui-rpc.publicnode.com",
        explorer_tx: "https://suiscan.xyz/mainnet/tx/{txid}",
        explorer_address: "https://suiscan.xyz/mainnet/account/{address}",
        explorer_api: None,
        features: SUI_FEATURES,
        enabled: true,
        coingecko_id: Some("sui"),
    },
    NetworkDescriptor {
        id: NETWORK_SUI_TESTNET,
        family: ChainFamily::Sui,
        name: "Sui Testnet",
        native_symbol: "SUI",
        is_testnet: true,
        eip155_chain_id: None,
        default_rpc: "https://sui-testnet-rpc.publicnode.com",
        explorer_tx: "https://suiscan.xyz/testnet/tx/{txid}",
        explorer_address: "https://suiscan.xyz/testnet/account/{address}",
        explorer_api: None,
        features: ChainFeatures {
            tokens: true,
            swap: false,
            utxo: false,
        },
        enabled: true,
        coingecko_id: Some("sui"),
    },
];

pub fn get_network(id: &str) -> Option<&'static NetworkDescriptor> {
    let id = normalize_network_id(id);
    NETWORKS.iter().find(|n| n.id == id)
}

pub fn require_network(id: &str) -> &'static NetworkDescriptor {
    get_network(id).unwrap_or(&NETWORKS[0])
}

pub fn list_network_info(enabled_only: bool) -> Vec<NetworkInfo> {
    NETWORKS
        .iter()
        .filter(|n| !enabled_only || n.enabled)
        .map(|n| n.to_info())
        .collect()
}

/// Unknown / empty / legacy aliases → solana-mainnet.
pub fn normalize_network_id(value: &str) -> &'static str {
    match value.trim() {
        "" | "mainnet" | "solana-mainnet" => NETWORK_SOLANA_MAINNET,
        "devnet" | "solana-devnet" => NETWORK_SOLANA_DEVNET,
        other => NETWORKS
            .iter()
            .find(|n| n.id == other)
            .map(|n| n.id)
            .unwrap_or(NETWORK_SOLANA_MAINNET),
    }
}

pub fn managed_rpc_url(network_id: &str) -> &'static str {
    require_network(network_id).default_rpc
}

pub fn default_enabled_network_ids() -> Vec<String> {
    vec![
        NETWORK_SOLANA_MAINNET.to_string(),
        NETWORK_ETHEREUM_MAINNET.to_string(),
        NETWORK_BITCOIN_MAINNET.to_string(),
        NETWORK_SUI_MAINNET.to_string(),
    ]
}

pub fn mainnet_id_for_family(family: ChainFamily) -> &'static str {
    match family {
        ChainFamily::Solana => NETWORK_SOLANA_MAINNET,
        ChainFamily::Evm => NETWORK_ETHEREUM_MAINNET,
        ChainFamily::Bitcoin => NETWORK_BITCOIN_MAINNET,
        ChainFamily::Sui => NETWORK_SUI_MAINNET,
    }
}

/// Env overlay for a family (dev-only). Network-specific map in settings wins first.
pub fn env_rpc_override(family: ChainFamily) -> Option<String> {
    let key = match family {
        ChainFamily::Solana => "TAURVIA_RPC_URL",
        ChainFamily::Evm => "TAURVIA_ETH_RPC_URL",
        ChainFamily::Bitcoin => "TAURVIA_BTC_ESPLORA_URL",
        ChainFamily::Sui => "TAURVIA_SUI_RPC_URL",
    };
    std::env::var(key)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
