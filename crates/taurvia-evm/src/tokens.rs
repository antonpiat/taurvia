use models::TokenBalance;

#[derive(Clone, Copy)]
pub struct CuratedToken {
    pub address: &'static str,
    pub symbol: &'static str,
    pub name: &'static str,
    pub decimals: u8,
    pub logo_uri: Option<&'static str>,
}

pub fn curated_tokens(network_id: &str) -> &'static [CuratedToken] {
    match network_id {
        models::NETWORK_ETHEREUM_MAINNET => ETHEREUM_MAINNET,
        models::NETWORK_BNB_MAINNET => BNB_MAINNET,
        _ => &[],
    }
}

const TW_WETH: &str = "https://raw.githubusercontent.com/trustwallet/assets/master/blockchains/ethereum/assets/0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2/logo.png";
const TW_USDC: &str = "https://raw.githubusercontent.com/trustwallet/assets/master/blockchains/ethereum/assets/0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48/logo.png";
const TW_USDT: &str = "https://raw.githubusercontent.com/trustwallet/assets/master/blockchains/ethereum/assets/0xdAC17F958D2ee523a2206206994597C13D831ec7/logo.png";
const TW_DAI: &str = "https://raw.githubusercontent.com/trustwallet/assets/master/blockchains/ethereum/assets/0x6B175474E89094C44Da98b954EedeAC495271d0F/logo.png";
const TW_WBTC: &str = "https://raw.githubusercontent.com/trustwallet/assets/master/blockchains/ethereum/assets/0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599/logo.png";

const ETHEREUM_MAINNET: &[CuratedToken] = &[
    CuratedToken {
        address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        symbol: "WETH",
        name: "Wrapped Ether",
        decimals: 18,
        logo_uri: Some(TW_WETH),
    },
    CuratedToken {
        address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        symbol: "USDC",
        name: "USD Coin",
        decimals: 6,
        logo_uri: Some(TW_USDC),
    },
    CuratedToken {
        address: "0xdAC17F958D2ee523a2206206994597C13D831ec7",
        symbol: "USDT",
        name: "Tether USD",
        decimals: 6,
        logo_uri: Some(TW_USDT),
    },
    CuratedToken {
        address: "0x6B175474E89094C44Da98b954EedeAC495271d0F",
        symbol: "DAI",
        name: "Dai Stablecoin",
        decimals: 18,
        logo_uri: Some(TW_DAI),
    },
    CuratedToken {
        address: "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599",
        symbol: "WBTC",
        name: "Wrapped BTC",
        decimals: 8,
        logo_uri: Some(TW_WBTC),
    },
];

const TW_WBNB: &str = "https://raw.githubusercontent.com/trustwallet/assets/master/blockchains/smartchain/assets/0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c/logo.png";
const TW_BSC_USDT: &str = "https://raw.githubusercontent.com/trustwallet/assets/master/blockchains/smartchain/assets/0x55d398326f99059fF775485246999027B3197955/logo.png";
const TW_BSC_USDC: &str = "https://raw.githubusercontent.com/trustwallet/assets/master/blockchains/smartchain/assets/0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d/logo.png";

const BNB_MAINNET: &[CuratedToken] = &[
    CuratedToken {
        address: "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c",
        symbol: "WBNB",
        name: "Wrapped BNB",
        decimals: 18,
        logo_uri: Some(TW_WBNB),
    },
    CuratedToken {
        address: "0x55d398326f99059fF775485246999027B3197955",
        symbol: "USDT",
        name: "Tether USD",
        decimals: 18,
        logo_uri: Some(TW_BSC_USDT),
    },
    CuratedToken {
        address: "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d",
        symbol: "USDC",
        name: "USD Coin",
        decimals: 18,
        logo_uri: Some(TW_BSC_USDC),
    },
];

pub fn token_info(token: &CuratedToken) -> models::TokenInfo {
    models::TokenInfo {
        mint: token.address.to_string(),
        symbol: token.symbol.to_string(),
        name: token.name.to_string(),
        decimals: token.decimals,
        logo_uri: token.logo_uri.map(|s| s.to_string()),
    }
}

pub fn resolve_curated(network_id: &str, asset: &str) -> Option<&'static CuratedToken> {
    if asset.eq_ignore_ascii_case("eth")
        || asset.eq_ignore_ascii_case("bnb")
        || asset.eq_ignore_ascii_case("native")
    {
        return None;
    }
    curated_tokens(network_id)
        .iter()
        .find(|t| t.address.eq_ignore_ascii_case(asset))
}

pub fn token_balance(
    token: &CuratedToken,
    raw: alloy::primitives::U256,
    price_usd: Option<f64>,
) -> Option<TokenBalance> {
    if raw.is_zero() {
        return None;
    }
    let ui = u256_to_f64(raw, token.decimals);
    Some(TokenBalance {
        mint: token.address.to_string(),
        symbol: token.symbol.to_string(),
        name: token.name.to_string(),
        amount: raw.to_string(),
        decimals: token.decimals,
        ui_amount: ui,
        logo_uri: token.logo_uri.map(|s| s.to_string()),
        price_usd,
        value_usd: price_usd.map(|p| p * ui),
    })
}

pub fn u256_to_f64(value: alloy::primitives::U256, decimals: u8) -> f64 {
    let s = value.to_string();
    let raw: f64 = s.parse().unwrap_or(0.0);
    raw / 10f64.powi(decimals as i32)
}

pub fn f64_to_u256(amount: f64, decimals: u8) -> alloy::primitives::U256 {
    let scaled = (amount * 10f64.powi(decimals as i32)).round();
    alloy::primitives::U256::from(scaled as u128)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bnb_mainnet_has_curated_majors() {
        let tokens = curated_tokens(models::NETWORK_BNB_MAINNET);
        assert!(tokens.iter().any(|t| t.symbol == "WBNB"));
        assert!(tokens.iter().any(|t| t.symbol == "USDT"));
        assert!(tokens.iter().any(|t| t.symbol == "USDC"));
        assert!(curated_tokens(models::NETWORK_BNB_TESTNET).is_empty());
        assert!(resolve_curated(models::NETWORK_BNB_MAINNET, "bnb").is_none());
    }
}
