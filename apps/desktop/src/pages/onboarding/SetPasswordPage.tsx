import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Alert } from "@/components/ui/misc";
import { useWallet } from "@/context/WalletContext";
import { ApiError, walletApi } from "@/lib/tauri";

export function SetPasswordPage() {
  const navigate = useNavigate();
  const { refresh } = useWallet();
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const mnemonic = sessionStorage.getItem("aegis_onboarding_mnemonic") ?? "";
  const mode = sessionStorage.getItem("aegis_onboarding_mode") ?? "create";

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    if (password.length < 8) {
      setError("Password must be at least 8 characters");
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
      await walletApi.unlockWallet(password);
      sessionStorage.removeItem("aegis_onboarding_mnemonic");
      sessionStorage.removeItem("aegis_onboarding_mode");
      await refresh();
      navigate("/onboarding/ready");
    } catch (err) {
      const apiError = err as ApiError;
      setError(apiError.message ?? "Failed to create wallet");
    } finally {
      setLoading(false);
    }
  };

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
                value={password}
                onChange={(e) => setPassword(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="confirm">Confirm password</Label>
              <Input
                id="confirm"
                type="password"
                value={confirm}
                onChange={(e) => setConfirm(e.target.value)}
              />
            </div>
            {error && <Alert className="border-destructive/40 text-destructive">{error}</Alert>}
            <Button className="w-full" type="submit" disabled={loading}>
              {loading ? "Securing wallet..." : mode === "import" ? "Import wallet" : "Create wallet"}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
