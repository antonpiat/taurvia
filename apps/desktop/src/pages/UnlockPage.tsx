import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { BrandMark } from "@/components/BrandMark";
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
import { ApiError, walletApi } from "@/lib/tauri";
import { Checkbox } from "@/components/ui/checkbox";

export function UnlockPage() {
  const navigate = useNavigate();
  const { unlock, refresh } = useWallet();
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [resetOpen, setResetOpen] = useState(false);
  const [resetConfirm, setResetConfirm] = useState(false);
  const [resetting, setResetting] = useState(false);
  const [resetError, setResetError] = useState<string | null>(null);

  const handleUnlock = async (event: React.FormEvent) => {
    event.preventDefault();
    setLoading(true);
    setError(null);
    try {
      await unlock(password);
      navigate("/dashboard");
    } catch (err) {
      const apiError = err as ApiError;
      setError(apiError.message ?? "Failed to unlock wallet");
    } finally {
      setLoading(false);
    }
  };

  const handleResetWallet = async () => {
    if (!resetConfirm) return;
    setResetting(true);
    setResetError(null);
    try {
      await walletApi.resetLocalWallet();
      await walletApi.clearOnboardingDraft();
      setResetOpen(false);
      await refresh();
      navigate("/onboarding", { replace: true });
    } catch (err) {
      const apiError = err as ApiError;
      setResetError(apiError.message ?? "Failed to reset wallet");
    } finally {
      setResetting(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-4 sm:p-6">
      <Card className="w-full max-w-md">
        <CardHeader>
          <div className="mb-2 flex items-center gap-2">
            <BrandMark className="h-6 w-6" />
            <span className="font-semibold">Taurvia</span>
          </div>
          <CardTitle>Unlock wallet</CardTitle>
          <CardDescription>Enter your password to access your Solana wallet.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <form className="space-y-4" onSubmit={handleUnlock}>
            <div className="space-y-2">
              <Label htmlFor="password">Password</Label>
              <Input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                autoFocus
              />
            </div>
            {error && <Alert className="border-destructive/40 text-destructive">{error}</Alert>}
            <Button className="w-full" type="submit" disabled={loading || !password}>
              {loading ? "Checking password..." : "Unlock"}
            </Button>
          </form>
          <Button
            type="button"
            variant="destructive"
            className="w-full"
            onClick={() => {
              setResetError(null);
              setResetConfirm(false);
              setResetOpen(true);
            }}
          >
            Forgot password — reset wallet
          </Button>
        </CardContent>
      </Card>

      <Dialog
        open={resetOpen}
        onOpenChange={(open) => {
          setResetOpen(open);
          if (!open) {
            setResetConfirm(false);
            setResetError(null);
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Reset wallet on this device?</DialogTitle>
            <DialogDescription>
              This deletes the local wallet file. Your password cannot be recovered. You can set up
              again with Import from recovery phrase or Import from backup. Without those, funds are
              lost.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-3">
            <label className="flex cursor-pointer items-start gap-2.5 text-sm">
              <Checkbox
                checked={resetConfirm}
                onCheckedChange={setResetConfirm}
                aria-label="Confirm wallet reset"
              />
              <span>
                I have my recovery phrase or a portable wallet backup, or I accept losing access to
                funds on this wallet.
              </span>
            </label>
            {resetError && (
              <Alert className="border-destructive/40 text-destructive">{resetError}</Alert>
            )}
            <Button
              variant="destructive"
              className="w-full"
              disabled={resetting || !resetConfirm}
              onClick={() => void handleResetWallet()}
            >
              {resetting ? "Resetting…" : "Reset wallet"}
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
