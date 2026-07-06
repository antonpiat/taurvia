import { useState } from "react";
import { useNavigate } from "react-router-dom";
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
  const { lock } = useWallet();
  const [seedOpen, setSeedOpen] = useState(false);
  const [password, setPassword] = useState("");
  const [mnemonic, setMnemonic] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

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

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <div>
        <h1 className="text-3xl font-semibold">Settings</h1>
        <p className="text-muted-foreground">Security and wallet preferences.</p>
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
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                />
              </div>
              {error && <Alert className="border-destructive/40 text-destructive">{error}</Alert>}
              <Button onClick={handleRevealSeed} disabled={loading || !password}>
                {loading ? "Verifying..." : "Reveal phrase"}
              </Button>
            </div>
          ) : (
            <Alert>
              <p className="font-mono text-sm leading-7">{mnemonic}</p>
            </Alert>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
