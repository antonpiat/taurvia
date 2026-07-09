import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { MaskedPhrase } from "@/components/MaskedPhrase";
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
import { ApiError, walletApi } from "@/lib/tauri";

export function SettingsPage() {
  const navigate = useNavigate();
  const { lock, refresh } = useWallet();
  const [seedOpen, setSeedOpen] = useState(false);
  const [removeOpen, setRemoveOpen] = useState(false);
  const [password, setPassword] = useState("");
  const [removePassword, setRemovePassword] = useState("");
  const [mnemonic, setMnemonic] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [removeError, setRemoveError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [removing, setRemoving] = useState(false);

  const handleLock = async () => {
    await lock();
    navigate("/unlock");
  };

  const handleRevealSeed = async () => {
    setLoading(true);
    setError(null);
    try {
      const phrase = await walletApi.revealMnemonic(password);
      setMnemonic(phrase);
    } catch (err) {
      const apiError = err as ApiError;
      setError(apiError.message ?? "Failed to reveal seed phrase");
    } finally {
      setLoading(false);
    }
  };

  const handleRemoveWallet = async () => {
    setRemoving(true);
    setRemoveError(null);
    try {
      await walletApi.removeWallet(removePassword);
      sessionStorage.removeItem("aegis_onboarding_mnemonic");
      sessionStorage.removeItem("aegis_onboarding_mode");
      setRemoveOpen(false);
      setRemovePassword("");
      await refresh();
      navigate("/onboarding", { replace: true });
    } catch (err) {
      const apiError = err as ApiError;
      setRemoveError(apiError.message ?? "Failed to remove wallet");
    } finally {
      setRemoving(false);
    }
  };

  const words = mnemonic?.split(/\s+/).filter(Boolean) ?? [];

  return (
    <div className="mx-auto w-full max-w-2xl space-y-4 sm:space-y-6">
      <div>
        <h1 className="text-2xl font-semibold sm:text-3xl">Settings</h1>
        <p className="text-sm text-muted-foreground sm:text-base">
          Security and wallet preferences.
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Security</CardTitle>
          <CardDescription>Protect access to your wallet on this device.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <Button variant="outline" onClick={handleLock}>
            Lock wallet
          </Button>
          <Button variant="secondary" onClick={() => setSeedOpen(true)}>
            Reveal recovery phrase
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Network</CardTitle>
          <CardDescription>MVP uses Solana mainnet via configured RPC URL.</CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">solana-mainnet</p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Advanced</CardTitle>
          <CardDescription>
            Token account tools stay out of the main Send and Swap flow.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Button variant="outline" onClick={() => navigate("/accounts")}>
            Manage accounts
          </Button>
        </CardContent>
      </Card>

      <Card className="border-destructive/40">
        <CardHeader>
          <CardTitle>Danger zone</CardTitle>
          <CardDescription>
            Remove this wallet from the device. This cannot be undone without your recovery phrase.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Button variant="destructive" onClick={() => setRemoveOpen(true)}>
            Remove wallet from this device
          </Button>
        </CardContent>
      </Card>

      <Dialog
        open={seedOpen}
        onOpenChange={(open) => {
          setSeedOpen(open);
          if (!open) {
            setPassword("");
            setMnemonic(null);
            setError(null);
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Reveal recovery phrase</DialogTitle>
            <DialogDescription>Enter your password. Never share your seed phrase.</DialogDescription>
          </DialogHeader>
          {!mnemonic ? (
            <div className="space-y-3">
              <div className="space-y-2">
                <Label htmlFor="seed-password">Password</Label>
                <Input
                  id="seed-password"
                  type="password"
                  autoComplete="current-password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                />
              </div>
              {error && <Alert className="border-destructive/40 text-destructive">{error}</Alert>}
              <Button onClick={() => void handleRevealSeed()} disabled={loading || !password}>
                {loading ? "Verifying..." : "Reveal phrase"}
              </Button>
            </div>
          ) : (
            <MaskedPhrase words={words} />
          )}
        </DialogContent>
      </Dialog>

      <Dialog
        open={removeOpen}
        onOpenChange={(open) => {
          setRemoveOpen(open);
          if (!open) {
            setRemovePassword("");
            setRemoveError(null);
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Remove wallet from this device</DialogTitle>
            <DialogDescription>
              This permanently deletes the encrypted wallet file on this device. You will need your
              recovery phrase to restore access. Funds on-chain are not moved.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-3">
            <Alert className="border-destructive/40 text-destructive">
              Irreversible on this device. Make sure your recovery phrase is backed up.
            </Alert>
            <div className="space-y-2">
              <Label htmlFor="remove-password">Password</Label>
              <Input
                id="remove-password"
                type="password"
                autoComplete="current-password"
                value={removePassword}
                onChange={(e) => setRemovePassword(e.target.value)}
              />
            </div>
            {removeError && (
              <Alert className="border-destructive/40 text-destructive">{removeError}</Alert>
            )}
            <Button
              variant="destructive"
              onClick={() => void handleRemoveWallet()}
              disabled={removing || !removePassword}
            >
              {removing ? "Removing..." : "Remove wallet"}
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
