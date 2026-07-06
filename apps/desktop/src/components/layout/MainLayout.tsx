import { NavLink, Outlet } from "react-router-dom";
import { Activity, ArrowDownLeft, ArrowUpRight, LayoutDashboard, Settings, Shield } from "lucide-react";
import { useWallet } from "@/context/WalletContext";
import { shortenAddress } from "@/lib/utils";
import { cn } from "@/lib/utils";

const navItems = [
  { to: "/dashboard", label: "Dashboard", icon: LayoutDashboard },
  { to: "/send", label: "Send", icon: ArrowUpRight },
  { to: "/receive", label: "Receive", icon: ArrowDownLeft },
  { to: "/activity", label: "Activity", icon: Activity },
  { to: "/settings", label: "Settings", icon: Settings },
];

export function MainLayout() {
  const { publicKey } = useWallet();

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
                  isActive ? "bg-primary/15 text-primary" : "text-muted-foreground hover:bg-accent hover:text-foreground",
                )
              }
            >
              <Icon className="h-4 w-4" />
              {label}
            </NavLink>
          ))}
        </nav>
        {publicKey && (
          <div className="mt-4 rounded-lg border border-border bg-background/60 p-3 text-xs text-muted-foreground">
            <p className="mb-1 font-medium text-foreground">Active wallet</p>
            <p className="font-mono">{shortenAddress(publicKey, 6)}</p>
          </div>
        )}
      </aside>
      <main className="flex-1 overflow-y-auto p-8">
        <Outlet />
      </main>
    </div>
  );
}
