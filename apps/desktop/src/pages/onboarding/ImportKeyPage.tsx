import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Eye, EyeOff } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Alert } from "@/components/ui/misc";
import { walletApi } from "@/lib/tauri";

export function ImportKeyPage() {
  const navigate = useNavigate();
  const [secret, setSecret] = useState("");
  const [revealed, setRevealed] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleContinue = async () => {
    const value = secret.trim();
    if (!value) {
      setError("Private key is required");
      return;
    }
    setLoading(true);
    setError(null);
    try {
      await walletApi.setOnboardingDraft({
        mnemonic: "",
        mode: "import-key",
        secret: value,
      });
      setSecret("");
      navigate("/onboarding/password");
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>Import private key</CardTitle>
          <CardDescription>
            Solana (base58 or JSON), Ethereum (0x hex), Bitcoin WIF, or Sui (suiprivkey1…). Other
            chains cannot be derived from a single key.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="relative">
            <Input
              type={revealed ? "text" : "password"}
              value={secret}
              onChange={(e) => setSecret(e.target.value)}
              placeholder="Paste private key"
              autoComplete="off"
              spellCheck={false}
            />
            <button
              type="button"
              className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground"
              onClick={() => setRevealed((v) => !v)}
              aria-label={revealed ? "Hide key" : "Show key"}
            >
              {revealed ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            </button>
          </div>
          {error && <Alert className="border-destructive/40 text-destructive">{error}</Alert>}
          <div className="flex gap-3">
            <Button variant="outline" onClick={() => navigate("/onboarding/restore")}>
              Back
            </Button>
            <Button onClick={() => void handleContinue()} disabled={loading || !secret.trim()}>
              {loading ? "Checking..." : "Continue"}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

