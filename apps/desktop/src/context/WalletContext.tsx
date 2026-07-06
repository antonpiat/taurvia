import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { ApiError, TokenBalance, walletApi } from "@/lib/tauri";

interface WalletContextValue {
  loading: boolean;
  walletExists: boolean;
  unlocked: boolean;
  publicKey: string | null;
  solBalance: number | null;
  tokens: TokenBalance[];
  error: string | null;
  refresh: () => Promise<void>;
  refreshBalances: () => Promise<void>;
  unlock: (password: string) => Promise<void>;
  lock: () => Promise<void>;
  clearError: () => void;
}

const WalletContext = createContext<WalletContextValue | null>(null);

export function WalletProvider({ children }: { children: React.ReactNode }) {
  const [loading, setLoading] = useState(true);
  const [walletExists, setWalletExists] = useState(false);
  const [unlocked, setUnlocked] = useState(false);
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [solBalance, setSolBalance] = useState<number | null>(null);
  const [tokens, setTokens] = useState<TokenBalance[]>([]);
  const [error, setError] = useState<string | null>(null);
  const refreshPromise = useRef<Promise<void> | null>(null);

  const applySnapshot = useCallback(
    (snapshot: Awaited<ReturnType<typeof walletApi.getWalletSnapshot>>) => {
      setWalletExists(snapshot.exists);
      setUnlocked(snapshot.unlocked);
      setPublicKey(snapshot.public_key);
      setSolBalance(snapshot.sol_balance);
      setTokens(snapshot.tokens ?? []);
    },
    [],
  );

  const refreshBalances = useCallback(async () => {
    if (!walletExists || !unlocked) return;
    const [balance, tokenBalances] = await Promise.all([
      walletApi.getSolBalance(),
      walletApi.getTokenBalances(),
    ]);
    setSolBalance(balance);
    setTokens(tokenBalances);
  }, [walletExists, unlocked]);

  const refresh = useCallback(async () => {
    if (refreshPromise.current) {
      return refreshPromise.current;
    }

    const run = (async () => {
      try {
        const snapshot = await walletApi.getWalletSnapshot();
        applySnapshot(snapshot);
      } catch (err) {
        const apiError = err as ApiError;
        setError(apiError.message ?? "Failed to load wallet state");
      } finally {
        refreshPromise.current = null;
      }
    })();

    refreshPromise.current = run;
    return run;
  }, [applySnapshot]);

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
      setPublicKey(key);
      setUnlocked(true);
      setWalletExists(true);
      const [balance, tokenBalances] = await Promise.all([
        walletApi.getSolBalance(),
        walletApi.getTokenBalances(),
      ]);
      setSolBalance(balance);
      setTokens(tokenBalances);
    },
    [],
  );

  const lock = useCallback(async () => {
    await walletApi.lockWallet();
    setUnlocked(false);
    setPublicKey(null);
    setSolBalance(null);
    setTokens([]);
  }, []);

  const clearError = useCallback(() => setError(null), []);

  const value = useMemo(
    () => ({
      loading,
      walletExists,
      unlocked,
      publicKey,
      solBalance,
      tokens,
      error,
      refresh,
      refreshBalances,
      unlock,
      lock,
      clearError,
    }),
    [
      loading,
      walletExists,
      unlocked,
      publicKey,
      solBalance,
      tokens,
      error,
      refresh,
      refreshBalances,
      unlock,
      lock,
      clearError,
    ],
  );

  return <WalletContext.Provider value={value}>{children}</WalletContext.Provider>;
}

export function useWallet() {
  const ctx = useContext(WalletContext);
  if (!ctx) throw new Error("useWallet must be used within WalletProvider");
  return ctx;
}
