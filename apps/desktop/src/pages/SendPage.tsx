import { useEffect, useMemo, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { TokenDropdown, type DropdownToken } from "@/components/TokenDropdown";
import { PageHeader } from "@/components/PageHeader";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Alert } from "@/components/ui/misc";
import { useWallet } from "@/context/WalletContext";
import { txExplorerUrl } from "@/lib/explorer";
import { nativeAssetId } from "@/lib/network";
import { ApiError, SendPreview, walletApi } from "@/lib/tauri";
import { localLogoForAsset, networkFamilyToChain, withLocalLogo } from "@/lib/tokenCatalog";
import { shortenAddress } from "@/lib/utils";

export function SendPage() {
  const {
    nativeBalance,
    nativeSymbol,
    tokens,
    refreshBalances,
    explorer,
    network,
    networkInfo,
    networks,
    enabledNetworks,
    changeNetwork,
  } = useWallet();
  const nativeMint = nativeAssetId(networkInfo?.family);
  const tokenChain = networkFamilyToChain(networkInfo?.family);
  const [selectedMint, setSelectedMint] = useState(nativeMint);
  const [pickerOpen, setPickerOpen] = useState(false);
  const switchable = networks.filter(
    (n) => n.enabled && enabledNetworks.includes(n.id) && !n.is_testnet,
  );

  useEffect(() => {
    setSelectedMint(nativeMint);
  }, [nativeMint]);
  const [to, setTo] = useState("");
  const [amount, setAmount] = useState("");
  const [password, setPassword] = useState("");
  const [preview, setPreview] = useState<SendPreview | null>(null);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [confirmError, setConfirmError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [successTxid, setSuccessTxid] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);

  const showTokens = Boolean(networkInfo?.features.tokens);

  const selectable = useMemo<DropdownToken[]>(() => {
    const options: DropdownToken[] = [
      {
        mint: nativeMint,
        symbol: nativeSymbol,
        name: networkInfo?.name ?? nativeSymbol,
        logo_uri: localLogoForAsset(tokenChain, nativeMint, nativeSymbol),
        balanceUi: nativeBalance ?? 0,
        chain: tokenChain,
      },
    ];
    if (showTokens) {
      for (const token of tokens) {
        const branded = withLocalLogo(token, tokenChain);
        options.push({
          mint: branded.mint,
          symbol: branded.symbol,
          name: branded.name,
          logo_uri: branded.logo_uri,
          balanceUi: branded.ui_amount,
          chain: tokenChain,
        });
      }
    }
    return options;
  }, [nativeBalance, nativeMint, nativeSymbol, networkInfo?.name, showTokens, tokens]);

  const selectedToken = selectable.find((token) => token.mint === selectedMint) ?? selectable[0];
  const isNative = selectedToken?.mint === nativeMint;
  const tokenSymbol = selectedToken?.symbol ?? nativeSymbol;
  const recipientPlaceholder =
    networkInfo?.family === "evm" || networkInfo?.family === "sui"
      ? "0x…"
      : networkInfo?.family === "bitcoin"
        ? networkInfo.is_testnet
          ? "tb1q…"
          : "bc1q…"
        : "Solana address";

  const handlePreview = async () => {
    setError(null);
    setConfirmError(null);
    setSuccess(null);
    setLoading(true);
    try {
      const amountNum = Number(amount);
      if (!Number.isFinite(amountNum) || amountNum <= 0) {
        throw new Error("Enter a valid amount");
      }
      const asset = isNative ? nativeMint : selectedMint;
      const result = await walletApi.previewSend(to, amountNum, asset);
      setPreview(result);
      setConfirmOpen(true);
    } catch (err) {
      const apiError = err as ApiError;
      setError(apiError.message ?? "Failed to prepare transaction");
    } finally {
      setLoading(false);
    }
  };

  const handleSend = async () => {
    if (!preview) return;
    setLoading(true);
    setConfirmError(null);
    try {
      const amountNum = Number(amount);
      const asset = isNative ? nativeMint : selectedMint;
      const result = await walletApi.sendTransfer(password, to, amountNum, asset);
      setSuccess(`Transaction submitted: ${shortenAddress(result.txid, 8)}`);
      setSuccessTxid(result.txid);
      setConfirmOpen(false);
      setPassword("");
      setConfirmError(null);
      setAmount("");
      setTo("");
      setPreview(null);
      await refreshBalances();
    } catch (err) {
      const apiError = err as ApiError;
      setConfirmError(apiError.message ?? "Transaction failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-4 sm:space-y-6">
      <PageHeader
        title="Send"
        description={`Transfer ${nativeSymbol}${showTokens ? " or tokens" : ""} securely.`}
      />

      <Card>
        <CardHeader>
          <CardTitle>Transfer details</CardTitle>
          <CardDescription>Review carefully before confirming.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {switchable.length > 1 && (
            <div className="flex flex-wrap gap-1">
              {switchable.map((n) => (
                <Button
                  key={n.id}
                  type="button"
                  size="sm"
                  variant={n.id === network ? "default" : "outline"}
                  onClick={() => void changeNetwork(n.id)}
                >
                  {n.name}
                </Button>
              ))}
            </div>
          )}
          {showTokens ? (
            <TokenDropdown
              label="Token"
              token={selectedToken}
              tokens={selectable}
              selectedMint={selectedMint}
              placeholder="Select token"
              open={pickerOpen}
              onOpenChange={setPickerOpen}
              onSelect={(mint) => {
                setSelectedMint(mint);
                setPickerOpen(false);
                setError(null);
                setSuccess(null);
              }}
              chain={tokenChain}
            />
          ) : (
            <div className="space-y-2">
              <Label>Asset</Label>
              <p className="font-medium">{nativeSymbol}</p>
            </div>
          )}
          <div className="space-y-2">
            <Label htmlFor="to">Recipient address</Label>
            <Input
              id="to"
              value={to}
              onChange={(e) => setTo(e.target.value)}
              placeholder={recipientPlaceholder}
            />
          </div>
          <div className="space-y-2">
            <div className="flex items-center justify-between gap-2">
              <Label htmlFor="amount">Amount ({tokenSymbol})</Label>
              {selectedToken?.balanceUi !== undefined && (
                <span className="text-xs text-muted-foreground">
                  Balance: {selectedToken.balanceUi}
                </span>
              )}
            </div>
            <Input
              id="amount"
              type="number"
              min="0"
              step="any"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
            />
          </div>
          {error && <Alert className="border-destructive/40 text-destructive">{error}</Alert>}
          {success && (
            <Alert className="border-primary/40 text-primary">
              <p>{success}</p>
              {successTxid && (
                <button
                  type="button"
                  className="mt-1 font-mono text-xs underline-offset-2 hover:underline"
                  onClick={() =>
                    void openUrl(txExplorerUrl(explorer, successTxid, { network, info: networkInfo }))
                  }
                >
                  View on explorer
                </button>
              )}
            </Alert>
          )}
          <Button className="w-full" onClick={handlePreview} disabled={loading || !to || !amount}>
            Review transaction
          </Button>
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
            <DialogTitle>Confirm transaction</DialogTitle>
            <DialogDescription>
              Verify the details and enter your password to sign.
            </DialogDescription>
          </DialogHeader>
          {preview && (
            <div className="space-y-3 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Network</span>
                <span className="font-medium">{preview.network_name}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Token</span>
                <span className="font-medium">{preview.token || tokenSymbol}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">From</span>
                <span className="font-mono">{shortenAddress(preview.from, 6)}</span>
              </div>
              <div className="flex flex-col gap-1">
                <span className="text-muted-foreground">To</span>
                <span className="break-all font-mono text-xs">{preview.to}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Amount</span>
                <span>
                  {preview.amount} {tokenSymbol}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Estimated fee</span>
                <span>
                  {preview.estimated_fee.toFixed(nativeSymbol === "BTC" ? 8 : 6)} {preview.fee_symbol}
                </span>
              </div>
              {preview.creates_token_account && (
                <Alert>A token account will be created for this asset.</Alert>
              )}
              <button
                type="button"
                className="text-xs text-primary"
                onClick={() => setShowAdvanced((v) => !v)}
              >
                {showAdvanced ? "Hide" : "Show"} advanced details
              </button>
              {showAdvanced && (
                <pre className="max-h-32 overflow-auto rounded-md bg-background p-3 text-xs text-muted-foreground">
                  {JSON.stringify(preview, null, 2)}
                </pre>
              )}
              <div className="space-y-2">
                <Label htmlFor="send-password">Password</Label>
                <Input
                  id="send-password"
                  type="password"
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
              <Button className="w-full" onClick={handleSend} disabled={loading || !password}>
                {loading ? "Signing..." : "Sign and send"}
              </Button>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
