import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, Badge } from "@/components/ui/misc";
import { useWallet } from "@/context/WalletContext";
import { formatUsd, shortenAddress } from "@/lib/utils";

export function ManageAccountsPage() {
  const navigate = useNavigate();
  const { publicKey, solBalance, tokens } = useWallet();

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <div>
        <h1 className="text-3xl font-semibold">Manage accounts</h1>
        <p className="text-muted-foreground">
          Advanced token account tools for power users.
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Wallet</CardTitle>
          <CardDescription>Your Solana address and native balance.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2 text-sm">
          <div className="flex justify-between gap-3">
            <span className="text-muted-foreground">Address</span>
            <span className="font-mono">{publicKey ? shortenAddress(publicKey, 8) : "—"}</span>
          </div>
          <div className="flex justify-between gap-3">
            <span className="text-muted-foreground">SOL</span>
            <span className="font-mono">{solBalance !== null ? solBalance : "—"}</span>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Token accounts</CardTitle>
          <CardDescription>
            Associated token accounts currently held in this wallet.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {tokens.length === 0 ? (
            <p className="text-sm text-muted-foreground">No SPL token accounts with balance.</p>
          ) : (
            tokens.map((token) => (
              <div
                key={token.mint}
                className="flex items-center justify-between rounded-lg border border-border bg-background/50 px-4 py-3"
              >
                <div>
                  <p className="font-medium">{token.symbol}</p>
                  <p className="text-xs text-muted-foreground">{token.name}</p>
                  <Badge className="mt-1">{shortenAddress(token.mint)}</Badge>
                </div>
                <div className="text-right">
                  <p className="font-mono">{token.ui_amount}</p>
                  <p className="text-xs text-muted-foreground">{formatUsd(token.value_usd)}</p>
                </div>
              </div>
            ))
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Coming soon</CardTitle>
          <CardDescription>
            Close empty accounts and recover rent without cluttering Send or Swap.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <Alert>
            Recipient token accounts are already created automatically when you send SPL tokens.
            You do not need to create them manually.
          </Alert>
          <ul className="list-disc space-y-1 pl-5 text-sm text-muted-foreground">
            <li>Close empty associated token accounts</li>
            <li>Recover rent to your SOL balance</li>
            <li>Create an ATA only when you explicitly need one</li>
          </ul>
          <Button variant="outline" onClick={() => navigate("/settings")}>
            Back to settings
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
