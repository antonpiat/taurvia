/**
 * Network identity helpers. Labels and features come from Rust `list_networks()`.
 */

import type { ChainFamily, NetworkInfo } from "@/bindings";

export const DEFAULT_NETWORK_ID = "solana-mainnet";

export function normalizeNetworkId(value: unknown): string {
  if (typeof value === "string" && value.trim() !== "") {
    if (value.trim() === "devnet") return "solana-devnet";
    if (value.trim() === "mainnet") return "solana-mainnet";
    return value.trim();
  }
  return DEFAULT_NETWORK_ID;
}

export function findNetwork(
  networks: NetworkInfo[],
  id: unknown,
): NetworkInfo | undefined {
  const normalized = normalizeNetworkId(id);
  return networks.find((n) => n.id === normalized);
}

/** Last-used picker: activated mainnets plus testnets of those paired mainnets. */
export function pairedMainnetId(info: NetworkInfo): string {
  if (!info.is_testnet) return info.id;
  switch (info.id) {
    case "solana-devnet":
      return "solana-mainnet";
    case "ethereum-sepolia":
      return "ethereum-mainnet";
    case "bnb-testnet":
      return "bnb-mainnet";
    case "bitcoin-testnet":
      return "bitcoin-mainnet";
    case "sui-testnet":
      return "sui-mainnet";
    case "polygon-amoy":
      return "polygon-mainnet";
    case "base-sepolia":
      return "base-mainnet";
    default:
      return info.id.replace(/-(testnet|sepolia|devnet|amoy)$/, "-mainnet");
  }
}

export function lastUsedNetworkOptions(
  networks: NetworkInfo[],
  activatedIds: string[],
): NetworkInfo[] {
  const activated = new Set(activatedIds);
  return networks.filter(
    (n) =>
      n.enabled &&
      (activated.has(n.id) || (n.is_testnet && activated.has(pairedMainnetId(n)))),
  );
}

function canSwap(info: NetworkInfo | undefined): boolean {
  return Boolean(info?.features.swap && !info.is_testnet);
}

export function canSwapAny(enabledIds: string[], networks: NetworkInfo[]): boolean {
  return enabledIds.some((id) => canSwap(networks.find((n) => n.id === id)));
}

export function networkShortLabel(info: NetworkInfo | undefined, id?: unknown): string {
  if (info) {
    return info.name;
  }
  return normalizeNetworkId(id);
}

export function nativeAssetId(family: ChainFamily | undefined): string {
  switch (family) {
    case "evm":
      return "eth";
    case "bitcoin":
      return "btc";
    case "sui":
      return "sui";
    default:
      return "sol";
  }
}

export function recipientAddressPlaceholder(info: NetworkInfo | undefined): string {
  switch (info?.family) {
    case "evm":
    case "sui":
      return "0x…";
    case "bitcoin":
      return info.is_testnet ? "tb1q…" : "bc1q…";
    default:
      return "Solana address";
  }
}

export function receiveWarning(info: NetworkInfo | undefined): string {
  const name = info?.name ?? "this network";
  return `Only send ${name} assets to this address.`;
}

export function familyLabel(family: ChainFamily): string {
  switch (family) {
    case "solana":
      return "Solana";
    case "evm":
      return "Ethereum";
    case "bitcoin":
      return "Bitcoin";
    case "sui":
      return "Sui";
    default:
      return family;
  }
}
