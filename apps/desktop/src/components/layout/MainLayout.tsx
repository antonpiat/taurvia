import { useEffect, useRef, useState } from "react";
import { NavLink, Outlet, useLocation, useNavigate } from "react-router-dom";
import {
  Activity,
  ArrowDownLeft,
  ArrowLeftRight,
  ArrowUpRight,
  Check,
  ChevronDown,
  Copy,
  LayoutDashboard,
  Lock,
  Settings,
  Wallet,
} from "lucide-react";
import { BrandMark } from "@/components/BrandMark";
import { Button } from "@/components/ui/button";
import { useWallet } from "@/context/WalletContext";
import { normalizeAppView, useLayoutMode, useSyncAppViewOnResize } from "@/lib/appView";
import { networkShortLabel, productChainLabel } from "@/lib/network";
import {
  DEFAULT_SETTINGS_SECTION,
  SETTINGS_SECTIONS,
} from "@/lib/settingsNav";
import { cn, formatUsd, shortenAddress } from "@/lib/utils";

const navItems = [
  { to: "/dashboard", label: "Dashboard", icon: LayoutDashboard },
  { to: "/swap", label: "Swap", icon: ArrowLeftRight },
  { to: "/send", label: "Send", icon: ArrowUpRight },
  { to: "/receive", label: "Receive", icon: ArrowDownLeft },
  { to: "/activity", label: "Activity", icon: Activity },
];

export function MainLayout() {
  const navigate = useNavigate();
  const location = useLocation();
  const {
    publicKey,
    totalPortfolioUsd,
    balancesLoading,
    lock,
    hideBalances,
    network,
    settings,
    saveSettings,
  } = useWallet();
  const layout = useLayoutMode();
  const isPhone = layout === "phone";
  const isCompact = layout === "compact";
  const isDesktop = layout === "desktop";
  const networkLabel = networkShortLabel(network);
  const chainLabel = productChainLabel(network);
  const [copied, setCopied] = useState(false);
  const [settingsExpanded, setSettingsExpanded] = useState(false);
  const wasOnSettings = useRef(false);
  const settingsRef = useRef(settings);
  settingsRef.current = settings;

  useSyncAppViewOnResize(
    normalizeAppView(settings.app_view),
    settings.window_width,
    settings.window_height,
    async ({ view, width, height }) => {
      const current = settingsRef.current;
      await saveSettings({
        ...current,
        app_view: view,
        window_width: width,
        window_height: height,
      });
    },
  );

  const settingsActive = location.pathname.startsWith("/settings");
  const settingsPath = `/settings/${DEFAULT_SETTINGS_SECTION}`;

  useEffect(() => {
    if (settingsActive && !wasOnSettings.current) {
      setSettingsExpanded(true);
    }
    if (!settingsActive) {
      setSettingsExpanded(false);
    }
    wasOnSettings.current = settingsActive;
  }, [settingsActive]);

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

  const handleSettingsClick = () => {
    if (!settingsActive) {
      setSettingsExpanded(true);
      navigate(settingsPath);
      return;
    }
    setSettingsExpanded((open) => !open);
  };

  return (
    <div
      className={cn(
        "flex h-dvh overflow-hidden bg-background",
        isPhone ? "flex-col" : "flex-row",
      )}
    >
      {!isPhone && (
        <aside
          className={cn(
            "sticky top-0 flex h-dvh shrink-0 flex-col border-r border-border bg-card/40",
            isCompact ? "w-20 p-3" : "w-64 p-4",
          )}
        >
          <div
            className={cn(
              "mb-6 flex shrink-0 items-center gap-2",
              isDesktop ? "mb-8 px-2" : "px-1",
            )}
          >
            <BrandMark className="h-7 w-7" />
            {isDesktop && (
              <div className="min-w-0">
                <p className="text-lg font-semibold">Taurvia</p>
                <p className="text-xs text-muted-foreground">{chainLabel}</p>
              </div>
            )}
          </div>

          <nav className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto">
            {navItems.map(({ to, label, icon: Icon }) => (
              <NavLink
                key={to}
                to={to}
                title={label}
                className={({ isActive }) =>
                  cn(
                    "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm transition-colors",
                    isDesktop ? "justify-start" : "justify-center",
                    isActive
                      ? "bg-primary/15 text-primary"
                      : "text-muted-foreground hover:bg-accent hover:text-foreground",
                  )
                }
              >
                <Icon className="h-4 w-4 shrink-0" />
                {isDesktop && <span>{label}</span>}
              </NavLink>
            ))}

            <div className="space-y-1">
              <button
                type="button"
                title="Settings"
                onClick={handleSettingsClick}
                className={cn(
                  "flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-sm transition-colors",
                  isDesktop ? "justify-start" : "justify-center",
                  settingsActive
                    ? "bg-primary/15 text-primary"
                    : "text-muted-foreground hover:bg-accent hover:text-foreground",
                )}
              >
                <Settings className="h-4 w-4 shrink-0" />
                {isDesktop && (
                  <span className="min-w-0 flex-1 text-left">Settings</span>
                )}
                {isDesktop && (
                  <ChevronDown
                    className={cn(
                      "h-3.5 w-3.5 shrink-0 transition-transform",
                      settingsExpanded && "rotate-180",
                    )}
                  />
                )}
              </button>

              {isDesktop && settingsActive && settingsExpanded && (
                <ul className="space-y-0.5" aria-label="Settings sections">
                  {SETTINGS_SECTIONS.map((item, index) => {
                    const to = `/settings/${item.id}`;
                    const isLast = index === SETTINGS_SECTIONS.length - 1;
                    return (
                      <li key={item.id} className="relative">
                        <span
                          aria-hidden
                          className={cn(
                            "absolute left-5 top-0 w-px bg-border",
                            index === 0 ? "top-1/2" : "top-0",
                            isLast ? "h-1/2" : "h-full",
                          )}
                        />
                        <span
                          aria-hidden
                          className="absolute left-5 top-1/2 h-px w-2.5 -translate-y-1/2 bg-border"
                        />
                        <NavLink
                          to={to}
                          className={({ isActive }) =>
                            cn(
                              "relative ml-7 block rounded-md px-2.5 py-1.5 text-left text-sm transition-colors",
                              isActive
                                ? "bg-secondary text-secondary-foreground"
                                : "text-muted-foreground hover:bg-accent/60 hover:text-foreground",
                              item.id === "danger" && !isActive && "text-destructive/80",
                              item.id === "danger" &&
                                isActive &&
                                "bg-destructive/15 text-destructive",
                            )
                          }
                        >
                          {item.label}
                        </NavLink>
                      </li>
                    );
                  })}
                </ul>
              )}
            </div>
          </nav>

          {publicKey && (
            <div className="mt-4 shrink-0 border-t border-border pt-3">
              {isCompact && (
                <div className="flex flex-col items-center gap-2">
                  <button
                    type="button"
                    onClick={() => void handleCopy()}
                    className="flex h-10 w-10 items-center justify-center rounded-lg border border-border bg-background/70 text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
                    aria-label="Copy wallet address"
                    title={shortenAddress(publicKey, 6)}
                  >
                    {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
                  </button>
                  <button
                    type="button"
                    onClick={() => navigate("/accounts")}
                    className="flex h-10 w-10 items-center justify-center rounded-lg border border-border bg-background/70 text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
                    aria-label="Manage accounts"
                    title="Manage accounts"
                  >
                    <Wallet className="h-4 w-4" />
                  </button>
                  <button
                    type="button"
                    onClick={() => void handleLock()}
                    className="flex h-10 w-10 items-center justify-center rounded-lg border border-border bg-background/70 text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
                    aria-label="Lock wallet"
                    title="Lock wallet"
                  >
                    <Lock className="h-4 w-4" />
                  </button>
                </div>
              )}

              {isDesktop && (
                <div className="space-y-3">
                  <button
                    type="button"
                    onClick={() => void handleCopy()}
                    className="flex w-full items-center gap-3 rounded-lg px-1 py-1 text-left transition-colors hover:bg-accent/40"
                    aria-label="Copy wallet address"
                  >
                    <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-border bg-secondary">
                      <Wallet className="h-4 w-4 text-foreground" />
                    </span>
                    <span className="min-w-0 flex-1">
                      <span className="block font-mono text-sm text-foreground">
                        {shortenAddress(publicKey, 4)}
                      </span>
                      <span className="block truncate text-[11px] text-muted-foreground">
                        {hideBalances
                          ? networkLabel
                          : balancesLoading && totalPortfolioUsd === null
                            ? "Loading…"
                            : `${formatUsd(totalPortfolioUsd)} · ${networkLabel}`}
                      </span>
                    </span>
                    <span className="text-muted-foreground">
                      {copied ? (
                        <Check className="h-3.5 w-3.5" />
                      ) : (
                        <Copy className="h-3.5 w-3.5" />
                      )}
                    </span>
                  </button>

                  <div className="grid grid-cols-2 gap-2">
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      className="w-full"
                      onClick={() => navigate("/accounts")}
                    >
                      Accounts
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      className="w-full"
                      onClick={() => void handleLock()}
                    >
                      <Lock className="h-3.5 w-3.5" />
                      Lock
                    </Button>
                  </div>
                </div>
              )}
            </div>
          )}
        </aside>
      )}

      {isPhone && (
        <header className="flex shrink-0 items-center justify-between border-b border-border bg-card/40 px-4 py-3">
          <div className="flex items-center gap-2">
            <BrandMark className="h-6 w-6" />
            <p className="text-sm font-semibold leading-tight">Taurvia</p>
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
      )}

      <main
        className={cn(
          "min-h-0 min-w-0 flex-1 overflow-y-auto p-4 sm:p-6",
          isPhone ? "pb-24" : "p-8 pb-8",
        )}
      >
        <div className="mx-auto w-full max-w-3xl">
          <Outlet />
        </div>
      </main>

      {isPhone && (
        <nav className="fixed inset-x-0 bottom-0 z-40 border-t border-border bg-card/95 backdrop-blur">
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
            <NavLink
              to={settingsPath}
              className={() =>
                cn(
                  "flex flex-col items-center gap-0.5 rounded-md px-1 py-1.5 text-[10px] transition-colors",
                  settingsActive
                    ? "bg-primary/15 text-primary"
                    : "text-muted-foreground hover:text-foreground",
                )
              }
            >
              <Settings className="h-4 w-4" />
              <span className="truncate">Settings</span>
            </NavLink>
          </div>
        </nav>
      )}
    </div>
  );
}
