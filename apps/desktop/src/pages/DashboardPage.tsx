import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/misc";
import { useWallet } from "@/context/WalletContext";
import { formatSol, shortenAddress } from "@/lib/utils";
import { Copy, RefreshCw } from "lucide-react";

export function DashboardPage() {
  const { solBalance, tokens, publicKey, refresh } = useWallet();
  const [refreshing, setRefreshing] = useState(false);

  const handleRefresh = async () => {
    setRefreshing(true);
    await refresh();
    setRefreshing(false);
  };

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-semibold">Dashboard</h1>
          <p className="text-muted-foreground">Your Solana balances at a glance.</p>
        </div>
        <Button variant="outline" onClick={handleRefresh} disabled={refreshing}>
          <RefreshCw className={`h-4 w-4 ${refreshing ? "animate-spin" : ""}`} />
          Refresh
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardDescription>Total balance</CardDescription>
          <CardTitle className="text-4xl">{solBalance !== null ? formatSol(solBalance) : "—"}</CardTitle>
        </CardHeader>
        <CardContent className="flex items-center justify-between">
          <div className="font-mono text-sm text-muted-foreground">
            {publicKey ? shortenAddress(publicKey, 8) : "No address"}
          </div>
          {publicKey && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => navigator.clipboard.writeText(publicKey)}
            >
              <Copy className="h-4 w-4" />
              Copy
            </Button>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>SPL Tokens</CardTitle>
          <CardDescription>Basic token balances on Solana mainnet.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {tokens.length === 0 ? (
            <p className="text-sm text-muted-foreground">No SPL token balances found.</p>
          ) : (
            tokens.map((token) => (
              <div
                key={token.mint}
                className="flex items-center justify-between rounded-lg border border-border bg-background/50 px-4 py-3"
              >
                <div>
                  <p className="font-medium">{token.symbol}</p>
                  <p className="text-xs text-muted-foreground">{token.name}</p>
                </div>
                <div className="text-right">
                  <p className="font-mono">{token.ui_amount}</p>
                  <Badge className="mt-1">{shortenAddress(token.mint)}</Badge>
                </div>
              </div>
            ))
          )}
        </CardContent>
      </Card>
    </div>
  );
}
