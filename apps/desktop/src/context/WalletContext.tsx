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
import { DEFAULT_NETWORK_ID, findNetwork, normalizeNetworkId } from "@/lib/network";
import type {
  AppSettings,
  ChainSnapshot,
  ExplorerKind,
  ImportKind,
  NetworkInfo,
  RuntimeConfig,
} from "@/lib/tauri";
import { TokenBalance, walletApi } from "@/lib/tauri";

const DEFAULT_SETTINGS: AppSettings = {
  rpc_url: null,
  rpc_urls: {},
  jupiter_api_key: null,
  network: DEFAULT_NETWORK_ID,
  enabled_networks: ["solana-mainnet", "ethereum-mainnet", "bitcoin-mainnet"],
  zerox_api_key: null,
  auto_lock_minutes: DEFAULT_AUTO_LOCK_MINUTES,
  hide_balances: true,
  explorer: "solscan",
  default_slippage_bps: 50,
  app_view: "desktop",
  window_width: null,
  window_height: null,
  swap_favorite_tokens: [],
};

interface WalletContextValue {
  loading: boolean;
  balancesLoading: boolean;
  walletExists: boolean;
  unlocked: boolean;
  publicKey: string | null;
  network: string;
  networks: NetworkInfo[];
  networkInfo: NetworkInfo | undefined;
  nativeBalance: number | null;
  nativeSymbol: string;
  nativePriceUsd: number | null;
  nativeValueUsd: number | null;
  totalPortfolioUsd: number | null;
  tokens: TokenBalance[];
  chains: ChainSnapshot[];
  accountName: string;
  importKind: ImportKind;
  canRevealMnemonic: boolean;
  enabledNetworks: string[];
  settings: AppSettings;
  hideBalances: boolean;
  explorer: ExplorerKind;
  refresh: () => Promise<void>;
  refreshBalances: () => Promise<void>;
  unlock: (password: string) => Promise<void>;
  lock: () => Promise<void>;
  saveSettings: (next: AppSettings) => Promise<RuntimeConfig>;
  setHideBalances: (hidden: boolean) => Promise<void>;
  changeNetwork: (network: string) => Promise<RuntimeConfig>;
  setEnabledNetworks: (networks: string[]) => Promise<RuntimeConfig>;
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
  const [networks, setNetworks] = useState<NetworkInfo[]>([]);
  const [nativeBalance, setNativeBalance] = useState<number | null>(null);
  const [nativeSymbol, setNativeSymbol] = useState("SOL");
  const [nativePriceUsd, setNativePriceUsd] = useState<number | null>(null);
  const [nativeValueUsd, setNativeValueUsd] = useState<number | null>(null);
  const [totalPortfolioUsd, setTotalPortfolioUsd] = useState<number | null>(null);
  const [tokens, setTokens] = useState<TokenBalance[]>([]);
  const [chains, setChains] = useState<ChainSnapshot[]>([]);
  const [accountName, setAccountName] = useState("Account 1");
  const [importKind, setImportKind] = useState<ImportKind>("mnemonic");
  const [canRevealMnemonic, setCanRevealMnemonic] = useState(false);
  const [enabledNetworks, setEnabledNetworksState] = useState<string[]>(
    DEFAULT_SETTINGS.enabled_networks ?? [],
  );
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const idleTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const unlockedRef = useRef(false);
  const autoLockMinutesRef = useRef(DEFAULT_AUTO_LOCK_MINUTES);
  const settingsRef = useRef(settings);
  const restoredWindowSize = useRef(false);
  const networkRef = useRef(DEFAULT_NETWORK_ID);
  const refreshGen = useRef(0);

  settingsRef.current = settings;

  useEffect(() => {
    unlockedRef.current = unlocked;
  }, [unlocked]);

  useEffect(() => {
    autoLockMinutesRef.current = normalizeAutoLockMinutes(settings.auto_lock_minutes);
  }, [settings.auto_lock_minutes]);

  const applySnapshot = useCallback(
    (snapshot: Awaited<ReturnType<typeof walletApi.getWalletSnapshot>>) => {
      const snapNet = normalizeNetworkId(snapshot.network);
      setWalletExists(snapshot.exists);
      setUnlocked(snapshot.unlocked);
      setTotalPortfolioUsd(snapshot.total_portfolio_usd);
      setChains(snapshot.chains ?? []);
      setAccountName(snapshot.account_name || "Account 1");
      setImportKind(snapshot.import_kind ?? "mnemonic");
      setCanRevealMnemonic(Boolean(snapshot.can_reveal_mnemonic));
      setEnabledNetworksState(snapshot.enabled_networks ?? DEFAULT_SETTINGS.enabled_networks ?? []);

      const wanted = networkRef.current;
      if (snapNet !== wanted) {
        const chain = (snapshot.chains ?? []).find((c) => c.network === wanted);
        if (chain) {
          setPublicKey(chain.public_key);
          setNativeBalance(chain.native_balance);
          setNativeSymbol(chain.native_symbol || "SOL");
          setNativePriceUsd(chain.native_price_usd);
          setNativeValueUsd(chain.native_value_usd);
          setTokens(chain.tokens ?? []);
        }
        return;
      }

      networkRef.current = snapNet;
      setNetwork(snapNet);
      setPublicKey(snapshot.public_key);
      setNativeBalance(snapshot.native_balance);
      setNativeSymbol(snapshot.native_symbol || "SOL");
      setNativePriceUsd(snapshot.native_price_usd);
      setNativeValueUsd(snapshot.native_value_usd);
      setTokens(snapshot.tokens ?? []);
    },
    [],
  );

  const refresh = useCallback(async () => {
    const gen = ++refreshGen.current;
    setBalancesLoading(true);
    try {
      const snapshot = await walletApi.getWalletSnapshot();
      if (gen !== refreshGen.current) return;
      applySnapshot(snapshot);
    } finally {
      if (gen === refreshGen.current) {
        setBalancesLoading(false);
      }
    }
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
        network: normalizeNetworkId(next.network ?? DEFAULT_SETTINGS.network),
        rpc_urls: next.rpc_urls ?? {},
        enabled_networks: next.enabled_networks ?? DEFAULT_SETTINGS.enabled_networks,
        zerox_api_key: next.zerox_api_key ?? null,
        swap_favorite_tokens: next.swap_favorite_tokens ?? [],
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
      network: normalizeNetworkId(next.network),
      enabled_networks: next.enabled_networks ?? DEFAULT_SETTINGS.enabled_networks,
      zerox_api_key: next.zerox_api_key ?? null,
    };
    const runtime = await walletApi.updateAppSettings(payload);
    setSettings(payload);
    return runtime;
  }, []);

  const setHideBalances = useCallback(
    async (hidden: boolean) => {
      const previous = settingsRef.current;
      const next = { ...previous, hide_balances: hidden };
      setSettings(next);
      try {
        await saveSettings(next);
      } catch (err) {
        setSettings(previous);
        throw err;
      }
    },
    [saveSettings],
  );

  const changeNetwork = useCallback(
    async (nextNetwork: string) => {
      const id = normalizeNetworkId(nextNetwork);
      if (id === networkRef.current) {
        return {
          rpc_url: settingsRef.current.rpc_url?.trim() || "",
          jupiter_api_key: settingsRef.current.jupiter_api_key ?? null,
        };
      }
      networkRef.current = id;
      setNetwork(id);
      // Drop in-flight snapshots so they cannot snap the tag back to the previous chain.
      refreshGen.current += 1;
      const runtime = await walletApi.changeWalletNetwork(id);
      if (networkRef.current !== id) {
        return runtime;
      }
      void reloadSettings();
      void refresh();
      return runtime;
    },
    [refresh, reloadSettings],
  );

  const setEnabledNetworks = useCallback(
    async (next: string[]) => {
      const runtime = await walletApi.setEnabledNetworks(next);
      setEnabledNetworksState(next);
      void reloadSettings();
      void refresh();
      return runtime;
    },
    [refresh, reloadSettings],
  );

  const lock = useCallback(async () => {
    if (idleTimer.current) {
      clearTimeout(idleTimer.current);
      idleTimer.current = null;
    }
    await walletApi.lockWallet();
    setUnlocked(false);
    setPublicKey(null);
    setNativeBalance(null);
    setNativePriceUsd(null);
    setNativeValueUsd(null);
    setTotalPortfolioUsd(null);
    setTokens([]);
    setChains([]);
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
      try {
        const listed = await walletApi.listNetworks();
        setNetworks(listed);
      } catch {
        setNetworks([]);
      }
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
      void refresh();
    },
    [refresh],
  );

  const networkInfo = useMemo(() => findNetwork(networks, network), [networks, network]);

  const value = useMemo(
    () => ({
      loading,
      balancesLoading,
      walletExists,
      unlocked,
      publicKey,
      network,
      networks,
      networkInfo,
      nativeBalance,
      nativeSymbol,
      nativePriceUsd,
      nativeValueUsd,
      totalPortfolioUsd,
      tokens,
      chains,
      accountName,
      importKind,
      canRevealMnemonic,
      enabledNetworks,
      settings,
      hideBalances: Boolean(settings.hide_balances),
      explorer: normalizeExplorer(settings.explorer),
      refresh,
      refreshBalances,
      unlock,
      lock,
      saveSettings,
      setHideBalances,
      changeNetwork,
      setEnabledNetworks,
    }),
    [
      loading,
      balancesLoading,
      walletExists,
      unlocked,
      publicKey,
      network,
      networks,
      networkInfo,
      nativeBalance,
      nativeSymbol,
      nativePriceUsd,
      nativeValueUsd,
      totalPortfolioUsd,
      tokens,
      chains,
      accountName,
      importKind,
      canRevealMnemonic,
      enabledNetworks,
      settings,
      refresh,
      refreshBalances,
      unlock,
      lock,
      saveSettings,
      setHideBalances,
      changeNetwork,
      setEnabledNetworks,
    ],
  );

  return <WalletContext.Provider value={value}>{children}</WalletContext.Provider>;
}

export function useWallet() {
  const ctx = useContext(WalletContext);
  if (!ctx) throw new Error("useWallet must be used within WalletProvider");
  return ctx;
}
