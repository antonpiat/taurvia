import type { ExplorerKind } from "@/bindings";
import { networkCluster } from "@/lib/network";

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
  signature: string,
  options?: { network?: string | null },
): string {
  const cluster = networkCluster(options?.network);
  const clusterQuery =
    cluster && cluster !== "mainnet-beta" ? `?cluster=${cluster}` : "";

  switch (normalizeExplorer(explorer)) {
    case "solanaExplorer":
      return `https://explorer.solana.com/tx/${signature}${clusterQuery}`;
    case "solscan":
    default:
      return `https://solscan.io/tx/${signature}${clusterQuery}`;
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
