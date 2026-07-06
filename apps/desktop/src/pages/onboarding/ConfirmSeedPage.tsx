import { useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Alert } from "@/components/ui/misc";

function pickQuizIndices(length: number): number[] {
  const indices = Array.from({ length }, (_, i) => i);
  for (let i = indices.length - 1; i > 0; i -= 1) {
    const j = Math.floor(Math.random() * (i + 1));
    [indices[i], indices[j]] = [indices[j], indices[i]];
  }
  return indices.slice(0, 3).sort((a, b) => a - b);
}

export function ConfirmSeedPage() {
  const navigate = useNavigate();
  const mnemonic = sessionStorage.getItem("aegis_onboarding_mnemonic") ?? "";
  const words = useMemo(() => mnemonic.split(/\s+/).filter(Boolean), [mnemonic]);
  const [quizIndices] = useState(() => pickQuizIndices(words.length || 12));
  const [answers, setAnswers] = useState<Record<number, string>>({});
  const [error, setError] = useState<string | null>(null);

  if (!mnemonic) {
    navigate("/onboarding");
    return null;
  }

  const handleConfirm = () => {
    const valid = quizIndices.every((index) => answers[index]?.trim().toLowerCase() === words[index]);
    if (!valid) {
      setError("One or more words are incorrect. Please check your backup phrase.");
      return;
    }
    navigate("/onboarding/password");
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6">
      <Card className="w-full max-w-xl">
        <CardHeader>
          <CardTitle>Confirm recovery phrase</CardTitle>
          <CardDescription>Enter the requested words from your seed phrase.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {quizIndices.map((index) => (
            <div className="space-y-2" key={index}>
              <Label htmlFor={`word-${index}`}>Word #{index + 1}</Label>
              <Input
                id={`word-${index}`}
                value={answers[index] ?? ""}
                onChange={(e) => setAnswers((prev) => ({ ...prev, [index]: e.target.value }))}
              />
            </div>
          ))}
          {error && <Alert className="border-destructive/40 text-destructive">{error}</Alert>}
          <div className="flex gap-3">
            <Button variant="outline" onClick={() => navigate("/onboarding/seed")}>
              Back
            </Button>
            <Button onClick={handleConfirm}>Continue</Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
