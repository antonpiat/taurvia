import { QRCodeSVG } from "qrcode.react";
import { NetworkPicker } from "@/components/NetworkPicker";
import { PageHeader } from "@/components/PageHeader";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useWallet } from "@/context/WalletContext";
import { receiveWarning } from "@/lib/network";
import { Copy } from "lucide-react";
import { useEffect, useState } from "react";

export function ReceivePage() {
  const { publicKey, networkInfo, nativeSymbol, networks, enabledNetworks, network, changeNetwork } =
    useWallet();
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    setCopied(false);
  }, [network, publicKey]);

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
        description={`Share your address to receive ${nativeSymbol}${
          networkInfo?.features.tokens ? " or tokens" : ""
        }.`}
      />

      <Card>
        <CardHeader>
          <CardTitle>Your address</CardTitle>
          <CardDescription>{receiveWarning(networkInfo)}</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col items-center gap-4">
          <NetworkPicker
            className="mx-auto w-full max-w-sm"
            networks={networks}
            activatedIds={enabledNetworks}
            selected={network}
            onSelect={(id) => void changeNetwork(id)}
          />
          {publicKey ? (
            <>
              <div className="rounded-xl bg-white p-3 sm:p-4">
                <QRCodeSVG key={publicKey} className="h-44 w-44 sm:h-[220px] sm:w-[220px]" value={publicKey} size={220} />
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
