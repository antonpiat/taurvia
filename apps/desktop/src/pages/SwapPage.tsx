import { useMemo, useState } from "react";
import { ArrowDownUp, ChevronDown, ChevronRight } from "lucide-react";
import { TokenDropdown } from "@/components/TokenDropdown";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Alert } from "@/components/ui/misc";
import { useWallet } from "@/context/WalletContext";
import { ApiError, SwapQuote, TokenInfo, walletApi } from "@/lib/tauri";

const WRAPPED_SOL = "So11111111111111111111111111111111111111112";
const DEFAULT_SLIPPAGE_BPS = 50;
const SAME_TOKEN_ERROR = "Choose two different tokens to continue.";

const MAJOR_TOKENS: TokenInfo[] = [
  {
    mint: WRAPPED_SOL,
    symbol: "SOL",
    name: "Solana",
    decimals: 9,
    logo_uri:
      "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png",
  },
  {
    mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    symbol: "USDC",
    name: "USD Coin",
    decimals: 6,
    logo_uri:
      "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/logo.png",
  },
  {
    mint: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
    symbol: "USDT",
    name: "Tether USD",
    decimals: 6,
    logo_uri:
      "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB/logo.svg",
  },
  {
    mint: "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN",
    symbol: "JUP",
    name: "Jupiter",
    decimals: 6,
    logo_uri: "https://static.jup.ag/jup/icon.png",
  },
  {
    mint: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
    symbol: "BONK",
    name: "Bonk",
    decimals: 5,
    logo_uri: "https://arweave.net/hQiPZOsRZXGXBJd_82PhVdlM_hACsT_q6wqwf5cSY7I",
  },
];

type SelectableToken = TokenInfo & { balanceUi?: number };

function isLikelySolanaMint(value: string): boolean {
  return /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(value.trim());
}

function looksLikeMintSymbol(symbol: string | null | undefined): boolean {
  if (!symbol) return true;
  return symbol.includes("...") || symbol.length > 12;
}

export function SwapPage() {
  const { solBalance, tokens, refreshBalances } = useWallet();
  const [fromMint, setFromMint] = useState(WRAPPED_SOL);
  const [toMint, setToMint] = useState("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
  const [amount, setAmount] = useState("");
  const [slippageBps, setSlippageBps] = useState(DEFAULT_SLIPPAGE_BPS);
  const [slippageInput, setSlippageInput] = useState("0.5");
  const [pasteMint, setPasteMint] = useState("");
  const [extraTokens, setExtraTokens] = useState<TokenInfo[]>([]);
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [pickerSide, setPickerSide] = useState<"from" | "to" | null>(null);
  const [quote, setQuote] = useState<SwapQuote | null>(null);
  const [password, setPassword] = useState("");
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [confirmError, setConfirmError] = useState<string | null>(null);
  const [signature, setSignature] = useState<string | null>(null);

  const selectable = useMemo(() => {
    const map = new Map<string, SelectableToken>();
    map.set(WRAPPED_SOL, {
      ...MAJOR_TOKENS[0],
      balanceUi: solBalance ?? 0,
    });
    for (const token of tokens) {
      map.set(token.mint, {
        mint: token.mint,
        symbol: token.symbol,
        name: token.name,
        decimals: token.decimals,
        logo_uri: token.logo_uri,
        balanceUi: token.ui_amount,
      });
    }
    for (const major of MAJOR_TOKENS) {
      if (!map.has(major.mint)) map.set(major.mint, major);
    }
    for (const extra of extraTokens) {
      if (!map.has(extra.mint)) map.set(extra.mint, extra);
    }
    return Array.from(map.values());
  }, [extraTokens, solBalance, tokens]);

  const fromToken = selectable.find((token) => token.mint === fromMint);
  const toToken = selectable.find((token) => token.mint === toMint);
  const fromSymbol =
    fromToken?.symbol && !looksLikeMintSymbol(fromToken.symbol)
      ? fromToken.symbol
      : quote?.input_symbol && !looksLikeMintSymbol(quote.input_symbol)
        ? quote.input_symbol
        : fromToken?.symbol ?? quote?.input_symbol ?? "token";
  const toSymbol =
    toToken?.symbol && !looksLikeMintSymbol(toToken.symbol)
      ? toToken.symbol
      : quote?.output_symbol && !looksLikeMintSymbol(quote.output_symbol)
        ? quote.output_symbol
        : toToken?.symbol ?? quote?.output_symbol ?? "token";

  const amountUi = Number(amount);
  const amountValid = Number.isFinite(amountUi) && amountUi > 0;
  const sameToken = Boolean(fromMint && toMint && fromMint === toMint);
  const canQuote =
    Boolean(fromMint && toMint) && !sameToken && amountValid && !loading;

  const sameTokenError = sameToken ? SAME_TOKEN_ERROR : null;
  const displayError = sameTokenError ?? error;

  const fromBalance = fromToken?.balanceUi ?? 0;

  const clearBoardFeedback = () => {
    setError(null);
    setQuote(null);
    setSignature(null);
  };

  const setPercentAmount = (pct: number) => {
    if (!fromToken || fromBalance <= 0) {
      setAmount("");
      clearBoardFeedback();
      return;
    }
    const next = fromBalance * pct;
    const decimals = Math.min(fromToken.decimals, 9);
    setAmount(next.toFixed(decimals).replace(/\.?0+$/, "") || "0");
    clearBoardFeedback();
  };

  const handleSelectToken = (side: "from" | "to", mint: string) => {
    if (side === "from") {
      setFromMint(mint);
    } else {
      setToMint(mint);
    }
    clearBoardFeedback();
    setPickerSide(null);
  };

  const handleResolveMint = async () => {
    setError(null);
    const mint = pasteMint.trim();
    if (!isLikelySolanaMint(mint)) {
      setError("Enter a valid Solana mint address.");
      return;
    }
    setLoading(true);
    try {
      const info = await walletApi.resolveToken(mint);
      setExtraTokens((prev) =>
        prev.some((token) => token.mint === info.mint) ? prev : [...prev, info],
      );
      setToMint(info.mint);
      setPasteMint("");
      setQuote(null);
    } catch (err) {
      const apiError = err as ApiError;
      setError(apiError.message ?? "Could not find that token.");
    } finally {
      setLoading(false);
    }
  };

  const handleFlip = () => {
    setFromMint(toMint);
    setToMint(fromMint);
    clearBoardFeedback();
  };

  const handleSlippagePercentChange = (value: string) => {
    setSlippageInput(value);
    const pct = Number(value);
    if (!Number.isFinite(pct) || pct < 0) return;
    setSlippageBps(Math.round(pct * 100));
    clearBoardFeedback();
  };

  const handleQuote = async () => {
    setError(null);
    setSignature(null);
    if (sameToken) {
      setError(SAME_TOKEN_ERROR);
      return;
    }
    if (!amountValid) {
      setError("Enter a valid amount.");
      return;
    }
    setLoading(true);
    try {
      const next = await walletApi.previewSwapQuote(
        fromMint,
        toMint,
        amountUi,
        slippageBps,
      );
      setQuote(next);
    } catch (err) {
      const apiError = err as ApiError;
      setError(apiError.message ?? String(err));
      setQuote(null);
    } finally {
      setLoading(false);
    }
  };

  const handleExecute = async () => {
    setConfirmError(null);
    if (!quote) {
      setConfirmError("Get a quote before swapping.");
      return;
    }
    setLoading(true);
    try {
      const result = await walletApi.executeSwap(
        password,
        fromMint,
        toMint,
        amountUi,
        slippageBps,
      );
      setSignature(result.signature);
      setConfirmOpen(false);
      setPassword("");
      setConfirmError(null);
      setError(null);
      setQuote(null);
      await refreshBalances();
    } catch (err) {
      const apiError = err as ApiError;
      // Keep failures in the confirm dialog so they don't stick on the swap board.
      setConfirmError(apiError.message ?? String(err));
    } finally {
      setLoading(false);
    }
  };

  const slippagePercentLabel = `${(slippageBps / 100).toFixed(
    slippageBps % 100 === 0 ? 1 : 2,
  )}%`;

  return (
    <div className="mx-auto w-full max-w-xl space-y-4 sm:space-y-6">
      <div>
        <h1 className="text-2xl font-semibold sm:text-3xl">Swap</h1>
        <p className="text-sm text-muted-foreground sm:text-base">
          Exchange tokens at the current estimated rate.
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Swap</CardTitle>
          <CardDescription>Review the estimated rate before continuing.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <TokenDropdown
            label="From"
            token={fromToken}
            tokens={selectable}
            selectedMint={fromMint}
            placeholder="Select token"
            open={pickerSide === "from"}
            onOpenChange={(open) => setPickerSide(open ? "from" : null)}
            onSelect={(mint) => handleSelectToken("from", mint)}
          />

          <div className="flex justify-center">
            <Button type="button" variant="outline" size="sm" onClick={handleFlip}>
              <ArrowDownUp className="h-4 w-4" />
              Flip
            </Button>
          </div>

          <TokenDropdown
            label="To"
            token={toToken}
            tokens={selectable}
            selectedMint={toMint}
            placeholder="Select token"
            open={pickerSide === "to"}
            onOpenChange={(open) => setPickerSide(open ? "to" : null)}
            onSelect={(mint) => handleSelectToken("to", mint)}
          />

          <div className="space-y-2">
            <div className="flex items-center justify-between gap-2">
              <Label htmlFor="amount">
                Amount{fromToken ? ` (${fromToken.symbol})` : ""}
              </Label>
              {fromToken?.balanceUi !== undefined && (
                <span className="text-xs text-muted-foreground">
                  Balance: {fromToken.balanceUi}
                </span>
              )}
            </div>
            <Input
              id="amount"
              value={amount}
              onChange={(e) => {
                setAmount(e.target.value);
                clearBoardFeedback();
              }}
              placeholder="0.0"
              inputMode="decimal"
            />
            <div className="flex gap-2">
              {[
                { label: "25%", pct: 0.25 },
                { label: "50%", pct: 0.5 },
                { label: "Max", pct: 1 },
              ].map(({ label, pct }) => (
                <Button
                  key={label}
                  type="button"
                  variant="outline"
                  size="sm"
                  className="flex-1"
                  disabled={!fromToken || fromBalance <= 0}
                  onClick={() => setPercentAmount(pct)}
                >
                  {label}
                </Button>
              ))}
            </div>
          </div>

          <div className="rounded-md border border-border bg-background/40 px-3 py-2 text-sm text-muted-foreground">
            Slippage: {slippagePercentLabel}
          </div>

          <div className="rounded-md border border-border">
            <button
              type="button"
              className="flex w-full items-center justify-between px-3 py-2.5 text-sm font-medium"
              onClick={() => setAdvancedOpen((open) => !open)}
            >
              <span>Advanced settings</span>
              {advancedOpen ? (
                <ChevronDown className="h-4 w-4 text-muted-foreground" />
              ) : (
                <ChevronRight className="h-4 w-4 text-muted-foreground" />
              )}
            </button>
            {advancedOpen && (
              <div className="space-y-4 border-t border-border px-3 py-3">
                <div className="space-y-2">
                  <Label htmlFor="slippage-pct">Slippage tolerance (%)</Label>
                  <Input
                    id="slippage-pct"
                    value={slippageInput}
                    onChange={(e) => handleSlippagePercentChange(e.target.value)}
                    inputMode="decimal"
                    placeholder="0.5"
                  />
                  <p className="text-xs text-muted-foreground">
                    Default is 0.5%. Higher slippage can help swaps succeed in volatile markets.
                  </p>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="paste-mint">Custom token mint address</Label>
                  <p className="text-xs text-muted-foreground">
                    Use this only if your token does not appear in the list.
                  </p>
                  <div className="flex gap-2">
                    <Input
                      id="paste-mint"
                      value={pasteMint}
                      onChange={(e) => setPasteMint(e.target.value)}
                      placeholder="Solana mint address"
                      autoComplete="off"
                      spellCheck={false}
                    />
                    <Button
                      type="button"
                      variant="secondary"
                      disabled={loading || !pasteMint.trim()}
                      onClick={() => void handleResolveMint()}
                    >
                      Add
                    </Button>
                  </div>
                </div>
              </div>
            )}
          </div>

          {quote && (
            <Alert className="space-y-1">
              <p className="font-medium">
                ≈ {quote.out_amount_ui} {quote.output_symbol}
              </p>
              <p className="text-sm text-muted-foreground">
                Price impact:{" "}
                {quote.price_impact_pct === null
                  ? "—"
                  : `${quote.price_impact_pct.toFixed(4)}%`}
              </p>
              <p className="text-sm text-muted-foreground">
                Network fee: {quote.network_fee_sol.toFixed(6)} SOL
              </p>
              <p className="text-xs text-muted-foreground">
                {fromToken?.symbol ?? quote.input_symbol} →{" "}
                {toToken?.symbol ?? quote.output_symbol}
              </p>
            </Alert>
          )}

          {signature && (
            <Alert>
              <p className="text-sm font-medium">Swap confirmed</p>
              <p className="break-all font-mono text-xs text-muted-foreground">{signature}</p>
            </Alert>
          )}

          {displayError && (
            <Alert className="border-destructive/40 text-destructive">{displayError}</Alert>
          )}

          <div className="flex flex-col gap-3 sm:flex-row">
            <Button
              className="w-full sm:w-auto"
              onClick={() => void handleQuote()}
              disabled={!canQuote}
            >
              {loading && !confirmOpen ? "Fetching..." : "Get quote"}
            </Button>
            <Button
              className="w-full sm:w-auto"
              variant="secondary"
              disabled={!quote || loading || sameToken}
              onClick={() => {
                setError(null);
                setConfirmError(null);
                setConfirmOpen(true);
              }}
            >
              Confirm & Swap
            </Button>
          </div>
          <p className="text-xs text-muted-foreground">
            Swaps are quoted through Jupiter. Transactions are signed locally on your device.
          </p>
        </CardContent>
      </Card>

      <Dialog
        open={confirmOpen}
        onOpenChange={(open) => {
          setConfirmOpen(open);
          if (!open) {
            setPassword("");
            setConfirmError(null);
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Confirm swap</DialogTitle>
            <DialogDescription>
              Review the estimated rate before continuing. Enter your wallet password to sign.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-3">
            {quote && (
              <div className="space-y-2 rounded-md border border-border bg-background/40 px-3 py-3 text-sm">
                <div className="flex justify-between gap-3">
                  <span className="text-muted-foreground">You pay</span>
                  <span className="font-medium">
                    {amount} {fromSymbol}
                  </span>
                </div>
                <div className="flex justify-between gap-3">
                  <span className="text-muted-foreground">You receive</span>
                  <span className="font-medium">
                    ~{quote.out_amount_ui} {toSymbol}
                  </span>
                </div>
                <div className="flex justify-between gap-3">
                  <span className="text-muted-foreground">Route</span>
                  <span className="font-medium">
                    {fromSymbol} → {toSymbol}
                  </span>
                </div>
              </div>
            )}
            <div className="space-y-2">
              <Label htmlFor="swap-password">Password</Label>
              <Input
                id="swap-password"
                type="password"
                autoComplete="current-password"
                value={password}
                onChange={(e) => {
                  setPassword(e.target.value);
                  setConfirmError(null);
                }}
              />
            </div>
            {confirmError && (
              <Alert className="border-destructive/40 text-destructive">{confirmError}</Alert>
            )}
            <Button
              onClick={() => void handleExecute()}
              disabled={loading || !password || !quote}
            >
              {loading ? "Submitting..." : "Submit swap"}
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
