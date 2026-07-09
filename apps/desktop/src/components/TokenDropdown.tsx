import { useEffect, useRef } from "react";
import { ChevronDown } from "lucide-react";
import { Label } from "@/components/ui/label";
import { cn, shortenAddress } from "@/lib/utils";

export type DropdownToken = {
  mint: string;
  symbol: string;
  name: string;
  logo_uri: string | null;
  balanceUi?: number;
};

function TokenAvatar({ symbol, logoUri }: { symbol: string; logoUri: string | null }) {
  if (logoUri) {
    return (
      <img
        src={logoUri}
        alt=""
        className="h-8 w-8 shrink-0 rounded-full border border-border bg-background object-cover"
        loading="lazy"
      />
    );
  }
  return (
    <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full border border-border bg-secondary text-xs font-semibold">
      {symbol.slice(0, 2).toUpperCase()}
    </div>
  );
}

export function TokenDropdown({
  label,
  token,
  tokens,
  selectedMint,
  placeholder,
  open,
  onOpenChange,
  onSelect,
}: {
  label: string;
  token: DropdownToken | undefined;
  tokens: DropdownToken[];
  selectedMint: string;
  placeholder: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSelect: (mint: string) => void;
}) {
  const rootRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onPointerDown = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) {
        onOpenChange(false);
      }
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onOpenChange(false);
    };
    document.addEventListener("mousedown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("mousedown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [open, onOpenChange]);

  return (
    <div className="space-y-2" ref={rootRef}>
      <Label>{label}</Label>
      <div className="relative">
        <button
          type="button"
          aria-haspopup="listbox"
          aria-expanded={open}
          onClick={() => onOpenChange(!open)}
          className={cn(
            "flex h-14 w-full items-center justify-between rounded-md border border-input bg-background px-3 text-left transition-colors",
            "hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
            open && "ring-2 ring-ring",
          )}
        >
          {token ? (
            <span className="flex min-w-0 items-center gap-3">
              <TokenAvatar symbol={token.symbol} logoUri={token.logo_uri} />
              <span className="min-w-0">
                <span className="block text-base font-semibold leading-tight">{token.symbol}</span>
                <span className="block truncate text-xs text-muted-foreground">{token.name}</span>
              </span>
            </span>
          ) : (
            <span className="text-muted-foreground">{placeholder}</span>
          )}
          <ChevronDown
            className={cn(
              "h-4 w-4 shrink-0 text-muted-foreground transition-transform",
              open && "rotate-180",
            )}
          />
        </button>

        {open && (
          <div
            role="listbox"
            className="absolute left-0 right-0 top-[calc(100%+0.35rem)] z-30 max-h-64 overflow-y-auto rounded-md border border-border bg-card p-1 shadow-lg"
          >
            {tokens.length === 0 ? (
              <p className="px-2.5 py-3 text-sm text-muted-foreground">No tokens available.</p>
            ) : (
              tokens.map((option) => {
                const selected = option.mint === selectedMint;
                return (
                  <button
                    key={option.mint}
                    type="button"
                    role="option"
                    aria-selected={selected}
                    onClick={() => onSelect(option.mint)}
                    className={cn(
                      "flex w-full items-center justify-between rounded-md px-2.5 py-2 text-left transition-colors",
                      selected ? "bg-primary/15" : "hover:bg-accent/50",
                    )}
                  >
                    <span className="flex min-w-0 items-center gap-3">
                      <TokenAvatar symbol={option.symbol} logoUri={option.logo_uri} />
                      <span className="min-w-0">
                        <span className="block font-semibold">{option.symbol}</span>
                        <span className="block truncate text-xs text-muted-foreground">
                          {option.name}
                        </span>
                      </span>
                    </span>
                    <span className="text-right text-xs text-muted-foreground">
                      {option.balanceUi !== undefined ? (
                        <span className="block font-mono text-foreground">{option.balanceUi}</span>
                      ) : null}
                      <span className="font-mono">{shortenAddress(option.mint)}</span>
                    </span>
                  </button>
                );
              })
            )}
          </div>
        )}
      </div>
    </div>
  );
}
