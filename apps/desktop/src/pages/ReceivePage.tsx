import { QRCodeSVG } from "qrcode.react";
import { PageHeader } from "@/components/PageHeader";
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
    <div className="space-y-4 sm:space-y-6">
      <PageHeader
        title="Receive"
        description="Share your address to receive SOL or SPL tokens."
      />

      <Card>
        <CardHeader>
          <CardTitle>Your address</CardTitle>
          <CardDescription>Only send Solana network assets to this address.</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col items-center gap-4">
          {publicKey ? (
            <>
              <div className="rounded-xl bg-white p-3 sm:p-4">
                <QRCodeSVG className="h-44 w-44 sm:h-[220px] sm:w-[220px]" value={publicKey} size={220} />
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
