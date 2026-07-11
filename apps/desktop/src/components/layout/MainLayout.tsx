import { useState } from "react";
import { NavLink, Outlet, useNavigate } from "react-router-dom";
import {
  Activity,
  ArrowDownLeft,
  ArrowLeftRight,
  ArrowUpRight,
  Check,
  Copy,
  LayoutDashboard,
  Lock,
  Settings,
} from "lucide-react";
import { BrandMark } from "@/components/BrandMark";
import { Button } from "@/components/ui/button";
import { useWallet } from "@/context/WalletContext";
import { cn, formatUsd, shortenAddress } from "@/lib/utils";

const navItems = [
  { to: "/dashboard", label: "Dashboard", icon: LayoutDashboard },
  { to: "/swap", label: "Swap", icon: ArrowLeftRight },
  { to: "/send", label: "Send", icon: ArrowUpRight },
  { to: "/receive", label: "Receive", icon: ArrowDownLeft },
  { to: "/activity", label: "Activity", icon: Activity },
  { to: "/settings", label: "Settings", icon: Settings },
];

export function MainLayout() {
  const navigate = useNavigate();
  const { publicKey, totalPortfolioUsd, balancesLoading, lock } = useWallet();
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    if (!publicKey) return;
    await navigator.clipboard.writeText(publicKey);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1500);
  };

  const handleLock = async () => {
    await lock();
    navigate("/unlock");
  };

  return (
    <div className="flex h-dvh flex-col overflow-hidden bg-background md:flex-row">
      {/* Desktop / tablet sidebar — sticky height so Active wallet stays pinned */}
      <aside className="sticky top-0 hidden h-dvh w-20 shrink-0 flex-col border-r border-border bg-card/40 p-3 md:flex lg:w-64 lg:p-4">
        <div className="mb-6 flex shrink-0 items-center gap-2 px-1 lg:mb-8 lg:px-2">
          <BrandMark className="h-7 w-7" />
          <div className="hidden min-w-0 lg:block">
            <p className="text-lg font-semibold">Taurvia</p>
            <p className="text-xs text-muted-foreground">Solana Wallet</p>
          </div>
        </div>

        <nav className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto">
          {navItems.map(({ to, label, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              title={label}
              className={({ isActive }) =>
                cn(
                  "flex items-center justify-center gap-3 rounded-lg px-3 py-2.5 text-sm transition-colors lg:justify-start",
                  isActive
                    ? "bg-primary/15 text-primary"
                    : "text-muted-foreground hover:bg-accent hover:text-foreground",
                )
              }
            >
              <Icon className="h-4 w-4 shrink-0" />
              <span className="hidden lg:inline">{label}</span>
            </NavLink>
          ))}
        </nav>

        {publicKey && (
          <div className="mt-4 shrink-0 space-y-3 rounded-lg border border-border bg-background/60 p-2 lg:p-3">
            <div className="flex items-start justify-between gap-2">
              <div className="hidden min-w-0 lg:block">
                <p className="text-xs font-medium text-foreground">Active wallet</p>
                <p className="truncate font-mono text-xs text-muted-foreground">
                  {shortenAddress(publicKey, 6)}
                </p>
              </div>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="mx-auto h-8 w-8 shrink-0 p-0 lg:mx-0"
                onClick={() => void handleCopy()}
                aria-label="Copy wallet address"
                title={shortenAddress(publicKey, 6)}
              >
                {copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
              </Button>
            </div>

            <div className="hidden items-center justify-between gap-2 text-xs text-muted-foreground lg:flex">
              <span className="truncate">
                {balancesLoading && totalPortfolioUsd === null
                  ? "Loading…"
                  : formatUsd(totalPortfolioUsd)}
              </span>
              <span className="shrink-0">Mainnet</span>
            </div>

            <div className="flex flex-col gap-2 lg:flex-row">
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="w-full lg:flex-1"
                onClick={() => void handleLock()}
                title="Lock wallet"
              >
                <Lock className="h-3.5 w-3.5" />
                <span className="hidden lg:inline">Lock</span>
              </Button>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="hidden text-muted-foreground lg:flex lg:flex-1"
                onClick={() => navigate("/accounts")}
              >
                Manage
              </Button>
            </div>
          </div>
        )}
      </aside>

      {/* Mobile top bar */}
      <header className="flex shrink-0 items-center justify-between border-b border-border bg-card/40 px-4 py-3 md:hidden">
        <div className="flex items-center gap-2">
          <BrandMark className="h-6 w-6" />
          <div>
            <p className="text-sm font-semibold leading-tight">Taurvia</p>
            <p className="text-[11px] text-muted-foreground">
              {balancesLoading && totalPortfolioUsd === null
                ? "Loading…"
                : formatUsd(totalPortfolioUsd)}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-1">
          {publicKey && (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="h-8 px-2 font-mono text-xs"
              onClick={() => void handleCopy()}
            >
              {copied ? <Check className="h-3.5 w-3.5" /> : null}
              {shortenAddress(publicKey, 4)}
            </Button>
          )}
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="h-8 w-8 p-0"
            onClick={() => void handleLock()}
            aria-label="Lock wallet"
          >
            <Lock className="h-3.5 w-3.5" />
          </Button>
        </div>
      </header>

      <main className="min-h-0 min-w-0 flex-1 overflow-y-auto p-4 pb-24 sm:p-6 md:p-8 md:pb-8">
        <Outlet />
      </main>

      {/* Mobile bottom nav */}
      <nav className="fixed inset-x-0 bottom-0 z-40 border-t border-border bg-card/95 backdrop-blur md:hidden">
        <div className="mx-auto grid max-w-lg grid-cols-6 gap-0.5 px-1 py-1.5">
          {navItems.map(({ to, label, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              className={({ isActive }) =>
                cn(
                  "flex flex-col items-center gap-0.5 rounded-md px-1 py-1.5 text-[10px] transition-colors",
                  isActive
                    ? "bg-primary/15 text-primary"
                    : "text-muted-foreground hover:text-foreground",
                )
              }
            >
              <Icon className="h-4 w-4" />
              <span className="truncate">{label}</span>
            </NavLink>
          ))}
        </div>
      </nav>
    </div>
  );
}
