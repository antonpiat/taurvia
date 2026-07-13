import { useEffect, useRef, useState, type CSSProperties } from "react";
import { useNavigate } from "react-router-dom";
import { Eye, EyeOff, FileText, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Textarea } from "@/components/ui/input";
import { Alert } from "@/components/ui/misc";
import { normalizeRecoveryPhrase, parseRecoveryFile } from "@/lib/recoveryPhrase";
import { ApiError, walletApi } from "@/lib/tauri";

export function CreateImportPage({ mode }: { mode: "create" | "import" }) {
  if (mode === "create") {
    return <CreateRedirect />;
  }
  return <ImportWalletForm />;
}

function CreateRedirect() {
  const navigate = useNavigate();
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const phrase = await walletApi.generateMnemonic();
        if (cancelled) return;
        await walletApi.setOnboardingDraft(phrase, "create");
        if (cancelled) return;
        navigate("/onboarding/seed", { replace: true });
      } catch (err) {
        if (!cancelled) setError(String(err));
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [navigate]);

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6">
      <Card className="w-full max-w-xl">
        <CardHeader>
          <CardTitle>Create wallet</CardTitle>
          <CardDescription>Generating a secure recovery phrase...</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {error && <Alert className="border-destructive/40 text-destructive">{error}</Alert>}
          <Button variant="outline" onClick={() => navigate("/onboarding")}>
            Back
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}

type LoadedFile = {
  name: string;
  wordCount: number;
};

function ImportWalletForm() {
  const navigate = useNavigate();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [mnemonic, setMnemonic] = useState("");
  const [loadedFile, setLoadedFile] = useState<LoadedFile | null>(null);
  const [editing, setEditing] = useState(false);
  const [revealed, setRevealed] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const wordCount = mnemonic.trim() ? mnemonic.trim().split(/\s+/).filter(Boolean).length : 0;
  const showEditor = !loadedFile || editing;

  const clearPhrase = () => {
    setMnemonic("");
    setLoadedFile(null);
    setEditing(false);
    setRevealed(false);
    setError(null);
  };

  const handleFile = async (file: File | null) => {
    if (!file) return;
    setError(null);
    setLoading(true);
    try {
      const contents = await file.text();
      const phrase = parseRecoveryFile(file.name, contents);
      if (!phrase) {
        throw new Error("Recovery file is empty");
      }
      await walletApi.validateMnemonic(phrase);
      setMnemonic(phrase);
      setLoadedFile({
        name: file.name,
        wordCount: phrase.split(/\s+/).filter(Boolean).length,
      });
      setEditing(false);
      setRevealed(false);
    } catch (err) {
      const apiError = err as ApiError;
      setError(apiError.message ?? String(err));
    } finally {
      setLoading(false);
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    }
  };

  const handleContinue = async () => {
    const normalized = normalizeRecoveryPhrase(mnemonic);
    if (!normalized) {
      setError("Seed phrase is required");
      return;
    }
    setLoading(true);
    setError(null);
    try {
      await walletApi.validateMnemonic(normalized);
      await walletApi.setOnboardingDraft(normalized, "import");
      clearPhrase();
      navigate("/onboarding/password");
    } catch (err) {
      const apiError = err as ApiError;
      setError(apiError.message ?? "Invalid recovery phrase");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6">
      <Card className="w-full max-w-2xl">
        <CardHeader>
          <CardTitle>Import from recovery phrase</CardTitle>
          <CardDescription>
            Paste your 12 or 24-word recovery phrase, or import it from a .txt / .json file. Prefer
            Import from backup when you have an exported wallet file. Anyone with this phrase can
            move your funds. The phrase stays masked on screen.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {loadedFile && !editing ? (
            <Alert className="flex items-start justify-between gap-3">
              <div className="flex items-start gap-3">
                <FileText className="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
                <div className="space-y-1 text-sm">
                  <p className="font-medium text-foreground">Recovery phrase loaded</p>
                  <p className="text-muted-foreground">
                    {loadedFile.wordCount}-word phrase from{" "}
                    <span className="font-mono">{loadedFile.name}</span>
                  </p>
                  <p className="text-xs text-muted-foreground">
                    Words are hidden. Continue, or edit if you need to change them.
                  </p>
                </div>
              </div>
              <div className="flex shrink-0 flex-col gap-2 sm:flex-row">
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() => {
                    setEditing(true);
                    setRevealed(false);
                  }}
                >
                  Edit
                </Button>
                <Button type="button" size="sm" variant="ghost" onClick={clearPhrase}>
                  <X className="h-4 w-4" />
                  Clear
                </Button>
              </div>
            </Alert>
          ) : null}

          {showEditor ? (
            <div className="space-y-2">
              <div className="flex items-center justify-between gap-2">
                <p className="text-sm text-muted-foreground">
                  {wordCount > 0 ? `${wordCount} words entered` : "Enter or paste your phrase"}
                </p>
                <Button
                  type="button"
                  size="sm"
                  variant="ghost"
                  disabled={!mnemonic}
                  onClick={() => setRevealed((value) => !value)}
                >
                  {revealed ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                  {revealed ? "Hide" : "Show"}
                </Button>
              </div>
              <Textarea
                value={mnemonic}
                onChange={(e) => {
                  setMnemonic(e.target.value);
                  if (loadedFile) setLoadedFile(null);
                }}
                placeholder="Paste or type your recovery phrase"
                autoComplete="off"
                spellCheck={false}
                autoCorrect="off"
                autoCapitalize="off"
                style={
                  revealed
                    ? undefined
                    : ({ WebkitTextSecurity: "disc" } as CSSProperties)
                }
              />
            </div>
          ) : null}

          <div className="flex flex-wrap gap-3">
            <input
              ref={fileInputRef}
              type="file"
              accept=".txt,.json,text/plain,application/json"
              className="hidden"
              onChange={(e) => void handleFile(e.target.files?.[0] ?? null)}
            />
            <Button
              type="button"
              variant="secondary"
              disabled={loading}
              onClick={() => fileInputRef.current?.click()}
            >
              Import from file
            </Button>
          </div>
          {error && <Alert className="border-destructive/40 text-destructive">{error}</Alert>}
          <div className="flex gap-3">
            <Button variant="outline" onClick={() => navigate("/onboarding")}>
              Back
            </Button>
            <Button onClick={() => void handleContinue()} disabled={loading || !mnemonic.trim()}>
              {loading ? "Checking..." : "Continue"}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
