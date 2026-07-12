/**
 * Curated Swap tokens + bundled icons for instant paint (no remote fetch).
 */
import type { TokenInfo } from "@/bindings";
import solLogo from "@/assets/tokens/sol.png";
import usdcLogo from "@/assets/tokens/usdc.png";
import usdtLogo from "@/assets/tokens/usdt.png";
import jupLogo from "@/assets/tokens/jup.png";
import bonkLogo from "@/assets/tokens/bonk.png";

export const WRAPPED_SOL = "So11111111111111111111111111111111111111112";

const LOCAL_LOGOS: Record<string, string> = {
  [WRAPPED_SOL]: solLogo,
  EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v: usdcLogo,
  Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB: usdtLogo,
  JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN: jupLogo,
  DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263: bonkLogo,
};

export const MAJOR_TOKENS: TokenInfo[] = [
  {
    mint: WRAPPED_SOL,
    symbol: "SOL",
    name: "Solana",
    decimals: 9,
    logo_uri: LOCAL_LOGOS[WRAPPED_SOL],
  },
  {
    mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    symbol: "USDC",
    name: "USD Coin",
    decimals: 6,
    logo_uri: LOCAL_LOGOS.EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v,
  },
  {
    mint: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
    symbol: "USDT",
    name: "Tether USD",
    decimals: 6,
    logo_uri: LOCAL_LOGOS.Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB,
  },
  {
    mint: "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN",
    symbol: "JUP",
    name: "Jupiter",
    decimals: 6,
    logo_uri: LOCAL_LOGOS.JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN,
  },
  {
    mint: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
    symbol: "BONK",
    name: "Bonk",
    decimals: 5,
    logo_uri: LOCAL_LOGOS.DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263,
  },
];

export function localLogoForMint(mint: string): string | null {
  return LOCAL_LOGOS[mint] ?? null;
}

/** Prefer bundled curated icons over remote logo_uri. */
export function withLocalLogo<T extends { mint: string; logo_uri?: string | null }>(
  token: T,
): T {
  const local = localLogoForMint(token.mint);
  if (!local) return token;
  return { ...token, logo_uri: local };
}

export function isCuratedMint(mint: string): boolean {
  return Object.prototype.hasOwnProperty.call(LOCAL_LOGOS, mint);
}

/**
 * Lean row for config.json: mint + display fields only.
 * Skips curated defaults (already bundled). Drops Vite/local asset paths;
 * keeps http(s) logos when present (small URLs, not image bytes).
 */
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
