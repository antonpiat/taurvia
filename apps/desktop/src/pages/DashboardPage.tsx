import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/misc";
import { useWallet } from "@/context/WalletContext";
import { formatSol, formatUsd, shortenAddress } from "@/lib/utils";
import { Copy, RefreshCw } from "lucide-react";

function TokenAvatar({ symbol, logoUri }: { symbol: string; logoUri: string | null }) {
  if (logoUri) {
    return (
      <img
        src={logoUri}
        alt={symbol}
        className="h-9 w-9 rounded-full border border-border bg-background object-cover"
        loading="lazy"
      />
    );
  }
  return (
    <div className="flex h-9 w-9 items-center justify-center rounded-full border border-border bg-secondary text-xs font-semibold">
      {symbol.slice(0, 2).toUpperCase()}
    </div>
  );
}

export function DashboardPage() {
  const {
    solBalance,
    solPriceUsd,
    solValueUsd,
    totalPortfolioUsd,
    tokens,
    publicKey,
    refresh,
  } = useWallet();
  const [refreshing, setRefreshing] = useState(false);

  const handleRefresh = async () => {
    setRefreshing(true);
    await refresh();
    setRefreshing(false);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-semibold">Dashboard</h1>
          <p className="text-muted-foreground">Portfolio value and token balances.</p>
        </div>
        <Button variant="outline" onClick={handleRefresh} disabled={refreshing}>
          <RefreshCw className={`h-4 w-4 ${refreshing ? "animate-spin" : ""}`} />
          Refresh
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardDescription>Portfolio value</CardDescription>
          <CardTitle className="text-4xl">{formatUsd(totalPortfolioUsd)}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center justify-between rounded-lg border border-border bg-background/50 px-4 py-3">
            <div className="flex items-center gap-3">
              <TokenAvatar
                symbol="SOL"
                logoUri="https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png"
              />
              <div>
                <p className="font-medium">SOL</p>
                <p className="text-xs text-muted-foreground">
                  {solPriceUsd !== null ? `${formatUsd(solPriceUsd)} / SOL` : "Price unavailable"}
                </p>
              </div>
            </div>
            <div className="text-right">
              <p className="font-mono">{solBalance !== null ? formatSol(solBalance) : "—"}</p>
              <p className="text-sm text-muted-foreground">{formatUsd(solValueUsd)}</p>
            </div>
          </div>
          <div className="flex items-center justify-between">
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
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>SPL Tokens</CardTitle>
          <CardDescription>Balances with live USD prices when available.</CardDescription>
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
                <div className="flex items-center gap-3">
                  <TokenAvatar symbol={token.symbol} logoUri={token.logo_uri} />
                  <div>
                    <p className="font-medium">{token.symbol}</p>
                    <p className="text-xs text-muted-foreground">{token.name}</p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="font-mono">{token.ui_amount}</p>
                  <p className="text-sm text-muted-foreground">{formatUsd(token.value_usd)}</p>
                  <p className="text-xs text-muted-foreground">
                    {token.price_usd !== null ? `${formatUsd(token.price_usd)} / token` : "—"}
                  </p>
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
