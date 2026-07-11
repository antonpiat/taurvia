import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { useNavigate } from "react-router-dom";
import { normalizeExplorer } from "@/lib/explorer";
import { normalizeAppView, restoreSavedWindowSize } from "@/lib/appView";
import { DEFAULT_AUTO_LOCK_MINUTES, normalizeAutoLockMinutes } from "@/lib/autoLock";
import { DEFAULT_NETWORK_ID, normalizeNetworkId } from "@/lib/network";
import type { AppSettings, ExplorerKind, RuntimeConfig } from "@/lib/tauri";
import { TokenBalance, walletApi } from "@/lib/tauri";

const DEFAULT_SETTINGS: AppSettings = {
  rpc_url: null,
  jupiter_api_key: null,
  auto_lock_minutes: DEFAULT_AUTO_LOCK_MINUTES,
  hide_balances: true,
  explorer: "solscan",
  default_slippage_bps: 50,
  app_view: "desktop",
  window_width: null,
  window_height: null,
};

interface WalletContextValue {
  loading: boolean;
  balancesLoading: boolean;
  walletExists: boolean;
  unlocked: boolean;
  publicKey: string | null;
  /** Wallet file network id (`solana-mainnet`, …). */
  network: string;
  solBalance: number | null;
  solPriceUsd: number | null;
  solValueUsd: number | null;
  totalPortfolioUsd: number | null;
  tokens: TokenBalance[];
  settings: AppSettings;
  hideBalances: boolean;
  explorer: ExplorerKind;
  refresh: () => Promise<void>;
  refreshBalances: () => Promise<void>;
  unlock: (password: string) => Promise<void>;
  lock: () => Promise<void>;
  reloadSettings: () => Promise<AppSettings>;
  saveSettings: (next: AppSettings) => Promise<RuntimeConfig>;
  setHideBalances: (hidden: boolean) => Promise<void>;
}

const WalletContext = createContext<WalletContextValue | null>(null);

export function WalletProvider({ children }: { children: React.ReactNode }) {
  const navigate = useNavigate();
  const [loading, setLoading] = useState(true);
  const [balancesLoading, setBalancesLoading] = useState(false);
  const [walletExists, setWalletExists] = useState(false);
  const [unlocked, setUnlocked] = useState(false);
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [network, setNetwork] = useState(DEFAULT_NETWORK_ID);
  const [solBalance, setSolBalance] = useState<number | null>(null);
  const [solPriceUsd, setSolPriceUsd] = useState<number | null>(null);
  const [solValueUsd, setSolValueUsd] = useState<number | null>(null);
  const [totalPortfolioUsd, setTotalPortfolioUsd] = useState<number | null>(null);
  const [tokens, setTokens] = useState<TokenBalance[]>([]);
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const refreshPromise = useRef<Promise<void> | null>(null);
  const idleTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const unlockedRef = useRef(false);
  const autoLockMinutesRef = useRef(DEFAULT_AUTO_LOCK_MINUTES);
  const settingsRef = useRef(settings);
  const restoredWindowSize = useRef(false);

  settingsRef.current = settings;

  useEffect(() => {
    unlockedRef.current = unlocked;
  }, [unlocked]);

  useEffect(() => {
    autoLockMinutesRef.current = normalizeAutoLockMinutes(settings.auto_lock_minutes);
  }, [settings.auto_lock_minutes]);

  const applySnapshot = useCallback(
    (snapshot: Awaited<ReturnType<typeof walletApi.getWalletSnapshot>>) => {
      setWalletExists(snapshot.exists);
      setUnlocked(snapshot.unlocked);
      setPublicKey(snapshot.public_key);
      setNetwork(normalizeNetworkId(snapshot.network));
      setSolBalance(snapshot.sol_balance);
      setSolPriceUsd(snapshot.sol_price_usd);
      setSolValueUsd(snapshot.sol_value_usd);
      setTotalPortfolioUsd(snapshot.total_portfolio_usd);
      setTokens(snapshot.tokens ?? []);
    },
    [],
  );

  const refresh = useCallback(async () => {
    if (refreshPromise.current) {
      return refreshPromise.current;
    }

    const run = (async () => {
      setBalancesLoading(true);
      try {
        applySnapshot(await walletApi.getWalletSnapshot());
      } finally {
        setBalancesLoading(false);
        refreshPromise.current = null;
      }
    })();

    refreshPromise.current = run;
    return run;
  }, [applySnapshot]);

  const refreshBalances = useCallback(async () => {
    if (!walletExists || !unlocked) return;
    await refresh();
  }, [walletExists, unlocked, refresh]);

  const reloadSettings = useCallback(async () => {
    try {
      const next = await walletApi.getAppSettings();
      const merged: AppSettings = {
        ...DEFAULT_SETTINGS,
        ...next,
        explorer: normalizeExplorer(next.explorer ?? DEFAULT_SETTINGS.explorer),
        app_view: normalizeAppView(next.app_view ?? DEFAULT_SETTINGS.app_view),
        window_width: next.window_width ?? null,
        window_height: next.window_height ?? null,
        hide_balances:
          next.hide_balances === undefined
            ? DEFAULT_SETTINGS.hide_balances
            : Boolean(next.hide_balances),
        default_slippage_bps: next.default_slippage_bps ?? DEFAULT_SETTINGS.default_slippage_bps,
        auto_lock_minutes: normalizeAutoLockMinutes(next.auto_lock_minutes),
      };
      setSettings(merged);
      return merged;
    } catch {
      setSettings(DEFAULT_SETTINGS);
      return DEFAULT_SETTINGS;
    }
  }, []);

  const saveSettings = useCallback(async (next: AppSettings) => {
    const payload: AppSettings = {
      ...next,
      explorer: normalizeExplorer(next.explorer),
      app_view: normalizeAppView(next.app_view),
      auto_lock_minutes: normalizeAutoLockMinutes(next.auto_lock_minutes),
    };
    const runtime = await walletApi.updateAppSettings(payload);
    setSettings(payload);
    return runtime;
  }, []);

  const setHideBalances = useCallback(
    async (hidden: boolean) => {
      await saveSettings({ ...settingsRef.current, hide_balances: hidden });
    },
    [saveSettings],
  );

  const lock = useCallback(async () => {
    if (idleTimer.current) {
      clearTimeout(idleTimer.current);
      idleTimer.current = null;
    }
    await walletApi.lockWallet();
    setUnlocked(false);
    setPublicKey(null);
    setSolBalance(null);
    setSolPriceUsd(null);
    setSolValueUsd(null);
    setTotalPortfolioUsd(null);
    setTokens([]);
    setBalancesLoading(false);
  }, []);

  const scheduleAutoLock = useCallback(() => {
    if (idleTimer.current) {
      clearTimeout(idleTimer.current);
      idleTimer.current = null;
    }
    const minutes = autoLockMinutesRef.current;
    if (!unlockedRef.current || !minutes || minutes <= 0) {
      return;
    }
    idleTimer.current = setTimeout(() => {
      void (async () => {
        await lock();
        navigate("/unlock", { replace: true });
      })();
    }, minutes * 60_000);
  }, [lock, navigate]);

  useEffect(() => {
    void (async () => {
      setLoading(true);
      const [, loaded] = await Promise.all([refresh(), reloadSettings()]);
      if (!restoredWindowSize.current) {
        restoredWindowSize.current = true;
        await restoreSavedWindowSize(loaded);
      }
      setLoading(false);
    })();
  }, [refresh, reloadSettings]);

  useEffect(() => {
    if (!unlocked) {
      if (idleTimer.current) {
        clearTimeout(idleTimer.current);
        idleTimer.current = null;
      }
      return;
    }
    scheduleAutoLock();
    // Skip mousemove — it resets the timer on every pixel and tanks idle CPU.
    const onActivity = () => scheduleAutoLock();
    const windowEvents: Array<keyof WindowEventMap> = [
      "pointerdown",
      "keydown",
      "scroll",
      "touchstart",
    ];
    for (const event of windowEvents) {
      window.addEventListener(event, onActivity, { passive: true });
    }
    document.addEventListener("visibilitychange", onActivity);
    return () => {
      for (const event of windowEvents) {
        window.removeEventListener(event, onActivity);
      }
      document.removeEventListener("visibilitychange", onActivity);
      if (idleTimer.current) {
        clearTimeout(idleTimer.current);
        idleTimer.current = null;
      }
    };
  }, [unlocked, settings.auto_lock_minutes, scheduleAutoLock]);

  const unlock = useCallback(
    async (password: string) => {
      const key = await walletApi.unlockWallet(password);
      setWalletExists(true);
      setUnlocked(true);
      setPublicKey(key);
      setSolBalance(null);
      setSolPriceUsd(null);
      setSolValueUsd(null);
      setTotalPortfolioUsd(null);
      setTokens([]);
      void refresh();
    },
    [refresh],
  );

  const value = useMemo(
    () => ({
      loading,
      balancesLoading,
      walletExists,
      unlocked,
      publicKey,
      network,
      solBalance,
      solPriceUsd,
      solValueUsd,
      totalPortfolioUsd,
      tokens,
      settings,
      hideBalances: Boolean(settings.hide_balances),
      explorer: normalizeExplorer(settings.explorer),
      refresh,
      refreshBalances,
      unlock,
      lock,
      reloadSettings,
      saveSettings,
      setHideBalances,
    }),
    [
      loading,
      balancesLoading,
      walletExists,
      unlocked,
      publicKey,
      network,
      solBalance,
      solPriceUsd,
      solValueUsd,
      totalPortfolioUsd,
      tokens,
      settings,
      refresh,
      refreshBalances,
      unlock,
      lock,
      reloadSettings,
      saveSettings,
      setHideBalances,
    ],
  );

  return <WalletContext.Provider value={value}>{children}</WalletContext.Provider>;
}

export function useWallet() {
  const ctx = useContext(WalletContext);
  if (!ctx) throw new Error("useWallet must be used within WalletProvider");
  return ctx;
}
