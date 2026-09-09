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

/** Last-used picker: activated mainnets plus testnets of those families. */
export function lastUsedNetworkOptions(
  networks: NetworkInfo[],
  activatedIds: string[],
): NetworkInfo[] {
  const activated = new Set(activatedIds);
  const families = new Set(
    networks.filter((n) => activated.has(n.id)).map((n) => n.family),
  );
  return networks.filter(
    (n) =>
      n.enabled &&
      (activated.has(n.id) || (n.is_testnet && families.has(n.family))),
  );
}

export function canSwap(info: NetworkInfo | undefined): boolean {
  return Boolean(info?.features.swap && !info.is_testnet);
}

export function canSwapAny(enabledIds: string[], networks: NetworkInfo[]): boolean {
  return enabledIds.some((id) => {
    const info = networks.find((n) => n.id === id);
    return canSwap(info);
  });
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
    default:
      return "sol";
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
