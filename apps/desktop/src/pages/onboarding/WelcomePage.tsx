import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { BrandMark } from "@/components/BrandMark";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { FileKey2, Wallet } from "lucide-react";
import { walletApi } from "@/lib/tauri";

export function WelcomePage() {
  const navigate = useNavigate();

  useEffect(() => {
    void walletApi.clearOnboardingDraft().catch(() => {
      // ignore — draft may already be empty
    });
  }, []);

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6">
      <Card className="w-full max-w-xl">
        <CardHeader>
          <div className="mb-2 flex items-center gap-2">
            <BrandMark className="h-7 w-7" />
            <span className="text-xl font-semibold">Taurvia</span>
          </div>
          <CardTitle>Welcome to your Solana wallet</CardTitle>
          <CardDescription>
            Non-custodial, local, and secure. Your keys never leave this device.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <Button className="w-full" onClick={() => navigate("/onboarding/create")}>
            <Wallet className="h-4 w-4" />
            Create a new wallet
          </Button>
          <Button
            className="w-full"
            variant="outline"
            onClick={() => navigate("/onboarding/import-backup")}
          >
            <FileKey2 className="h-4 w-4" />
            Import from backup
          </Button>
          <Button
            className="w-full"
            variant="destructive"
            onClick={() => navigate("/onboarding/import")}
          >
            Import from recovery phrase
          </Button>
          <p className="text-xs text-muted-foreground">
            Prefer a wallet backup file when you have one. Recovery phrase import is for seed
            restore only — treat those words as highly sensitive.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
