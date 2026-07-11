/**
 * Network identity helpers.
 * Today: Solana mainnet/devnet. Keep labels/URLs here so EVM can plug in later
 * without scattering hardcoded "Mainnet" / "solana-mainnet" across the UI.
 */

export const DEFAULT_NETWORK_ID = "solana-mainnet";

export function normalizeNetworkId(value: unknown): string {
  if (typeof value === "string" && value.trim() !== "") {
    return value.trim();
  }
  return DEFAULT_NETWORK_ID;
}

/** Short chip label: Mainnet / Devnet / raw id for unknown chains. */
export function networkShortLabel(id: unknown): string {
  switch (normalizeNetworkId(id)) {
    case "solana-mainnet":
      return "Mainnet";
    case "solana-devnet":
      return "Devnet";
    default:
      return normalizeNetworkId(id);
  }
}

/** Settings / status row id (kebab-case). */
export function networkDisplayId(id: unknown): string {
  return normalizeNetworkId(id);
}

/**
 * Solana explorer cluster query value.
 * Returns null for non-Solana ids (future EVM) so callers skip cluster params.
 */
export function networkCluster(id: unknown): "mainnet-beta" | "devnet" | null {
  switch (normalizeNetworkId(id)) {
    case "solana-mainnet":
      return "mainnet-beta";
    case "solana-devnet":
      return "devnet";
    default:
      return null;
  }
}

export function isSolanaNetwork(id: unknown): boolean {
  return normalizeNetworkId(id).startsWith("solana-");
}

/** Shell subtitle under the brand mark. */
export function productChainLabel(id: unknown): string {
  return isSolanaNetwork(id) ? "Solana Wallet" : "Wallet";
}
