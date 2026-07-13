import { useEffect, useMemo, useRef, useState } from "react";
import { ChevronDown } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { withLocalLogo } from "@/lib/tokenCatalog";
import { cn, formatHiddenBalance, shortenAddress } from "@/lib/utils";
import { useWallet } from "@/context/WalletContext";
import type { TokenInfo } from "@/lib/tauri";
import { walletApi } from "@/lib/tauri";

export type DropdownToken = {
  mint: string;
  symbol: string;
  name: string;
  logo_uri: string | null;
  balanceUi?: number;
};

function TokenAvatar({
  symbol,
  logoUri,
  eager,
}: {
  symbol: string;
  logoUri: string | null;
  eager?: boolean;
}) {
  if (logoUri) {
    return (
      <img
        src={logoUri}
        alt=""
        className="h-8 w-8 shrink-0 rounded-full border border-border bg-background object-cover"
        loading={eager ? "eager" : "lazy"}
        decoding="async"
      />
    );
  }
  return (
    <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full border border-border bg-secondary text-xs font-semibold">
      {symbol.slice(0, 2).toUpperCase()}
    </div>
  );
}

function matchesQuery(token: DropdownToken, query: string): boolean {
  if (!query) return true;
  const q = query.toLowerCase();
  return (
    token.symbol.toLowerCase().includes(q) ||
    token.name.toLowerCase().includes(q) ||
    token.mint.toLowerCase().includes(q)
  );
}

export function TokenDropdown({
  label,
  token,
  tokens,
  selectedMint,
  placeholder,
  open,
  onOpenChange,
  onSelect,
  onAddToken,
  enableRemoteSearch = false,
}: {
  label: string;
  token: DropdownToken | undefined;
  tokens: DropdownToken[];
  selectedMint: string;
  placeholder: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSelect: (mint: string) => void;
  /** Persist + add a remote search hit to the local list. */
  onAddToken?: (info: TokenInfo) => void;
  enableRemoteSearch?: boolean;
}) {
  const { hideBalances } = useWallet();
  const rootRef = useRef<HTMLDivElement>(null);
  const [query, setQuery] = useState("");
  const [remote, setRemote] = useState<TokenInfo[]>([]);
  const [searching, setSearching] = useState(false);
  const [searchError, setSearchError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) {
      setQuery("");
      setRemote([]);
      setSearching(false);
      setSearchError(null);
    }
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const onPointerDown = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) {
        onOpenChange(false);
      }
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onOpenChange(false);
    };
    document.addEventListener("mousedown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("mousedown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [open, onOpenChange]);

  const localHits = useMemo(
    () => tokens.filter((t) => matchesQuery(t, query.trim())),
    [tokens, query],
  );

  useEffect(() => {
    if (!open || !enableRemoteSearch) return;
    const q = query.trim();
    if (!q) {
      setRemote([]);
      setSearching(false);
      setSearchError(null);
      return;
    }

    let cancelled = false;
    setSearching(true);
    setSearchError(null);
    const timer = window.setTimeout(() => {
      void (async () => {
        try {
          const results = await walletApi.searchTokens(q);
          if (cancelled) return;
          const localMints = new Set(tokens.map((t) => t.mint));
          setRemote(results.filter((r) => !localMints.has(r.mint)).map(withLocalLogo));
          setSearchError(null);
        } catch (err) {
          if (!cancelled) {
            setRemote([]);
            const message =
              err && typeof err === "object" && "message" in err
                ? String((err as { message?: unknown }).message ?? "")
                : "";
            setSearchError(message || "Token search failed");
          }
        } finally {
          if (!cancelled) setSearching(false);
        }
      })();
    }, 200);

    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [query, open, enableRemoteSearch, tokens]);

  const displayToken = token ? withLocalLogo(token) : undefined;

  return (
    <div className="space-y-2" ref={rootRef}>
      <Label>{label}</Label>
      <div className="relative">
        <button
          type="button"
          aria-haspopup="listbox"
          aria-expanded={open}
          onClick={() => onOpenChange(!open)}
          className={cn(
            "flex h-14 w-full items-center justify-between rounded-md border border-input bg-background px-3 text-left transition-colors",
            "hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
            open && "ring-2 ring-ring",
          )}
        >
          {displayToken ? (
            <span className="flex min-w-0 items-center gap-3">
              <TokenAvatar
                symbol={displayToken.symbol}
                logoUri={displayToken.logo_uri}
                eager
              />
              <span className="min-w-0">
                <span className="block text-base font-semibold leading-tight">
                  {displayToken.symbol}
                </span>
                <span className="block truncate text-xs text-muted-foreground">
                  {displayToken.name}
                </span>
              </span>
            </span>
          ) : (
            <span className="text-muted-foreground">{placeholder}</span>
          )}
          <ChevronDown
            className={cn(
              "h-4 w-4 shrink-0 text-muted-foreground transition-transform",
              open && "rotate-180",
            )}
          />
        </button>

        {open && (
          <div
            role="listbox"
            className="absolute left-0 right-0 top-[calc(100%+0.35rem)] z-30 max-h-80 overflow-y-auto rounded-md border border-border bg-card p-1 shadow-lg"
          >
            {enableRemoteSearch && (
              <div className="sticky top-0 z-10 bg-card p-1 pb-1.5">
                <Input
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  placeholder="Search symbol, name, or mint"
                  autoComplete="off"
                  spellCheck={false}
                  autoFocus
                  className="h-9"
                />
              </div>
            )}

            {searchError && (
              <p className="px-2.5 py-2 text-sm text-destructive">{searchError}</p>
            )}

            {localHits.length === 0 && remote.length === 0 && !searching ? (
              <p className="px-2.5 py-3 text-sm text-muted-foreground">
                {query.trim()
                  ? searchError
                    ? "Could not load remote results."
                    : "No tokens found."
                  : "No tokens available."}
              </p>
            ) : (
              <>
                {localHits.map((option) => {
                  const row = withLocalLogo(option);
                  const selected = row.mint === selectedMint;
                  return (
                    <button
                      key={row.mint}
                      type="button"
                      role="option"
                      aria-selected={selected}
                      onClick={() => onSelect(row.mint)}
                      className={cn(
                        "flex w-full items-center justify-between rounded-md px-2.5 py-2 text-left transition-colors",
                        selected ? "bg-primary/15" : "hover:bg-accent/50",
                      )}
                    >
                      <span className="flex min-w-0 items-center gap-3">
                        <TokenAvatar symbol={row.symbol} logoUri={row.logo_uri} />
                        <span className="min-w-0">
                          <span className="block font-semibold">{row.symbol}</span>
                          <span className="block truncate text-xs text-muted-foreground">
                            {row.name}
                          </span>
                        </span>
                      </span>
                      <span className="text-right text-xs text-muted-foreground">
                        {row.balanceUi !== undefined ? (
                          <span className="block font-mono text-foreground">
                            {formatHiddenBalance(hideBalances, String(row.balanceUi))}
                          </span>
                        ) : null}
                        <span className="font-mono">{shortenAddress(row.mint)}</span>
                      </span>
                    </button>
                  );
                })}

                {searching && (
                  <p className="px-2.5 py-2 text-xs text-muted-foreground">Searching…</p>
                )}

                {remote.length > 0 && (
                  <>
                    <p className="px-2.5 pt-2 text-[11px] font-medium uppercase tracking-wide text-muted-foreground">
                      More results
                    </p>
                    {remote.map((info) => {
                      const row = withLocalLogo(info);
                      return (
                        <button
                          key={row.mint}
                          type="button"
                          role="option"
                          aria-selected={false}
                          onClick={() => {
                            onAddToken?.(row);
                            onSelect(row.mint);
                          }}
                          className="flex w-full items-center justify-between rounded-md px-2.5 py-2 text-left transition-colors hover:bg-accent/50"
                        >
                          <span className="flex min-w-0 items-center gap-3">
                            <TokenAvatar symbol={row.symbol} logoUri={row.logo_uri} />
                            <span className="min-w-0">
                              <span className="block font-semibold">{row.symbol}</span>
                              <span className="block truncate text-xs text-muted-foreground">
                                {row.name}
                              </span>
                            </span>
                          </span>
                          <span className="font-mono text-xs text-muted-foreground">
                            {shortenAddress(row.mint)}
                          </span>
                        </button>
                      );
                    })}
                  </>
                )}
              </>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
