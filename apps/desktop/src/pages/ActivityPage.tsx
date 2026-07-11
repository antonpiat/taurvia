import { useEffect, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { PageHeader } from "@/components/PageHeader";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/misc";
import { useWallet } from "@/context/WalletContext";
import { txExplorerUrl } from "@/lib/explorer";
import { ActivityItem, ApiError, walletApi } from "@/lib/tauri";
import { shortenAddress } from "@/lib/utils";

export function ActivityPage() {
  const { explorer, network } = useWallet();
  const [items, setItems] = useState<ActivityItem[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    void (async () => {
      try {
        const activity = await walletApi.getActivity(20);
        setItems(activity);
      } catch (err) {
        const apiError = err as ApiError;
        setError(apiError.message ?? "Failed to load activity");
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  return (
    <div className="space-y-4 sm:space-y-6">
      <PageHeader
        title="Activity"
        description="Recent on-chain transactions for your wallet."
      />

      <Card>
        <CardHeader>
          <CardTitle>Transaction history</CardTitle>
          <CardDescription>Signatures fetched directly from Solana RPC.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {loading && <p className="text-sm text-muted-foreground">Loading activity...</p>}
          {error && <p className="text-sm text-destructive">{error}</p>}
          {!loading && !error && items.length === 0 && (
            <p className="text-sm text-muted-foreground">No transactions found yet.</p>
          )}
          {items.map((item) => (
            <div
              key={item.signature}
              className="flex flex-col gap-3 rounded-lg border border-border bg-background/50 px-3 py-3 sm:flex-row sm:items-start sm:justify-between sm:px-4"
            >
              <div className="min-w-0">
                <p className="font-medium">{item.description}</p>
                <button
                  type="button"
                  className="font-mono text-xs text-primary underline-offset-2 hover:underline"
                  onClick={() => void openUrl(txExplorerUrl(explorer, item.signature, { network }))}
                >
                  {shortenAddress(item.signature, 8)}
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
                {item.amount_sol !== null && (
                  <p className="mt-2 font-mono text-sm">{item.amount_sol.toFixed(6)} SOL</p>
                )}
              </div>
            </div>
          ))}
        </CardContent>
      </Card>
    </div>
  );
}
