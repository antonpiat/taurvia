import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Alert } from "@/components/ui/misc";
import { useWallet } from "@/context/WalletContext";
import { ApiError } from "@/lib/tauri";
import { Shield } from "lucide-react";

export function UnlockPage() {
  const navigate = useNavigate();
  const { unlock } = useWallet();
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

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

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6">
      <Card className="w-full max-w-md">
        <CardHeader>
          <div className="mb-2 flex items-center gap-2 text-primary">
            <Shield className="h-5 w-5" />
            <span className="font-semibold">Aegis</span>
          </div>
          <CardTitle>Unlock wallet</CardTitle>
          <CardDescription>Enter your password to access your Solana wallet.</CardDescription>
        </CardHeader>
        <CardContent>
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
              {loading ? "Unlocking..." : "Unlock"}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
