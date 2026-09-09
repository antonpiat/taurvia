/**
 * Curated tokens + bundled icons. Prefer local logos; remote Jupiter / Trust Wallet for the rest.
 */
import type { TokenInfo } from "@/bindings";
import solLogo from "@/assets/tokens/sol.png";
import usdcLogo from "@/assets/tokens/usdc.png";
import usdtLogo from "@/assets/tokens/usdt.png";
import jupLogo from "@/assets/tokens/jup.png";
import bonkLogo from "@/assets/tokens/bonk.png";
import ethLogo from "@/assets/tokens/eth.svg";
import btcLogo from "@/assets/tokens/btc.svg";

export const WRAPPED_SOL = "So11111111111111111111111111111111111111112";
export const ETH_NATIVE = "eth";
export const BTC_NATIVE = "btc";

export const ETH_USDC = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
export const ETH_USDT = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
export const ETH_WETH = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
export const ETH_DAI = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
export const ETH_WBTC = "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599";

export type TokenChain = "solana" | "evm" | "bitcoin";

const SOL_LOGOS: Record<string, string> = {
  [WRAPPED_SOL]: solLogo,
  EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v: usdcLogo,
  Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB: usdtLogo,
  JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN: jupLogo,
  DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263: bonkLogo,
};

const EVM_LOGOS: Record<string, string> = {
  [ETH_NATIVE]: ethLogo,
  [ETH_WETH.toLowerCase()]: ethLogo,
  [ETH_USDC.toLowerCase()]: usdcLogo,
  [ETH_USDT.toLowerCase()]: usdtLogo,
  [ETH_DAI.toLowerCase()]: usdcLogo,
  [ETH_WBTC.toLowerCase()]: btcLogo,
};

export const MAJOR_TOKENS: TokenInfo[] = [
  {
    mint: WRAPPED_SOL,
    symbol: "SOL",
    name: "Solana",
    decimals: 9,
    logo_uri: SOL_LOGOS[WRAPPED_SOL],
  },
  {
    mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    symbol: "USDC",
    name: "USD Coin",
    decimals: 6,
    logo_uri: SOL_LOGOS.EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v,
  },
  {
    mint: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
    symbol: "USDT",
    name: "Tether USD",
    decimals: 6,
    logo_uri: SOL_LOGOS.Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB,
  },
  {
    mint: "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN",
    symbol: "JUP",
    name: "Jupiter",
    decimals: 6,
    logo_uri: SOL_LOGOS.JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN,
  },
  {
    mint: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
    symbol: "BONK",
    name: "Bonk",
    decimals: 5,
    logo_uri: SOL_LOGOS.DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263,
  },
];

export const ETH_MAJOR_TOKENS: TokenInfo[] = [
  { mint: ETH_NATIVE, symbol: "ETH", name: "Ethereum", decimals: 18, logo_uri: ethLogo },
  { mint: ETH_WETH, symbol: "WETH", name: "Wrapped Ether", decimals: 18, logo_uri: ethLogo },
  { mint: ETH_USDC, symbol: "USDC", name: "USD Coin", decimals: 6, logo_uri: usdcLogo },
  { mint: ETH_USDT, symbol: "USDT", name: "Tether USD", decimals: 6, logo_uri: usdtLogo },
  { mint: ETH_DAI, symbol: "DAI", name: "Dai", decimals: 18, logo_uri: usdcLogo },
  { mint: ETH_WBTC, symbol: "WBTC", name: "Wrapped BTC", decimals: 8, logo_uri: btcLogo },
];

export const BTC_NATIVE_TOKEN: TokenInfo = {
  mint: BTC_NATIVE,
  symbol: "BTC",
  name: "Bitcoin",
  decimals: 8,
  logo_uri: btcLogo,
};

export function chainBadgeSrc(chain: TokenChain): string {
  if (chain === "evm") return ethLogo;
  if (chain === "bitcoin") return btcLogo;
  return solLogo;
}

export function localLogoForAsset(
  chain: TokenChain | undefined,
  mint: string,
  symbol?: string,
): string | null {
  const m = mint.trim();
  if (chain === "bitcoin" || m.toLowerCase() === "btc" || symbol === "BTC") return btcLogo;
  if (chain === "evm") {
    if (m.toLowerCase() === "eth" || symbol === "ETH") return ethLogo;
    return EVM_LOGOS[m.toLowerCase()] ?? null;
  }
  if (m.toLowerCase() === "eth") return ethLogo;
  if (m.toLowerCase() === "btc") return btcLogo;
  if (m.toLowerCase() === "sol" || symbol === "SOL") return solLogo;
  return SOL_LOGOS[m] ?? null;
}

export function withLocalLogo<T extends { mint: string; logo_uri?: string | null; symbol?: string }>(
  token: T,
  chain?: TokenChain,
): T {
  const local = localLogoForAsset(chain, token.mint, token.symbol);
  if (!local) return token;
  return { ...token, logo_uri: local };
}

export function isCuratedMint(mint: string): boolean {
  return Object.prototype.hasOwnProperty.call(SOL_LOGOS, mint);
}

export function toStoredFavorite(info: TokenInfo): TokenInfo | null {
  if (isCuratedMint(info.mint)) return null;
  const logo =
    info.logo_uri &&
    (info.logo_uri.startsWith("https://") || info.logo_uri.startsWith("http://"))
      ? info.logo_uri
      : null;
  return {
    mint: info.mint,
    symbol: info.symbol,
    name: info.name,
    decimals: info.decimals,
    logo_uri: logo,
  };
}

export const MAX_SWAP_FAVORITES = 50;

export function networkFamilyToChain(family: string | undefined): TokenChain | undefined {
  if (family === "evm") return "evm";
  if (family === "bitcoin") return "bitcoin";
  if (family === "solana") return "solana";
  return undefined;
}

export function inferTokenChain(mint: string): TokenChain {
  const m = mint.trim();
  if (m.toLowerCase() === "btc") return "bitcoin";
  if (m.toLowerCase() === "eth" || m.startsWith("0x") || m.startsWith("0X")) return "evm";
  return "solana";
}
