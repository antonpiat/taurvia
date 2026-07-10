import { useEffect, useState } from "react";
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
  const { lock, refresh, refreshBalances } = useWallet();
  const [seedOpen, setSeedOpen] = useState(false);
  const [removeOpen, setRemoveOpen] = useState(false);
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [password, setPassword] = useState("");
  const [removePassword, setRemovePassword] = useState("");
  const [mnemonic, setMnemonic] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [removeError, setRemoveError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [removing, setRemoving] = useState(false);
  const [rpcUrl, setRpcUrl] = useState("");
  const [jupiterKey, setJupiterKey] = useState("");
  const [managedDefault, setManagedDefault] = useState("");
  const [activeRpc, setActiveRpc] = useState("");
  const [configMessage, setConfigMessage] = useState<string | null>(null);
  const [configError, setConfigError] = useState<string | null>(null);
  const [savingConfig, setSavingConfig] = useState(false);

  useEffect(() => {
    void (async () => {
      try {
        const [settings, defaultUrl] = await Promise.all([
          walletApi.getAppSettings(),
          walletApi.getManagedDefaultRpcUrl(),
        ]);
        setManagedDefault(defaultUrl);
        setRpcUrl(settings.rpc_url ?? "");
        setJupiterKey(settings.jupiter_api_key ?? "");
        setActiveRpc(settings.rpc_url?.trim() || defaultUrl);
      } catch {
        // ignore initial load errors
      }
    })();
  }, []);

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
      await walletApi.clearOnboardingDraft();
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

  const handleSaveNetwork = async () => {
    setSavingConfig(true);
    setConfigError(null);
    setConfigMessage(null);
    try {
      const runtime = await walletApi.updateAppSettings({
        rpc_url: rpcUrl.trim() ? rpcUrl.trim() : null,
        jupiter_api_key: jupiterKey.trim() ? jupiterKey.trim() : null,
      });
      setActiveRpc(runtime.rpc_url);
      setConfigMessage("Network settings saved. Refreshing balances…");
      await refreshBalances();
    } catch (err) {
      const apiError = err as ApiError;
      setConfigError(apiError.message ?? "Failed to save network settings");
    } finally {
      setSavingConfig(false);
    }
  };

  const handleResetNetwork = async () => {
    setRpcUrl("");
    setJupiterKey("");
    setSavingConfig(true);
    setConfigError(null);
    setConfigMessage(null);
    try {
      const runtime = await walletApi.updateAppSettings({
        rpc_url: null,
        jupiter_api_key: null,
      });
      setActiveRpc(runtime.rpc_url);
      setConfigMessage("Reset to managed default RPC.");
      await refreshBalances();
    } catch (err) {
      const apiError = err as ApiError;
      setConfigError(apiError.message ?? "Failed to reset network settings");
    } finally {
      setSavingConfig(false);
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
          <CardDescription>
            Solana mainnet. Works out of the box with Aegis managed defaults — no API key required.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-2 text-sm">
          <div className="flex justify-between gap-3">
            <span className="text-muted-foreground">Network</span>
            <span>solana-mainnet</span>
          </div>
          <div className="flex justify-between gap-3">
            <span className="text-muted-foreground">Active RPC</span>
            <span className="max-w-[60%] truncate font-mono text-xs">{activeRpc || "—"}</span>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Advanced</CardTitle>
          <CardDescription>
            Optional power-user overrides. Most users can leave these empty.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <Button variant="outline" onClick={() => navigate("/accounts")}>
            Manage accounts
          </Button>
          <Button variant="secondary" onClick={() => setAdvancedOpen((open) => !open)}>
            {advancedOpen ? "Hide network overrides" : "Custom RPC / Jupiter key"}
          </Button>
          {advancedOpen && (
            <div className="space-y-3 rounded-md border border-border p-3">
              <p className="text-xs text-muted-foreground">
                Default RPC: {managedDefault || "managed public mainnet"}. Developer{" "}
                <code className="font-mono">.env</code> is local-only and never ships in releases.
              </p>
              <div className="space-y-2">
                <Label htmlFor="rpc-url">Custom RPC URL</Label>
                <Input
                  id="rpc-url"
                  value={rpcUrl}
                  onChange={(e) => setRpcUrl(e.target.value)}
                  placeholder={managedDefault || "https://api.mainnet-beta.solana.com"}
                  autoComplete="off"
                  spellCheck={false}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="jupiter-key">Jupiter API key (optional)</Label>
                <Input
                  id="jupiter-key"
                  type="password"
                  value={jupiterKey}
                  onChange={(e) => setJupiterKey(e.target.value)}
                  placeholder="Leave empty for keyless Jupiter"
                  autoComplete="off"
                />
              </div>
              {configError && (
                <Alert className="border-destructive/40 text-destructive">{configError}</Alert>
              )}
              {configMessage && <Alert>{configMessage}</Alert>}
              <div className="flex flex-col gap-2 sm:flex-row">
                <Button onClick={() => void handleSaveNetwork()} disabled={savingConfig}>
                  {savingConfig ? "Saving..." : "Save"}
                </Button>
                <Button
                  variant="outline"
                  onClick={() => void handleResetNetwork()}
                  disabled={savingConfig}
                >
                  Reset to default
                </Button>
              </div>
            </div>
          )}
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
            <DialogDescription>Enter your password to view the seed phrase.</DialogDescription>
          </DialogHeader>
          <div className="space-y-3">
            <div className="space-y-2">
              <Label htmlFor="reveal-password">Password</Label>
              <Input
                id="reveal-password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
              />
            </div>
            {error && <Alert className="border-destructive/40 text-destructive">{error}</Alert>}
            {mnemonic && <MaskedPhrase words={words} />}
            <Button onClick={() => void handleRevealSeed()} disabled={loading || !password}>
              {loading ? "Verifying..." : "Reveal"}
            </Button>
          </div>
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
            <DialogTitle>Remove wallet</DialogTitle>
            <DialogDescription>
              Confirm with your password. Make sure you have your recovery phrase.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-3">
            <div className="space-y-2">
              <Label htmlFor="remove-password">Password</Label>
              <Input
                id="remove-password"
                type="password"
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
