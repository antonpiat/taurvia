import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { PasswordRequirements } from "@/components/PasswordRequirements";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Alert } from "@/components/ui/misc";
import { Checkbox } from "@/components/ui/checkbox";
import { useWallet } from "@/context/WalletContext";
import { isPasswordStrong, passwordStrengthError } from "@/lib/password";
import { ApiError, walletApi } from "@/lib/tauri";

export function SetPasswordPage() {
  const navigate = useNavigate();
  const { unlock, refresh } = useWallet();
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [mnemonic, setMnemonic] = useState("");
  const [mode, setMode] = useState<"create" | "import">("create");
  const [ready, setReady] = useState(false);
  const [enableDeviceProtection, setEnableDeviceProtection] = useState(false);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const draft = await walletApi.getOnboardingDraft();
        if (cancelled) return;
        if (!draft?.mnemonic) {
          navigate("/onboarding", { replace: true });
          return;
        }
        setMnemonic(draft.mnemonic);
        setMode(draft.mode === "import" ? "import" : "create");
        setReady(true);
      } catch {
        if (!cancelled) navigate("/onboarding", { replace: true });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [navigate]);

  const passwordsMatch = confirm.length > 0 && password === confirm;
  const canSubmit = isPasswordStrong(password) && passwordsMatch && !loading;

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    const strengthError = passwordStrengthError(password);
    if (strengthError) {
      setError(strengthError);
      return;
    }
    if (password !== confirm) {
      setError("Passwords do not match");
      return;
    }

    setLoading(true);
    setError(null);
    try {
      if (mode === "import") {
        await walletApi.importWallet(mnemonic, password);
      } else {
        await walletApi.createWallet(mnemonic, password);
      }
      await walletApi.clearOnboardingDraft();

      // Bind before unlock so a protection failure stays visible (unlock remounts into the app).
      if (enableDeviceProtection) {
        try {
          await walletApi.enableDeviceProtection(password);
        } catch (protErr) {
          const apiError = protErr as ApiError;
          await unlock(password);
          navigate("/settings/security", {
            replace: true,
            state: {
              notice:
                apiError.message ??
                "Wallet is ready, but Enhanced device protection could not be enabled. Turn it on below.",
            },
          });
          return;
        }
      }

      await unlock(password);
      navigate("/onboarding/ready");
    } catch (err) {
      const apiError = err as ApiError;
      setError(apiError.message ?? "Failed to create wallet");
      // File may already exist after a mid-flow failure — sync so the user can unlock.
      try {
        const snap = await walletApi.getWalletSnapshot();
        if (snap.exists) await refresh();
      } catch {
        // ignore snapshot errors
      }
    } finally {
      setLoading(false);
    }
  };

  if (!ready) {
    return null;
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>Set password</CardTitle>
          <CardDescription>This password encrypts your wallet file on this device.</CardDescription>
        </CardHeader>
        <CardContent>
          <form className="space-y-4" onSubmit={handleSubmit}>
            <div className="space-y-2">
              <Label htmlFor="password">Password</Label>
              <Input
                id="password"
                type="password"
                autoComplete="new-password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
              />
              <PasswordRequirements password={password} className="pt-1" />
            </div>
            <div className="space-y-2">
              <Label htmlFor="confirm">Confirm password</Label>
              <Input
                id="confirm"
                type="password"
                autoComplete="new-password"
                value={confirm}
                onChange={(e) => setConfirm(e.target.value)}
              />
              {confirm.length > 0 && (
                <p
                  className={
                    passwordsMatch
                      ? "text-xs text-emerald-600 dark:text-emerald-400"
                      : "text-xs text-destructive"
                  }
                >
                  {passwordsMatch ? "Passwords match" : "Passwords do not match"}
                </p>
              )}
            </div>
            <label className="flex cursor-pointer items-start gap-2.5 text-sm">
              <Checkbox
                checked={enableDeviceProtection}
                onCheckedChange={setEnableDeviceProtection}
                aria-label="Enable Enhanced device protection"
              />
              <span>
                Enable Enhanced device protection
                <span className="mt-1 block text-xs text-muted-foreground">
                  Binds decryption to this device’s credential store. OS reinstall, keychain reset,
                  or replacing the device can make the local wallet file unrecoverable without your
                  recovery phrase. Confirm you have (or will back up) that phrase before enabling.
                </span>
              </span>
            </label>
            {error && <Alert className="border-destructive/40 text-destructive">{error}</Alert>}
            <Button className="w-full" type="submit" disabled={!canSubmit}>
              {loading ? "Securing wallet..." : mode === "import" ? "Import wallet" : "Create wallet"}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
