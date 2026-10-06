/**
 * Network identity helpers. Labels and features come from Rust `list_networks()`.
 */

import type { NetworkInfo } from "@/bindings";

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

/** Paired mainnet id for a testnet row (solana-devnet → solana-mainnet). */
export function pairedMainnetId(info: NetworkInfo): string {
  if (!info.is_testnet) return info.id;
  const prefix = info.id.split("-")[0];
  return `${prefix}-mainnet`;
}

export function pairedTestnet(
  networks: NetworkInfo[],
  mainnet: NetworkInfo,
): NetworkInfo | undefined {
  if (mainnet.is_testnet) return mainnet;
  return networks.find(
    (n) => n.enabled && n.is_testnet && pairedMainnetId(n) === mainnet.id,
  );
}

/** Stable UI order: SOL, BTC, ETH, BNB, POL, SUI, then anything else. */
const PICKER_FAMILY_ORDER = ["solana", "bitcoin", "ethereum", "bnb", "polygon", "sui"];

export function sortNetworkPickerOptions(options: NetworkInfo[]): NetworkInfo[] {
  const rank = (id: string) => {
    const prefix = id.split("-")[0] ?? id;
    const i = PICKER_FAMILY_ORDER.indexOf(prefix);
    return i === -1 ? PICKER_FAMILY_ORDER.length : i;
  };
  return [...options].sort((a, b) => {
    const d = rank(a.id) - rank(b.id);
    if (d !== 0) return d;
    return Number(a.is_testnet) - Number(b.is_testnet);
  });
}

export function lastUsedNetworkOptions(
  networks: NetworkInfo[],
  activatedIds: string[],
  selected?: string,
  developerMode = false,
): NetworkInfo[] {
  const activated = new Set(activatedIds.map((id) => normalizeNetworkId(id)));
  const mains = sortNetworkPickerOptions(
    networks.filter((n) => n.enabled && !n.is_testnet && activated.has(n.id)),
  );
  const options = developerMode
    ? mains.map((main) => pairedTestnet(networks, main) ?? main)
    : mains;
  const selectedInfo = findNetwork(networks, selected);
  if (
    selectedInfo?.enabled &&
    selectedInfo.is_testnet === developerMode &&
    !options.some((n) => n.id === selectedInfo.id)
  ) {
    return sortNetworkPickerOptions([...options, selectedInfo]);
  }
  return options;
}

function canSwap(info: NetworkInfo | undefined): boolean {
  return Boolean(info?.features.swap && !info.is_testnet);
}

export function canSwapAny(
  enabledIds: string[],
  networks: NetworkInfo[],
  developerMode = false,
): boolean {
  if (developerMode) return false;
  return enabledIds.some((id) => canSwap(networks.find((n) => n.id === id)));
}

export function networkShortLabel(info: NetworkInfo | undefined, id?: unknown): string {
  if (info) {
    return info.name;
  }
  return normalizeNetworkId(id);
}

export function nativeAssetId(info: NetworkInfo | undefined): string {
  if (info?.native_symbol) {
    return info.native_symbol.toLowerCase();
  }
  return "sol";
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
