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
  Shield,
} from "lucide-react";
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
  const { publicKey, totalPortfolioUsd, lock } = useWallet();
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
    <div className="flex min-h-screen bg-background">
      <aside className="flex w-64 flex-col border-r border-border bg-card/40 p-4">
        <div className="mb-8 flex items-center gap-2 px-2">
          <Shield className="h-6 w-6 text-primary" />
          <div>
            <p className="text-lg font-semibold">Aegis</p>
            <p className="text-xs text-muted-foreground">Solana Wallet</p>
          </div>
        </div>
        <nav className="flex flex-1 flex-col gap-1">
          {navItems.map(({ to, label, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors",
                  isActive
                    ? "bg-primary/15 text-primary"
                    : "text-muted-foreground hover:bg-accent hover:text-foreground",
                )
              }
            >
              <Icon className="h-4 w-4" />
              {label}
            </NavLink>
          ))}
        </nav>

        {publicKey && (
          <div className="mt-4 space-y-3 rounded-lg border border-border bg-background/60 p-3">
            <div className="flex items-start justify-between gap-2">
              <div className="min-w-0">
                <p className="text-xs font-medium text-foreground">Active wallet</p>
                <p className="font-mono text-xs text-muted-foreground">
                  {shortenAddress(publicKey, 6)}
                </p>
              </div>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="h-8 w-8 shrink-0 p-0"
                onClick={() => void handleCopy()}
                aria-label="Copy wallet address"
              >
                {copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
              </Button>
            </div>

            <div className="flex items-center justify-between gap-2 text-xs text-muted-foreground">
              <span>{formatUsd(totalPortfolioUsd)}</span>
              <span>Mainnet</span>
            </div>

            <div className="flex gap-2">
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="flex-1"
                onClick={() => void handleLock()}
              >
                <Lock className="h-3.5 w-3.5" />
                Lock
              </Button>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="flex-1 text-muted-foreground"
                onClick={() => navigate("/accounts")}
              >
                Manage
              </Button>
            </div>
          </div>
        )}
      </aside>
      <main className="flex-1 overflow-y-auto p-8">
        <Outlet />
      </main>
    </div>
  );
}
