import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/misc";
import { PageHeader } from "@/components/PageHeader";
import { TokenIcon } from "@/components/TokenIcon";
import { useWallet } from "@/context/WalletContext";
import { findNetwork } from "@/lib/network";
import { networkFamilyToChain, withLocalLogo } from "@/lib/tokenCatalog";
import { formatNative, formatUsdMaybeHidden, formatHiddenBalance, shortenAddress } from "@/lib/utils";
import { Copy, Eye, EyeOff, RefreshCw } from "lucide-react";

function Skeleton({ className }: { className?: string }) {
  return <div className={`animate-pulse rounded-md bg-secondary/80 ${className ?? ""}`} />;
}

export function DashboardPage() {
  const {
    chains,
    totalPortfolioUsd,
    publicKey,
    networks,
    refresh,
    balancesLoading,
    hideBalances,
    setHideBalances,
  } = useWallet();
  const [refreshing, setRefreshing] = useState(false);
  const [togglingHide, setTogglingHide] = useState(false);

  const handleRefresh = async () => {
    setRefreshing(true);
    await refresh();
    setRefreshing(false);
  };

  const busy = refreshing || balancesLoading;
  const showSkeleton = balancesLoading && chains.length === 0;

  return (
    <div className="space-y-4 sm:space-y-6">
      <PageHeader
        title="Dashboard"
        description={balancesLoading ? "Loading balances…" : "Portfolio by chain."}
        actions={
          <>
            <Button
              variant="outline"
              onClick={() => {
                setTogglingHide(true);
                void setHideBalances(!hideBalances).finally(() => setTogglingHide(false));
              }}
              disabled={togglingHide}
              className="flex-1 sm:flex-none"
              aria-pressed={hideBalances}
              aria-label={hideBalances ? "Show balances" : "Hide balances"}
            >
              {hideBalances ? <Eye className="h-4 w-4" /> : <EyeOff className="h-4 w-4" />}
              {hideBalances ? "Show" : "Hide"}
            </Button>
            <Button variant="outline" onClick={handleRefresh} disabled={busy} className="flex-1 sm:flex-none">
              <RefreshCw className={`h-4 w-4 ${busy ? "animate-spin" : ""}`} />
              Refresh
            </Button>
          </>
        }
      />

      <Card>
        <CardHeader className="space-y-1">
          <CardDescription>Portfolio value</CardDescription>
          <CardTitle className="text-3xl sm:text-4xl">
            {showSkeleton ? (
              <Skeleton className="h-10 w-40" />
            ) : (
              formatUsdMaybeHidden(hideBalances, totalPortfolioUsd)
            )}
          </CardTitle>
        </CardHeader>
      </Card>

      {showSkeleton
        ? Array.from({ length: 2 }).map((_, i) => (
            <Card key={i}>
              <CardContent className="space-y-3 pt-6">
                <Skeleton className="h-9 w-28" />
                <Skeleton className="h-16 w-full" />
              </CardContent>
            </Card>
          ))
        : chains.map((chain) => {
            const info = findNetwork(networks, chain.network);
            const tokenChain = networkFamilyToChain(info?.family);
            const tokens = chain.tokens ?? [];
            return (
              <Card key={chain.network}>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2 text-lg">
                    <TokenIcon
                      symbol={chain.native_symbol}
                      mint={chain.native_symbol.toLowerCase()}
                      chain={tokenChain}
                      networkId={chain.network}
                      size={28}
                    />
                    {info?.name ?? chain.native_symbol}
                  </CardTitle>
                  <CardDescription className="flex items-center justify-between gap-2 font-mono text-xs">
                    <span className="truncate">
                      {chain.public_key ? shortenAddress(chain.public_key, 8) : "—"}
                    </span>
                    {chain.public_key && (
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => void navigator.clipboard.writeText(chain.public_key ?? "")}
                      >
                        <Copy className="h-3.5 w-3.5" />
                        Copy
                      </Button>
                    )}
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                  <div className="flex flex-col gap-3 rounded-lg border border-border bg-background/50 px-3 py-3 sm:flex-row sm:items-center sm:justify-between sm:px-4">
                    <div className="flex min-w-0 items-center gap-3">
                      <TokenIcon
                        symbol={chain.native_symbol}
                        mint={chain.native_symbol.toLowerCase()}
                        chain={tokenChain}
                        networkId={chain.network}
                        size={36}
                      />
                      <div className="min-w-0">
                        <p className="font-medium">{chain.native_symbol}</p>
                        <p className="text-xs text-muted-foreground">
                          {chain.native_price_usd !== null
                            ? `${formatUsdMaybeHidden(hideBalances, chain.native_price_usd)} / ${chain.native_symbol}`
                            : "Price unavailable"}
                        </p>
                      </div>
                    </div>
                    <div className="text-left sm:text-right">
                      <p className="font-mono">
                        {chain.native_balance !== null
                          ? formatHiddenBalance(
                              hideBalances,
                              formatNative(chain.native_balance, chain.native_symbol),
                            )
                          : "—"}
                      </p>
                      <p className="text-sm text-muted-foreground">
                        {formatUsdMaybeHidden(hideBalances, chain.native_value_usd)}
                      </p>
                    </div>
                  </div>
                  {tokens.map((token) => {
                    const branded = withLocalLogo(token, tokenChain);
                    return (
                      <div
                        key={token.mint}
                        className="flex flex-col gap-3 rounded-lg border border-border bg-background/50 px-3 py-3 sm:flex-row sm:items-center sm:justify-between sm:px-4"
                      >
                        <div className="flex min-w-0 items-center gap-3">
                          <TokenIcon
                            symbol={token.symbol}
                            mint={token.mint}
                            chain={tokenChain}
                            networkId={chain.network}
                            logoUri={branded.logo_uri}
                            size={36}
                          />
                          <div className="min-w-0">
                            <p className="font-medium">{token.symbol}</p>
                            <p className="truncate text-xs text-muted-foreground">{token.name}</p>
                          </div>
                        </div>
                        <div className="min-w-0 text-left sm:text-right">
                          <p className="font-mono">
                            {formatHiddenBalance(hideBalances, String(token.ui_amount))}
                          </p>
                          <p className="text-sm text-muted-foreground">
                            {formatUsdMaybeHidden(hideBalances, token.value_usd)}
                          </p>
                          <Badge className="mt-1">{shortenAddress(token.mint)}</Badge>
                        </div>
                      </div>
                    );
                  })}
                </CardContent>
              </Card>
            );
          })}

      {!showSkeleton && chains.length === 0 && publicKey && (
        <p className="text-sm text-muted-foreground">No balances yet. Activate a network in Settings.</p>
      )}
    </div>
  );
}
