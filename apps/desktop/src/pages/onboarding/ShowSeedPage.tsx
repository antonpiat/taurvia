import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { MaskedPhrase } from "@/components/MaskedPhrase";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert } from "@/components/ui/misc";
import { ApiError, walletApi } from "@/lib/tauri";

export function ShowSeedPage() {
  const navigate = useNavigate();
  const [mnemonic, setMnemonic] = useState(
    () => sessionStorage.getItem("aegis_onboarding_mnemonic") ?? "",
  );
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const words = useMemo(() => mnemonic.split(/\s+/).filter(Boolean), [mnemonic]);

  useEffect(() => {
    if (!mnemonic) {
      navigate("/onboarding", { replace: true });
    }
  }, [mnemonic, navigate]);

  if (!mnemonic) {
    return null;
  }

  const handleRegenerate = async () => {
    setLoading(true);
    setError(null);
    try {
      const phrase = await walletApi.generateMnemonic();
      sessionStorage.setItem("aegis_onboarding_mnemonic", phrase);
      setMnemonic(phrase);
    } catch (err) {
      const apiError = err as ApiError;
      setError(apiError.message ?? "Failed to generate a new phrase");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6">
      <Card className="w-full max-w-2xl">
        <CardHeader>
          <CardTitle>Your recovery phrase</CardTitle>
          <CardDescription>Store this offline. Anyone with this phrase can access your funds.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Alert className="border-amber-500/40 text-amber-200">
            Never share your seed phrase. Aegis will never ask for it.
          </Alert>
          <MaskedPhrase words={words} />
          {error && <Alert className="border-destructive/40 text-destructive">{error}</Alert>}
          <div className="flex flex-wrap gap-3">
            <Button variant="outline" onClick={() => navigate("/onboarding")}>
              Back
            </Button>
            <Button variant="secondary" disabled={loading} onClick={() => void handleRegenerate()}>
              {loading ? "Generating..." : "Generate New Phrase"}
            </Button>
            <Button onClick={() => navigate("/onboarding/confirm")} disabled={loading}>
              I wrote it down
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
