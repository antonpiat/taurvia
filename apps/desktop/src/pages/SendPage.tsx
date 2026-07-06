import { useState } from "react";
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
import { ApiError, SendPreview, walletApi } from "@/lib/tauri";
import { shortenAddress } from "@/lib/utils";

type TokenOption = { type: "sol" } | { type: "spl"; mint: string; symbol: string };

export function SendPage() {
  const { tokens } = useWallet();
  const [token, setToken] = useState<TokenOption>({ type: "sol" });
  const [to, setTo] = useState("");
  const [amount, setAmount] = useState("");
  const [password, setPassword] = useState("");
  const [preview, setPreview] = useState<SendPreview | null>(null);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);

  const handlePreview = async () => {
    setError(null);
    setLoading(true);
    try {
      const amountNum = Number(amount);
      const result =
        token.type === "sol"
          ? await walletApi.previewSolSend(to, amountNum)
          : await walletApi.previewSplSend(token.mint, to, amountNum);
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
    setError(null);
    try {
      const amountNum = Number(amount);
      const result =
        token.type === "sol"
          ? await walletApi.sendSol(password, to, amountNum)
          : await walletApi.sendSpl(password, token.mint, to, amountNum);
      setSuccess(`Transaction confirmed: ${shortenAddress(result.signature, 8)}`);
      setConfirmOpen(false);
      setPassword("");
      setAmount("");
      setTo("");
    } catch (err) {
      const apiError = err as ApiError;
      setError(apiError.message ?? "Transaction failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <div>
        <h1 className="text-3xl font-semibold">Send</h1>
        <p className="text-muted-foreground">Transfer SOL or SPL tokens securely.</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Transfer details</CardTitle>
          <CardDescription>Review carefully before confirming.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="token">Token</Label>
            <select
              id="token"
              className="flex h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
              value={token.type === "sol" ? "sol" : token.mint}
              onChange={(e) => {
                const value = e.target.value;
                if (value === "sol") setToken({ type: "sol" });
                else {
                  const selected = tokens.find((t) => t.mint === value);
                  if (selected) setToken({ type: "spl", mint: selected.mint, symbol: selected.symbol });
                }
              }}
            >
              <option value="sol">SOL</option>
              {tokens.map((t) => (
                <option key={t.mint} value={t.mint}>
                  {t.symbol}
                </option>
              ))}
            </select>
          </div>
          <div className="space-y-2">
            <Label htmlFor="to">Recipient address</Label>
            <Input id="to" value={to} onChange={(e) => setTo(e.target.value)} placeholder="Solana address" />
          </div>
          <div className="space-y-2">
            <Label htmlFor="amount">Amount</Label>
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
          {success && <Alert className="border-primary/40 text-primary">{success}</Alert>}
          <Button className="w-full" onClick={handlePreview} disabled={loading || !to || !amount}>
            Review transaction
          </Button>
        </CardContent>
      </Card>

      <Dialog open={confirmOpen} onOpenChange={setConfirmOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Confirm transaction</DialogTitle>
            <DialogDescription>Verify the details and enter your password to sign.</DialogDescription>
          </DialogHeader>
          {preview && (
            <div className="space-y-3 text-sm">
              <div className="flex justify-between"><span className="text-muted-foreground">From</span><span className="font-mono">{shortenAddress(preview.from, 6)}</span></div>
              <div className="flex justify-between"><span className="text-muted-foreground">To</span><span className="font-mono">{shortenAddress(preview.to, 6)}</span></div>
              <div className="flex justify-between"><span className="text-muted-foreground">Amount</span><span>{preview.amount}</span></div>
              <div className="flex justify-between"><span className="text-muted-foreground">Estimated fee</span><span>{preview.estimated_fee_sol.toFixed(6)} SOL</span></div>
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
                  onChange={(e) => setPassword(e.target.value)}
                />
              </div>
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
