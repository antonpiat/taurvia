import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Check, ChevronDown } from "lucide-react";
import type { NetworkInfo } from "@/bindings";
import { TokenIcon } from "@/components/TokenIcon";
import { lastUsedNetworkOptions } from "@/lib/network";
import { cn } from "@/lib/utils";
import { useWallet } from "@/context/WalletContext";

function NetworkRow({
  info,
  selected,
  onSelect,
}: {
  info: NetworkInfo;
  selected: boolean;
  onSelect: (id: string) => void;
}) {
  return (
    <button
      type="button"
      role="option"
      aria-selected={selected}
      onClick={() => onSelect(info.id)}
      className={cn(
        "flex w-full items-center gap-2.5 rounded-md px-2 py-2 text-left transition-colors",
        selected ? "bg-primary/15" : "hover:bg-accent/50",
      )}
    >
      <TokenIcon
        symbol={info.native_symbol}
        mint={info.native_symbol.toLowerCase()}
        networkId={info.id}
        size={22}
      />
      <span className="min-w-0 flex-1">
        <span className="block truncate text-sm font-medium">{info.name}</span>
        <span className="block truncate text-[11px] text-muted-foreground">
          {info.native_symbol}
          {info.is_testnet ? " · testnet" : ""}
        </span>
      </span>
      {selected ? <Check className="h-3.5 w-3.5 shrink-0 text-primary" /> : null}
    </button>
  );
}

type MenuBox = {
  left: number;
  width: number;
  maxHeight: number;
  top?: number;
  bottom?: number;
};

export function NetworkPicker({
  networks,
  activatedIds,
  selected,
  onSelect,
  extraFilter,
  variant = "field",
  disabled,
  className,
}: {
  networks: NetworkInfo[];
  activatedIds: string[];
  selected: string;
  onSelect: (id: string) => void;
  extraFilter?: (info: NetworkInfo) => boolean;
  variant?: "field" | "toolbar";
  disabled?: boolean;
  className?: string;
}) {
  const [open, setOpen] = useState(false);
  const [menuBox, setMenuBox] = useState<MenuBox | null>(null);
  const rootRef = useRef<HTMLDivElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const { settings } = useWallet();
  const developerMode = Boolean(settings.developer_mode);
  const options = lastUsedNetworkOptions(
    networks,
    activatedIds,
    selected,
    developerMode,
  ).filter((n) => (extraFilter ? extraFilter(n) : true));
  const current = options.find((n) => n.id === selected) ?? options[0];
  const mainnets = options.filter((n) => !n.is_testnet);
  const testnets = options.filter((n) => n.is_testnet);

  const placeMenu = () => {
    const el = rootRef.current;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    const gap = 6;
    const pad = 8;
    const below = window.innerHeight - rect.bottom - gap;
    const above = rect.top - gap;
    const dropUp = variant === "toolbar" || (below < 240 && above > below);
    const maxHeight = Math.min(320, Math.max(120, dropUp ? above : below));
    const width = Math.min(Math.max(rect.width, 220), window.innerWidth - pad * 2);
    let left = rect.left;
    if (left + width > window.innerWidth - pad) {
      left = Math.max(pad, rect.right - width);
    }
    if (left < pad) left = pad;
    setMenuBox(
      dropUp
        ? { left, width, bottom: window.innerHeight - rect.top + gap, maxHeight }
        : { left, width, top: rect.bottom + gap, maxHeight },
    );
  };

  useLayoutEffect(() => {
    if (!open) {
      setMenuBox(null);
      return;
    }
    placeMenu();
    const onReposition = () => placeMenu();
    window.addEventListener("resize", onReposition);
    window.addEventListener("scroll", onReposition, true);
    return () => {
      window.removeEventListener("resize", onReposition);
      window.removeEventListener("scroll", onReposition, true);
    };
  }, [open, variant]);

  useEffect(() => {
    if (!open) return;
    const onPointerDown = (event: MouseEvent) => {
      const target = event.target as Node;
      if (rootRef.current?.contains(target)) return;
      if (menuRef.current?.contains(target)) return;
      setOpen(false);
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    document.addEventListener("mousedown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("mousedown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [open]);

  if (options.length === 0 || !current) return null;

  const pick = (id: string) => {
    setOpen(false);
    if (id !== selected) onSelect(id);
  };

  const compact = variant === "toolbar";
  const dropUp = menuBox?.bottom !== undefined;

  const menu =
    open && menuBox
      ? createPortal(
          <div
            ref={menuRef}
            role="listbox"
            style={{
              position: "fixed",
              left: menuBox.left,
              width: menuBox.width,
              maxHeight: menuBox.maxHeight,
              top: menuBox.top,
              bottom: menuBox.bottom,
            }}
            className="z-[80] overflow-y-auto rounded-md border border-border bg-card p-1 shadow-lg"
          >
            {mainnets.length > 0 && (
              <p className="px-2 pb-1 pt-1.5 text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
                Networks
              </p>
            )}
            {mainnets.map((n) => (
              <NetworkRow key={n.id} info={n} selected={n.id === selected} onSelect={pick} />
            ))}
            {testnets.length > 0 && (
              <>
                <p className="px-2 pb-1 pt-2 text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
                  Testnets
                </p>
                {testnets.map((n) => (
                  <NetworkRow key={n.id} info={n} selected={n.id === selected} onSelect={pick} />
                ))}
              </>
            )}
          </div>,
          document.body,
        )
      : null;

  return (
    <div ref={rootRef} className={cn("relative", className)}>
      <button
        type="button"
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-label="Network"
        disabled={disabled || options.length === 1}
        onClick={() => !disabled && options.length > 1 && setOpen((v) => !v)}
        className={cn(
          "flex w-full items-center gap-2 border border-input bg-background text-left transition-colors",
          "hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
          "disabled:pointer-events-none disabled:opacity-80",
          compact ? "h-9 rounded-lg px-2" : "h-12 rounded-md px-3",
          open && "ring-2 ring-ring",
        )}
      >
        <TokenIcon
          symbol={current.native_symbol}
          mint={current.native_symbol.toLowerCase()}
          networkId={current.id}
          size={compact ? 18 : 24}
        />
        <span className="min-w-0 flex-1">
          <span className={cn("block truncate font-medium", compact ? "text-xs" : "text-sm")}>
            {current.name}
          </span>
        </span>
        {options.length > 1 ? (
          <ChevronDown
            className={cn(
              "h-4 w-4 shrink-0 text-muted-foreground transition-transform",
              dropUp ? "rotate-180" : open && "rotate-180",
            )}
          />
        ) : null}
      </button>
      {menu}
    </div>
  );
}
