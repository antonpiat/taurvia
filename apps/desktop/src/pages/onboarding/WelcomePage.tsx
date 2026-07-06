import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Shield, Wallet } from "lucide-react";

export function WelcomePage() {
  const navigate = useNavigate();

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6">
      <Card className="w-full max-w-xl">
        <CardHeader>
          <div className="mb-2 flex items-center gap-2 text-primary">
            <Shield className="h-6 w-6" />
            <span className="text-xl font-semibold">Aegis</span>
          </div>
          <CardTitle>Welcome to your Solana wallet</CardTitle>
          <CardDescription>
            Non-custodial, local, and secure. Your keys never leave this device.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <Button
            className="w-full"
            onClick={() => {
              sessionStorage.setItem("aegis_onboarding_mode", "create");
              navigate("/onboarding/create");
            }}
          >
            <Wallet className="h-4 w-4" />
            Create a new wallet
          </Button>
          <Button
            className="w-full"
            variant="outline"
            onClick={() => {
              sessionStorage.setItem("aegis_onboarding_mode", "import");
              navigate("/onboarding/import");
            }}
          >
            Import existing wallet
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
