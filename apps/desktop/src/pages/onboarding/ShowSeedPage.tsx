import { useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert } from "@/components/ui/misc";

export function ShowSeedPage() {
  const navigate = useNavigate();
  const mnemonic = sessionStorage.getItem("aegis_onboarding_mnemonic") ?? "";
  const words = useMemo(() => mnemonic.split(/\s+/).filter(Boolean), [mnemonic]);

  if (!mnemonic) {
    navigate("/onboarding");
    return null;
  }

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
          <div className="grid grid-cols-3 gap-3 rounded-xl border border-border bg-background/60 p-4">
            {words.map((word, index) => (
              <div key={`${word}-${index}`} className="rounded-md bg-card px-3 py-2 text-sm">
                <span className="mr-2 text-muted-foreground">{index + 1}.</span>
                {word}
              </div>
            ))}
          </div>
          <div className="flex gap-3">
            <Button variant="outline" onClick={() => navigate(-1)}>
              Back
            </Button>
            <Button onClick={() => navigate("/onboarding/confirm")}>I wrote it down</Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
