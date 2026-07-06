import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/misc";
import { ActivityItem, ApiError, walletApi } from "@/lib/tauri";
import { shortenAddress } from "@/lib/utils";

export function ActivityPage() {
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
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-semibold">Activity</h1>
        <p className="text-muted-foreground">Recent on-chain transactions for your wallet.</p>
      </div>

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
              className="flex items-start justify-between rounded-lg border border-border bg-background/50 px-4 py-3"
            >
              <div>
                <p className="font-medium">{item.description}</p>
                <p className="font-mono text-xs text-muted-foreground">{shortenAddress(item.signature, 8)}</p>
                {item.timestamp && (
                  <p className="text-xs text-muted-foreground">
                    {new Date(item.timestamp * 1000).toLocaleString()}
                  </p>
                )}
              </div>
              <div className="text-right">
                <Badge className={item.status === "confirmed" ? "text-primary" : "text-destructive"}>
                  {item.status}
                </Badge>
                {item.amount_sol !== null && (
                  <p className="mt-2 text-sm font-mono">{item.amount_sol.toFixed(6)} SOL</p>
                )}
              </div>
            </div>
          ))}
        </CardContent>
      </Card>
    </div>
  );
}
