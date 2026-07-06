import { QRCodeSVG } from "qrcode.react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useWallet } from "@/context/WalletContext";
import { Copy } from "lucide-react";
import { useState } from "react";

export function ReceivePage() {
  const { publicKey } = useWallet();
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    if (!publicKey) return;
    await navigator.clipboard.writeText(publicKey);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="mx-auto max-w-xl space-y-6">
      <div>
        <h1 className="text-3xl font-semibold">Receive</h1>
        <p className="text-muted-foreground">Share your address to receive SOL or SPL tokens.</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Your address</CardTitle>
          <CardDescription>Only send Solana network assets to this address.</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col items-center gap-4">
          {publicKey ? (
            <>
              <div className="rounded-xl bg-white p-4">
                <QRCodeSVG value={publicKey} size={220} />
              </div>
              <p className="break-all text-center font-mono text-sm">{publicKey}</p>
              <Button onClick={handleCopy}>
                <Copy className="h-4 w-4" />
                {copied ? "Copied" : "Copy address"}
              </Button>
            </>
          ) : (
            <p className="text-sm text-muted-foreground">Unlock your wallet to view your address.</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
