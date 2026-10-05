import { useMemo, useState } from "react";
import { cn } from "@/lib/utils";
import { chainBadgeSrc, localLogoForAsset, type TokenChain } from "@/lib/tokenCatalog";

function initials(symbol: string): string {
  const t = symbol.replace(/[^a-zA-Z0-9]/g, "").slice(0, 2).toUpperCase();
  return t || "?";
}

function hueFromId(id: string): number {
  let h = 0;
  for (let i = 0; i < id.length; i += 1) h = (h * 31 + id.charCodeAt(i)) % 360;
  return h;
}

export function TokenIcon({
  symbol,
  mint,
  chain,
  networkId,
  logoUri,
  size = 36,
  className,
}: {
  symbol: string;
  mint?: string;
  chain?: TokenChain;
  networkId?: string;
  logoUri?: string | null;
  size?: number;
  className?: string;
}) {
  const [failed, setFailed] = useState(false);
  const src = useMemo(() => {
    const local = localLogoForAsset(chain, mint ?? "", symbol);
    if (local) return local;
    return logoUri || null;
  }, [chain, mint, symbol, logoUri]);
  const showImg = Boolean(src) && !failed;
  const hue = hueFromId(mint || symbol);
  const badge = chain ? chainBadgeSrc(chain, symbol, networkId) : null;

  return (
    <span
      className={cn("relative inline-flex shrink-0 items-center justify-center", className)}
      style={{ width: size, height: size }}
    >
      {showImg ? (
        <img
          src={src ?? undefined}
          alt=""
          className="h-full w-full rounded-full border border-border bg-background object-cover"
          onError={() => setFailed(true)}
        />
      ) : (
        <span
          className="flex h-full w-full items-center justify-center rounded-full border border-border text-[0.65em] font-semibold text-white"
          style={{ background: `hsl(${hue} 45% 38%)` }}
        >
          {initials(symbol)}
        </span>
      )}
      {badge && (
        <img
          src={badge}
          alt=""
          className="absolute -bottom-0.5 -right-0.5 rounded-full border border-background bg-background object-cover"
          style={{ width: Math.max(10, size * 0.38), height: Math.max(10, size * 0.38) }}
        />
      )}
    </span>
  );
}
