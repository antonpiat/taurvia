import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Textarea } from "@/components/ui/input";
import { Alert } from "@/components/ui/misc";
import { walletApi } from "@/lib/tauri";

export function CreateImportPage({ mode }: { mode: "create" | "import" }) {
  const navigate = useNavigate();
  const [mnemonic, setMnemonic] = useState("");
  const [loading, setLoading] = useState(mode === "create");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (mode !== "create") return;
    void (async () => {
      try {
        const phrase = await walletApi.generateMnemonic();
        setMnemonic(phrase);
      } catch (err) {
        setError(String(err));
      } finally {
        setLoading(false);
      }
    })();
  }, [mode]);

  const handleContinue = () => {
    const normalized = mnemonic.trim().toLowerCase();
    if (!normalized) {
      setError("Seed phrase is required");
      return;
    }
    sessionStorage.setItem("aegis_onboarding_mnemonic", normalized);
    if (mode === "import") {
      navigate("/onboarding/password");
      return;
    }
    navigate("/onboarding/seed");
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6">
      <Card className="w-full max-w-2xl">
        <CardHeader>
          <CardTitle>{mode === "create" ? "Create wallet" : "Import wallet"}</CardTitle>
          <CardDescription>
            {mode === "create"
              ? "Write down your recovery phrase. You will need it to restore your wallet."
              : "Enter your 12 or 24-word recovery phrase."}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {loading ? (
            <p className="text-sm text-muted-foreground">Generating secure seed phrase...</p>
          ) : (
            <Textarea
              value={mnemonic}
              onChange={(e) => setMnemonic(e.target.value)}
              readOnly={mode === "create"}
              placeholder="word1 word2 word3 ..."
            />
          )}
          {error && <Alert className="border-destructive/40 text-destructive">{error}</Alert>}
          <div className="flex gap-3">
            <Button variant="outline" onClick={() => navigate("/onboarding")}>
              Back
            </Button>
            <Button onClick={handleContinue} disabled={loading || !mnemonic.trim()}>
              Continue
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
