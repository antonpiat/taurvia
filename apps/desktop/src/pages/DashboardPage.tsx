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
        className="h-9 w-9 shrink-0 rounded-full border border-border bg-background object-cover"
        loading="lazy"
      />
    );
  }
  return (
    <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-border bg-secondary text-xs font-semibold">
      {symbol.slice(0, 2).toUpperCase()}
    </div>
  );
}

function Skeleton({ className }: { className?: string }) {
  return <div className={`animate-pulse rounded-md bg-secondary/80 ${className ?? ""}`} />;
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
    balancesLoading,
  } = useWallet();
  const [refreshing, setRefreshing] = useState(false);

  const handleRefresh = async () => {
    setRefreshing(true);
    await refresh();
    setRefreshing(false);
  };

  const busy = refreshing || balancesLoading;
  const showSkeleton = balancesLoading && solBalance === null;

  return (
    <div className="mx-auto w-full max-w-3xl space-y-4 sm:space-y-6">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div className="min-w-0">
          <h1 className="text-2xl font-semibold sm:text-3xl">Dashboard</h1>
          <p className="text-sm text-muted-foreground sm:text-base">
            {balancesLoading
              ? "Loading balances and token details…"
              : "Portfolio value and token balances."}
          </p>
        </div>
        <Button
          variant="outline"
          onClick={handleRefresh}
          disabled={busy}
          className="w-full shrink-0 sm:w-auto"
        >
          <RefreshCw className={`h-4 w-4 ${busy ? "animate-spin" : ""}`} />
          Refresh
        </Button>
      </div>

      <Card>
        <CardHeader className="space-y-1">
          <CardDescription>Portfolio value</CardDescription>
          <CardTitle className="text-3xl sm:text-4xl">
            {showSkeleton ? <Skeleton className="h-10 w-40" /> : formatUsd(totalPortfolioUsd)}
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex flex-col gap-3 rounded-lg border border-border bg-background/50 px-3 py-3 sm:flex-row sm:items-center sm:justify-between sm:px-4">
            <div className="flex min-w-0 items-center gap-3">
              <TokenAvatar
                symbol="SOL"
                logoUri="https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png"
              />
              <div className="min-w-0">
                <p className="font-medium">SOL</p>
                <p className="text-xs text-muted-foreground">
                  {showSkeleton
                    ? "Fetching price…"
                    : solPriceUsd !== null
                      ? `${formatUsd(solPriceUsd)} / SOL`
                      : "Price unavailable"}
                </p>
              </div>
            </div>
            <div className="text-left sm:text-right">
              {showSkeleton ? (
                <>
                  <Skeleton className="mb-1 h-5 w-28" />
                  <Skeleton className="h-4 w-20" />
                </>
              ) : (
                <>
                  <p className="font-mono">{solBalance !== null ? formatSol(solBalance) : "—"}</p>
                  <p className="text-sm text-muted-foreground">{formatUsd(solValueUsd)}</p>
                </>
              )}
            </div>
          </div>
          <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
            <div className="truncate font-mono text-sm text-muted-foreground">
              {publicKey ? shortenAddress(publicKey, 8) : "No address"}
            </div>
            {publicKey && (
              <Button
                variant="ghost"
                size="sm"
                className="w-full sm:w-auto"
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
          {showSkeleton ? (
            Array.from({ length: 3 }).map((_, index) => (
              <div
                key={index}
                className="flex items-center justify-between rounded-lg border border-border bg-background/50 px-3 py-3 sm:px-4"
              >
                <div className="flex items-center gap-3">
                  <Skeleton className="h-9 w-9 rounded-full" />
                  <div>
                    <Skeleton className="mb-1 h-4 w-16" />
                    <Skeleton className="h-3 w-24" />
                  </div>
                </div>
                <div className="text-right">
                  <Skeleton className="mb-1 ml-auto h-4 w-20" />
                  <Skeleton className="ml-auto h-3 w-14" />
                </div>
              </div>
            ))
          ) : tokens.length === 0 ? (
            <p className="text-sm text-muted-foreground">No SPL token balances found.</p>
          ) : (
            tokens.map((token) => (
              <div
                key={token.mint}
                className="flex flex-col gap-3 rounded-lg border border-border bg-background/50 px-3 py-3 sm:flex-row sm:items-center sm:justify-between sm:px-4"
              >
                <div className="flex min-w-0 items-center gap-3">
                  <TokenAvatar symbol={token.symbol} logoUri={token.logo_uri} />
                  <div className="min-w-0">
                    <p className="font-medium">{token.symbol}</p>
                    <p className="truncate text-xs text-muted-foreground">{token.name}</p>
                  </div>
                </div>
                <div className="min-w-0 text-left sm:text-right">
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
