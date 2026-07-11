import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { MaskedPhrase } from "@/components/MaskedPhrase";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert } from "@/components/ui/misc";
import { ApiError, walletApi } from "@/lib/tauri";

export function ShowSeedPage() {
  const navigate = useNavigate();
  const [mnemonic, setMnemonic] = useState("");
  const [loading, setLoading] = useState(true);
  const [regenerating, setRegenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const words = useMemo(() => mnemonic.split(/\s+/).filter(Boolean), [mnemonic]);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const draft = await walletApi.getOnboardingDraft();
        if (cancelled) return;
        if (!draft?.mnemonic || draft.mode !== "create") {
          navigate("/onboarding", { replace: true });
          return;
        }
        setMnemonic(draft.mnemonic);
      } catch {
        if (!cancelled) navigate("/onboarding", { replace: true });
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [navigate]);

  if (loading || !mnemonic) {
    return null;
  }

  const handleRegenerate = async () => {
    setRegenerating(true);
    setError(null);
    try {
      const phrase = await walletApi.generateMnemonic();
      await walletApi.setOnboardingDraft(phrase, "create");
      setMnemonic(phrase);
    } catch (err) {
      const apiError = err as ApiError;
      setError(apiError.message ?? "Failed to generate a new phrase");
    } finally {
      setRegenerating(false);
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
            Never share your seed phrase. Taurvia will never ask for it.
          </Alert>
          <MaskedPhrase words={words} />
          {error && <Alert className="border-destructive/40 text-destructive">{error}</Alert>}
          <div className="flex flex-wrap gap-3">
            <Button variant="outline" onClick={() => navigate("/onboarding")}>
              Back
            </Button>
            <Button
              variant="secondary"
              disabled={regenerating}
              onClick={() => void handleRegenerate()}
            >
              {regenerating ? "Generating..." : "Generate New Phrase"}
            </Button>
            <Button onClick={() => navigate("/onboarding/confirm")} disabled={regenerating}>
              I wrote it down
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
