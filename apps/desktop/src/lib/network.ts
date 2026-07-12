/**
 * Network identity helpers.
 * Solana mainnet/devnet today; keep labels/URLs here so EVM can plug in later.
 */

import type { Network } from "@/bindings";

export const DEFAULT_NETWORK_ID: Network = "solana-mainnet";

export function normalizeNetworkId(value: unknown): string {
  if (value === "solana-devnet" || value === "devnet") return "solana-devnet";
  if (value === "solana-mainnet" || value === "mainnet") return "solana-mainnet";
  if (typeof value === "string" && value.trim() !== "") {
    return value.trim();
  }
  return DEFAULT_NETWORK_ID;
}

export function toNetwork(value: unknown): Network {
  return normalizeNetworkId(value) === "solana-devnet"
    ? "solana-devnet"
    : "solana-mainnet";
}

export function isMainnet(id: unknown): boolean {
  return toNetwork(id) === "solana-mainnet";
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
