import { useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { FileText, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Alert } from "@/components/ui/misc";
import { useWallet } from "@/context/WalletContext";
import { ApiError, walletApi } from "@/lib/tauri";

export function ImportBackupPage() {
  const navigate = useNavigate();
  const { refresh } = useWallet();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [fileName, setFileName] = useState<string | null>(null);
  const [walletJson, setWalletJson] = useState("");
  const [password, setPassword] = useState("");
  const [enableDeviceProtection, setEnableDeviceProtection] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const clearFile = () => {
    setFileName(null);
    setWalletJson("");
    if (fileInputRef.current) fileInputRef.current.value = "";
  };

  const onFile = async (file: File | null) => {
    setError(null);
    if (!file) return;
    try {
      const text = await file.text();
      JSON.parse(text);
      setWalletJson(text);
      setFileName(file.name);
    } catch {
      setError("That file is not a valid wallet backup JSON.");
      clearFile();
    }
  };

  const handleImport = async () => {
    setError(null);
    if (!walletJson.trim()) {
      setError("Choose a wallet backup file.");
      return;
    }
    if (!password) {
      setError("Enter the password for this backup.");
      return;
    }
    setLoading(true);
    try {
      await walletApi.importWalletBackup(walletJson, password);
      if (enableDeviceProtection) {
        await walletApi.enableDeviceProtection(password);
      }
      await refresh();
      navigate("/dashboard", { replace: true });
    } catch (err) {
      const apiError = err as ApiError;
      setError(apiError.message ?? "Could not import backup.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6">
      <Card className="w-full max-w-xl">
        <CardHeader>
          <CardTitle>Import from backup</CardTitle>
          <CardDescription>
            Restore a password-protected wallet JSON exported from Taurvia. Device-bound backups
            cannot be opened on a new machine — use your recovery phrase instead, or disable
            Enhanced device protection before exporting on the original device.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <input
            ref={fileInputRef}
            type="file"
            accept="application/json,.json"
            className="hidden"
            onChange={(e) => void onFile(e.target.files?.[0] ?? null)}
          />
          {fileName ? (
            <div className="flex items-center justify-between gap-2 rounded-md border border-border px-3 py-2">
              <span className="flex min-w-0 items-center gap-2 text-sm">
                <FileText className="h-4 w-4 shrink-0" />
                <span className="truncate">{fileName}</span>
              </span>
              <Button type="button" variant="ghost" size="sm" onClick={clearFile}>
                <X className="h-4 w-4" />
              </Button>
            </div>
          ) : (
            <Button
              type="button"
              variant="outline"
              className="w-full"
              onClick={() => fileInputRef.current?.click()}
            >
              Choose backup file
            </Button>
          )}
          <div className="space-y-2">
            <Label htmlFor="backup-password">Backup password</Label>
            <Input
              id="backup-password"
              type="password"
              autoComplete="current-password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
          </div>
          <label className="flex cursor-pointer items-start gap-2 text-sm">
            <input
              type="checkbox"
              className="mt-1"
              checked={enableDeviceProtection}
              onChange={(e) => setEnableDeviceProtection(e.target.checked)}
            />
            <span>
              Enable Enhanced device protection on this machine
              <span className="mt-1 block text-xs text-muted-foreground">
                Optional. OS reinstall, keychain reset, or replacing the device can make the local
                wallet file unrecoverable without your recovery phrase.
              </span>
            </span>
          </label>
          {error && <Alert className="border-destructive/40 text-destructive">{error}</Alert>}
          <div className="flex gap-2">
            <Button variant="outline" onClick={() => navigate("/onboarding")}>
              Back
            </Button>
            <Button
              className="flex-1"
              disabled={loading || !walletJson || !password}
              onClick={() => void handleImport()}
            >
              {loading ? "Importing…" : "Import backup"}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
