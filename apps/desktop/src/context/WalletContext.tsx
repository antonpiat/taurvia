import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { TokenBalance, walletApi } from "@/lib/tauri";

interface WalletContextValue {
  loading: boolean;
  balancesLoading: boolean;
  walletExists: boolean;
  unlocked: boolean;
  publicKey: string | null;
  solBalance: number | null;
  solPriceUsd: number | null;
  solValueUsd: number | null;
  totalPortfolioUsd: number | null;
  tokens: TokenBalance[];
  refresh: () => Promise<void>;
  refreshBalances: () => Promise<void>;
  unlock: (password: string) => Promise<void>;
  lock: () => Promise<void>;
}

const WalletContext = createContext<WalletContextValue | null>(null);

export function WalletProvider({ children }: { children: React.ReactNode }) {
  const [loading, setLoading] = useState(true);
  const [balancesLoading, setBalancesLoading] = useState(false);
  const [walletExists, setWalletExists] = useState(false);
  const [unlocked, setUnlocked] = useState(false);
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [solBalance, setSolBalance] = useState<number | null>(null);
  const [solPriceUsd, setSolPriceUsd] = useState<number | null>(null);
  const [solValueUsd, setSolValueUsd] = useState<number | null>(null);
  const [totalPortfolioUsd, setTotalPortfolioUsd] = useState<number | null>(null);
  const [tokens, setTokens] = useState<TokenBalance[]>([]);
  const refreshPromise = useRef<Promise<void> | null>(null);

  const applySnapshot = useCallback(
    (snapshot: Awaited<ReturnType<typeof walletApi.getWalletSnapshot>>) => {
      setWalletExists(snapshot.exists);
      setUnlocked(snapshot.unlocked);
      setPublicKey(snapshot.public_key);
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

  useEffect(() => {
    void (async () => {
      setLoading(true);
      await refresh();
      setLoading(false);
    })();
  }, [refresh]);

  const unlock = useCallback(
    async (password: string) => {
      const key = await walletApi.unlockWallet(password);
      // Unlock should feel instant: enter the app immediately, then load balances.
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

  const lock = useCallback(async () => {
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

  const value = useMemo(
    () => ({
      loading,
      balancesLoading,
      walletExists,
      unlocked,
      publicKey,
      solBalance,
      solPriceUsd,
      solValueUsd,
      totalPortfolioUsd,
      tokens,
      refresh,
      refreshBalances,
      unlock,
      lock,
    }),
    [
      loading,
      balancesLoading,
      walletExists,
      unlocked,
      publicKey,
      solBalance,
      solPriceUsd,
      solValueUsd,
      totalPortfolioUsd,
      tokens,
      refresh,
      refreshBalances,
      unlock,
      lock,
    ],
  );

  return <WalletContext.Provider value={value}>{children}</WalletContext.Provider>;
}

export function useWallet() {
  const ctx = useContext(WalletContext);
  if (!ctx) throw new Error("useWallet must be used within WalletProvider");
  return ctx;
}
