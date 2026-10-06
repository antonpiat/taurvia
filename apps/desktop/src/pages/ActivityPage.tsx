import { useEffect, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { NetworkPicker } from "@/components/NetworkPicker";
import { PageHeader } from "@/components/PageHeader";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/misc";
import { useWallet } from "@/context/WalletContext";
import { txExplorerUrl } from "@/lib/explorer";
import { ActivityItem, ApiError, walletApi } from "@/lib/tauri";
import { shortenAddress } from "@/lib/utils";

export function ActivityPage() {
  const {
    explorer,
    network,
    networkInfo,
    networks,
    enabledNetworks,
    changeNetwork,
    networkReady,
  } = useWallet();
  const [items, setItems] = useState<ActivityItem[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!networkReady) {
      setLoading(true);
      setItems([]);
      setError(null);
      return;
    }
    let cancelled = false;
    setLoading(true);
    setError(null);
    void (async () => {
      try {
        const activity = await walletApi.getActivity(20);
        if (cancelled) return;
        setItems(activity);
      } catch (err) {
        if (cancelled) return;
        const apiError = err as ApiError;
        setError(apiError.message ?? "Failed to load activity");
        setItems([]);
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [network, networkReady]);

  return (
    <div className="space-y-4 sm:space-y-6">
      <PageHeader
        title="Activity"
        description="Recent on-chain transactions for your wallet."
      />

      <Card>
        <CardHeader>
          <CardTitle>Transaction history</CardTitle>
          <CardDescription>
            Fetched from {networkInfo?.name ?? "the active network"}.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <NetworkPicker
            networks={networks}
            activatedIds={enabledNetworks}
            selected={network}
            onSelect={(id) => void changeNetwork(id)}
          />
          {loading && <p className="text-sm text-muted-foreground">Loading activity...</p>}
          {error && <p className="text-sm text-destructive">{error}</p>}
          {!loading && !error && items.length === 0 && (
            <p className="text-sm text-muted-foreground">No transactions found yet.</p>
          )}
          {items.map((item) => (
            <div
              key={item.txid}
              className="flex flex-col gap-3 rounded-lg border border-border bg-background/50 px-3 py-3 sm:flex-row sm:items-start sm:justify-between sm:px-4"
            >
              <div className="min-w-0">
                <p className="font-medium">{item.description}</p>
                <button
                  type="button"
                  className="font-mono text-xs text-primary underline-offset-2 hover:underline"
                  onClick={() =>
                    void openUrl(txExplorerUrl(explorer, item.txid, { network, info: networkInfo }))
                  }
                >
                  {shortenAddress(item.txid, 8)}
                </button>
                {item.timestamp && (
                  <p className="text-xs text-muted-foreground">
                    {new Date(item.timestamp * 1000).toLocaleString()}
                  </p>
                )}
              </div>
              <div className="text-left sm:text-right">
                <Badge className={item.status === "confirmed" ? "text-primary" : "text-destructive"}>
                  {item.status}
                </Badge>
                {item.amount !== null && (
                  <p className="mt-2 font-mono text-sm">
                    {item.amount.toFixed(6)} {item.amount_symbol ?? ""}
                  </p>
                )}
              </div>
            </div>
          ))}
        </CardContent>
      </Card>
    </div>
  );
}
