import type { ExplorerKind, NetworkInfo } from "@/bindings";

export function normalizeExplorer(value: unknown): ExplorerKind {
  switch (value) {
    case "solanaExplorer":
    case "solana-explorer":
    case "SolanaExplorer":
      return "solanaExplorer";
    case "solscan":
    case "Solscan":
    default:
      return "solscan";
  }
}

export function txExplorerUrl(
  explorer: ExplorerKind,
  txid: string,
  options?: { network?: string | null; info?: NetworkInfo | null },
): string {
  const info = options?.info;
  if (info && info.family !== "solana") {
    return info.explorer_tx.replace("{txid}", txid);
  }
  const clusterQuery =
    options?.network === "solana-devnet" || info?.id === "solana-devnet"
      ? "?cluster=devnet"
      : "";
  switch (normalizeExplorer(explorer)) {
    case "solanaExplorer":
      return `https://explorer.solana.com/tx/${txid}${clusterQuery}`;
    case "solscan":
    default:
      return `https://solscan.io/tx/${txid}${clusterQuery}`;
  }
}

export function explorerLabel(explorer: ExplorerKind): string {
  switch (normalizeExplorer(explorer)) {
    case "solanaExplorer":
      return "Solana Explorer";
    case "solscan":
    default:
      return "Solscan";
  }
}
