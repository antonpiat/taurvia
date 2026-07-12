import { useMemo, useState } from "react";
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
import { ApiError, SendPreview, walletApi } from "@/lib/tauri";
import { localLogoForMint, withLocalLogo, WRAPPED_SOL } from "@/lib/tokenCatalog";
import { shortenAddress } from "@/lib/utils";

const SOL_MINT = "sol";

export function SendPage() {
  const { solBalance, tokens, refreshBalances, explorer, network } = useWallet();
  const [selectedMint, setSelectedMint] = useState(SOL_MINT);
  const [pickerOpen, setPickerOpen] = useState(false);
  const [to, setTo] = useState("");
  const [amount, setAmount] = useState("");
  const [password, setPassword] = useState("");
  const [preview, setPreview] = useState<SendPreview | null>(null);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [confirmError, setConfirmError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [successSignature, setSuccessSignature] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);

  const selectable = useMemo<DropdownToken[]>(() => {
    const options: DropdownToken[] = [
      {
        mint: SOL_MINT,
        symbol: "SOL",
        name: "Solana",
        logo_uri: localLogoForMint(WRAPPED_SOL),
        balanceUi: solBalance ?? 0,
      },
    ];
    for (const token of tokens) {
      const branded = withLocalLogo(token);
      options.push({
        mint: branded.mint,
        symbol: branded.symbol,
        name: branded.name,
        logo_uri: branded.logo_uri,
        balanceUi: branded.ui_amount,
      });
    }
    return options;
  }, [solBalance, tokens]);

  const selectedToken = selectable.find((token) => token.mint === selectedMint);
  const isSol = selectedMint === SOL_MINT;
  const tokenSymbol = selectedToken?.symbol ?? (isSol ? "SOL" : "token");

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
      const result = isSol
        ? await walletApi.previewSolSend(to, amountNum)
        : await walletApi.previewSplSend(selectedMint, to, amountNum);
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
      const result = isSol
        ? await walletApi.sendSol(password, to, amountNum)
        : await walletApi.sendSpl(password, selectedMint, to, amountNum);
      setSuccess(`Transaction confirmed: ${shortenAddress(result.signature, 8)}`);
      setSuccessSignature(result.signature);
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
      <PageHeader title="Send" description="Transfer SOL or SPL tokens securely." />

      <Card>
        <CardHeader>
          <CardTitle>Transfer details</CardTitle>
          <CardDescription>Review carefully before confirming.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
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
          />
          <div className="space-y-2">
            <Label htmlFor="to">Recipient address</Label>
            <Input
              id="to"
              value={to}
              onChange={(e) => setTo(e.target.value)}
              placeholder="Solana address"
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
              {successSignature && (
                <button
                  type="button"
                  className="mt-1 font-mono text-xs underline-offset-2 hover:underline"
                  onClick={() => void openUrl(txExplorerUrl(explorer, successSignature, { network }))}
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
                <span className="text-muted-foreground">Token</span>
                <span className="font-medium">{tokenSymbol}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">From</span>
                <span className="font-mono">{shortenAddress(preview.from, 6)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">To</span>
                <span className="font-mono">{shortenAddress(preview.to, 6)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Amount</span>
                <span>
                  {preview.amount} {tokenSymbol}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Estimated fee</span>
                <span>{preview.estimated_fee_sol.toFixed(6)} SOL</span>
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
