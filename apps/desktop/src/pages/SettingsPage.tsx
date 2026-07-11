import { useEffect, useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { Navigate, useNavigate, useParams } from "react-router-dom";
import { SelectDropdown } from "@/components/SelectDropdown";
import { MaskedPhrase } from "@/components/MaskedPhrase";
import { PageHeader } from "@/components/PageHeader";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
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
import {
  APP_VIEW_WINDOW_SIZES,
  applyAppViewWindowSize,
  normalizeAppView,
  type AppViewKind,
  useLayoutMode,
} from "@/lib/appView";
import { DEFAULT_AUTO_LOCK_MINUTES, normalizeAutoLockMinutes } from "@/lib/autoLock";
import { explorerLabel, normalizeExplorer } from "@/lib/explorer";
import { networkDisplayId } from "@/lib/network";
import {
  DEFAULT_SETTINGS_SECTION,
  SETTINGS_SECTIONS,
  isSettingsSectionId,
  type SettingsSectionId,
} from "@/lib/settingsNav";
import { ApiError, AppSettings, ExplorerKind, walletApi } from "@/lib/tauri";

const APP_VIEW_OPTIONS: Array<{
  value: AppViewKind;
  label: string;
  description: string;
}> = [
  {
    value: "desktop",
    label: "Desktop",
    description: `Full sidebar · ${APP_VIEW_WINDOW_SIZES.desktop.label}`,
  },
  {
    value: "compact",
    label: "Compact",
    description: `Icon sidebar · ${APP_VIEW_WINDOW_SIZES.compact.label}`,
  },
  {
    value: "phone",
    label: "Phone",
    description: `Top bar & tabs · ${APP_VIEW_WINDOW_SIZES.phone.label}`,
  },
];

const AUTO_LOCK_OPTIONS: Array<{
  value: string;
  label: string;
  description: string;
  minutes: number;
}> = [
  { value: "0", label: "Off", description: "Stay unlocked until you lock", minutes: 0 },
  { value: "1", label: "1 minute", description: "Lock after 1 minute idle", minutes: 1 },
  {
    value: "5",
    label: "5 minutes",
    description: "Lock after 5 minutes idle (default)",
    minutes: DEFAULT_AUTO_LOCK_MINUTES,
  },
  { value: "15", label: "15 minutes", description: "Lock after 15 minutes idle", minutes: 15 },
  { value: "30", label: "30 minutes", description: "Lock after 30 minutes idle", minutes: 30 },
  { value: "60", label: "60 minutes", description: "Lock after 60 minutes idle", minutes: 60 },
];

const SECTION_COPY: Record<SettingsSectionId, string> = {
  view: "Layout and window size. Manual sizes are kept across restarts.",
  wallet: "Session preferences for this device.",
  security: "Protect access to your wallet on this device.",
  transactions: "Swap slippage and Solana explorer links.",
  network: "Active cluster and RPC endpoint.",
  advanced: "Optional RPC and Jupiter overrides.",
  danger: "Remove this wallet from the device.",
};

type OpenMenu = "section" | "app-view" | "auto-lock" | "explorer" | null;

export function SettingsPage() {
  const navigate = useNavigate();
  const { section: sectionParam } = useParams<{ section: string }>();
  const { refresh, refreshBalances, settings, saveSettings, network } = useWallet();
  const layout = useLayoutMode();
  const [seedOpen, setSeedOpen] = useState(false);
  const [removeOpen, setRemoveOpen] = useState(false);
  const [passwordOpen, setPasswordOpen] = useState(false);
  const [exportOpen, setExportOpen] = useState(false);
  const [openMenu, setOpenMenu] = useState<OpenMenu>(null);
  const [password, setPassword] = useState("");
  const [removePassword, setRemovePassword] = useState("");
  const [oldPassword, setOldPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [exportPassword, setExportPassword] = useState("");
  const [mnemonic, setMnemonic] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [removeError, setRemoveError] = useState<string | null>(null);
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const [passwordMessage, setPasswordMessage] = useState<string | null>(null);
  const [exportError, setExportError] = useState<string | null>(null);
  const [exportMessage, setExportMessage] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [removing, setRemoving] = useState(false);
  const [changingPassword, setChangingPassword] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [rpcUrl, setRpcUrl] = useState(settings.rpc_url ?? "");
  const [jupiterKey, setJupiterKey] = useState(settings.jupiter_api_key ?? "");
  const [managedDefault, setManagedDefault] = useState("");
  const [activeRpc, setActiveRpc] = useState(settings.rpc_url?.trim() || "");
  const [configMessage, setConfigMessage] = useState<string | null>(null);
  const [configError, setConfigError] = useState<string | null>(null);
  const [savingConfig, setSavingConfig] = useState(false);
  const [prefsMessage, setPrefsMessage] = useState<string | null>(null);
  const [prefsError, setPrefsError] = useState<string | null>(null);
  const [slippageInput, setSlippageInput] = useState(
    ((settings.default_slippage_bps ?? 50) / 100).toString(),
  );
  const [savingPrefs, setSavingPrefs] = useState(false);

  useEffect(() => {
    setOpenMenu(null);
  }, [sectionParam]);

  useEffect(() => {
    setRpcUrl(settings.rpc_url ?? "");
    setJupiterKey(settings.jupiter_api_key ?? "");
    setSlippageInput(((settings.default_slippage_bps ?? 50) / 100).toString());
  }, [settings.rpc_url, settings.jupiter_api_key, settings.default_slippage_bps]);

  useEffect(() => {
    void (async () => {
      try {
        const defaultUrl = await walletApi.getManagedDefaultRpcUrl();
        setManagedDefault(defaultUrl);
        setActiveRpc(settings.rpc_url?.trim() || defaultUrl);
      } catch {
        // ignore initial load errors
      }
    })();
  }, [settings.rpc_url]);

  const patchSettings = async (patch: Partial<AppSettings>) => {
    setPrefsError(null);
    setPrefsMessage(null);
    setSavingPrefs(true);
    const next: AppSettings = {
      ...settings,
      ...patch,
      explorer: normalizeExplorer(patch.explorer ?? settings.explorer),
      app_view: normalizeAppView(patch.app_view ?? settings.app_view),
    };
    try {
      await saveSettings(next);
      setPrefsMessage("Preferences saved.");
    } catch (err) {
      const apiError = err as ApiError;
      setPrefsError(apiError.message ?? "Failed to save preferences");
    } finally {
      setSavingPrefs(false);
    }
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

  const handleChangePassword = async () => {
    setPasswordError(null);
    setPasswordMessage(null);
    if (newPassword.length < 8) {
      setPasswordError("Password must be at least 8 characters");
      return;
    }
    if (newPassword !== confirmPassword) {
      setPasswordError("Passwords do not match");
      return;
    }
    setChangingPassword(true);
    try {
      await walletApi.changeWalletPassword(oldPassword, newPassword);
      setPasswordMessage("Password updated.");
      setOldPassword("");
      setNewPassword("");
      setConfirmPassword("");
    } catch (err) {
      const apiError = err as ApiError;
      setPasswordError(apiError.message ?? "Failed to change password");
    } finally {
      setChangingPassword(false);
    }
  };

  const handleExportWallet = async () => {
    setExportError(null);
    setExportMessage(null);
    setExporting(true);
    try {
      const path = await save({
        defaultPath: "taurvia-wallet-backup.json",
        filters: [{ name: "Wallet backup", extensions: ["json"] }],
      });
      if (!path) {
        setExportMessage("Export cancelled.");
        return;
      }
      await walletApi.exportWalletToPath(exportPassword, path);
      setExportMessage("Encrypted wallet backup saved.");
      setExportPassword("");
    } catch (err) {
      const apiError = err as ApiError;
      setExportError(apiError.message ?? "Failed to export wallet");
    } finally {
      setExporting(false);
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
      const next: AppSettings = {
        ...settings,
        rpc_url: rpcUrl.trim() ? rpcUrl.trim() : null,
        jupiter_api_key: jupiterKey.trim() ? jupiterKey.trim() : null,
      };
      const runtime = await saveSettings(next);
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
      const next: AppSettings = {
        ...settings,
        rpc_url: null,
        jupiter_api_key: null,
      };
      const runtime = await saveSettings(next);
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

  const handleSlippageBlur = async () => {
    const parsed = Number.parseFloat(slippageInput);
    if (!Number.isFinite(parsed) || parsed <= 0 || parsed > 50) {
      setSlippageInput(((settings.default_slippage_bps ?? 50) / 100).toString());
      return;
    }
    const bps = Math.round(parsed * 100);
    setSlippageInput((bps / 100).toString());
    await patchSettings({ default_slippage_bps: bps });
  };

  const words = mnemonic?.split(/\s+/).filter(Boolean) ?? [];

  if (!isSettingsSectionId(sectionParam)) {
    return <Navigate to={`/settings/${DEFAULT_SETTINGS_SECTION}`} replace />;
  }

  const section = sectionParam;
  const active = SETTINGS_SECTIONS.find((item) => item.id === section) ?? SETTINGS_SECTIONS[0];
  const autoLockValue = String(normalizeAutoLockMinutes(settings.auto_lock_minutes));
  const explorerValue = normalizeExplorer(settings.explorer);
  const appViewValue = layout;

  return (
    <div className="space-y-4 sm:space-y-6">
      <PageHeader
        parent="Settings"
        title={active.label}
        description={SECTION_COPY[section]}
      />

      {prefsMessage && <Alert>{prefsMessage}</Alert>}
      {prefsError && <Alert className="border-destructive/40 text-destructive">{prefsError}</Alert>}

      {layout !== "desktop" && (
        <SelectDropdown
          label="Section"
          value={section}
          options={SETTINGS_SECTIONS.map((item) => ({
            value: item.id,
            label: item.label,
            description: item.hint,
          }))}
          open={openMenu === "section"}
          onOpenChange={(open) => setOpenMenu(open ? "section" : null)}
          onChange={(next) => navigate(`/settings/${next}`)}
        />
      )}

      <div className="min-w-0 space-y-3">
        {section === "view" && (
          <Card>
            <CardContent className="space-y-4 pt-6">
              <SelectDropdown
                label="App layout"
                value={appViewValue}
                options={APP_VIEW_OPTIONS}
                open={openMenu === "app-view"}
                disabled={savingPrefs}
                onOpenChange={(open) => setOpenMenu(open ? "app-view" : null)}
                onChange={(next) => {
                  const view = normalizeAppView(next);
                  const size = APP_VIEW_WINDOW_SIZES[view];
                  void (async () => {
                    await patchSettings({
                      app_view: view,
                      window_width: size.width,
                      window_height: size.height,
                    });
                    await applyAppViewWindowSize(view);
                  })();
                }}
              />
              <p className="text-xs text-muted-foreground">
                Choosing a layout applies its default size. Manual resizing is remembered and
                restored the next time you open the app.
              </p>
            </CardContent>
          </Card>
        )}

        {section === "wallet" && (
          <Card>
            <CardContent className="space-y-4 pt-6">
              <SelectDropdown
                label="Auto-lock timeout"
                value={autoLockValue}
                options={AUTO_LOCK_OPTIONS.map((option) => ({
                  value: option.value,
                  label: option.label,
                  description: option.description,
                }))}
                open={openMenu === "auto-lock"}
                disabled={savingPrefs}
                onOpenChange={(open) => setOpenMenu(open ? "auto-lock" : null)}
                onChange={(next) => {
                  const option = AUTO_LOCK_OPTIONS.find((item) => item.value === next);
                  void patchSettings({
                    auto_lock_minutes: normalizeAutoLockMinutes(
                      option?.minutes ?? DEFAULT_AUTO_LOCK_MINUTES,
                    ),
                  });
                }}
              />
            </CardContent>
          </Card>
        )}

        {section === "security" && (
          <Card>
            <CardContent className="space-y-3 pt-6">
              <Button
                variant="outline"
                className="w-full justify-start"
                onClick={() => setPasswordOpen(true)}
              >
                Change wallet password
              </Button>
              <Button
                variant="outline"
                className="w-full justify-start"
                onClick={() => setSeedOpen(true)}
              >
                Reveal recovery phrase
              </Button>
              <Button
                variant="outline"
                className="w-full justify-start"
                onClick={() => setExportOpen(true)}
              >
                Export encrypted wallet
              </Button>
              <p className="text-xs text-muted-foreground">
                Sending and swapping always require your password.
              </p>
            </CardContent>
          </Card>
        )}

        {section === "transactions" && (
          <Card>
            <CardContent className="space-y-4 pt-6">
              <div className="space-y-2">
                <Label htmlFor="default-slippage">Default slippage (%)</Label>
                <Input
                  id="default-slippage"
                  value={slippageInput}
                  onChange={(e) => setSlippageInput(e.target.value)}
                  onBlur={() => void handleSlippageBlur()}
                  inputMode="decimal"
                />
              </div>
              <SelectDropdown
                label="Block explorer"
                value={explorerValue}
                options={[
                  {
                    value: "solscan" as ExplorerKind,
                    label: explorerLabel("solscan"),
                    description: "Open transaction links on Solscan",
                  },
                  {
                    value: "solanaExplorer" as ExplorerKind,
                    label: explorerLabel("solanaExplorer"),
                    description: "Open transaction links on Solana Explorer",
                  },
                ]}
                open={openMenu === "explorer"}
                disabled={savingPrefs}
                onOpenChange={(open) => setOpenMenu(open ? "explorer" : null)}
                onChange={(next) => void patchSettings({ explorer: next })}
              />
            </CardContent>
          </Card>
        )}

        {section === "network" && (
          <Card>
            <CardContent className="space-y-3 pt-6 text-sm">
              <div className="flex justify-between gap-3">
                <span className="text-muted-foreground">Network</span>
                <span>{networkDisplayId(network)}</span>
              </div>
              <div className="flex justify-between gap-3">
                <span className="text-muted-foreground">Active RPC</span>
                <span className="max-w-[60%] truncate font-mono text-xs">{activeRpc || "—"}</span>
              </div>
              <p className="text-xs text-muted-foreground">
                Works out of the box with Taurvia managed defaults — no API key required.
              </p>
            </CardContent>
          </Card>
        )}

        {section === "advanced" && (
          <Card>
            <CardContent className="space-y-4 pt-6">
              <p className="text-xs text-muted-foreground">
                Default RPC: {managedDefault || "managed public mainnet"}. Leave fields empty to
                use Taurvia defaults. Developer <code className="font-mono">.env</code> is
                local-only and never ships in releases.
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
                  {savingConfig ? "Saving..." : "Save overrides"}
                </Button>
                <Button
                  variant="outline"
                  onClick={() => void handleResetNetwork()}
                  disabled={savingConfig}
                >
                  Reset to default
                </Button>
              </div>
            </CardContent>
          </Card>
        )}

        {section === "danger" && (
          <Card className="border-destructive/40">
            <CardContent className="space-y-3 pt-6">
              <p className="text-sm text-muted-foreground">
                This cannot be undone without your recovery phrase.
              </p>
              <Button
                variant="destructive"
                className="w-full justify-start sm:w-auto"
                onClick={() => setRemoveOpen(true)}
              >
                Remove wallet from this device
              </Button>
            </CardContent>
          </Card>
        )}
      </div>

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
        open={passwordOpen}
        onOpenChange={(open) => {
          setPasswordOpen(open);
          if (!open) {
            setOldPassword("");
            setNewPassword("");
            setConfirmPassword("");
            setPasswordError(null);
            setPasswordMessage(null);
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Change wallet password</DialogTitle>
            <DialogDescription>
              Re-encrypts your wallet file with a new password. Minimum 8 characters.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-3">
            <div className="space-y-2">
              <Label htmlFor="old-password">Current password</Label>
              <Input
                id="old-password"
                type="password"
                value={oldPassword}
                onChange={(e) => setOldPassword(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="new-password">New password</Label>
              <Input
                id="new-password"
                type="password"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="confirm-password">Confirm new password</Label>
              <Input
                id="confirm-password"
                type="password"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
              />
            </div>
            {passwordError && (
              <Alert className="border-destructive/40 text-destructive">{passwordError}</Alert>
            )}
            {passwordMessage && <Alert>{passwordMessage}</Alert>}
            <Button
              onClick={() => void handleChangePassword()}
              disabled={changingPassword || !oldPassword || !newPassword || !confirmPassword}
            >
              {changingPassword ? "Updating..." : "Update password"}
            </Button>
          </div>
        </DialogContent>
      </Dialog>

      <Dialog
        open={exportOpen}
        onOpenChange={(open) => {
          setExportOpen(open);
          if (!open) {
            setExportPassword("");
            setExportError(null);
            setExportMessage(null);
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Export encrypted wallet</DialogTitle>
            <DialogDescription>
              Saves your encrypted wallet backup JSON. This does not include a plaintext seed.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-3">
            <div className="space-y-2">
              <Label htmlFor="export-password">Password</Label>
              <Input
                id="export-password"
                type="password"
                value={exportPassword}
                onChange={(e) => setExportPassword(e.target.value)}
              />
            </div>
            {exportError && (
              <Alert className="border-destructive/40 text-destructive">{exportError}</Alert>
            )}
            {exportMessage && <Alert>{exportMessage}</Alert>}
            <Button
              onClick={() => void handleExportWallet()}
              disabled={exporting || !exportPassword}
            >
              {exporting ? "Exporting..." : "Export backup"}
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
