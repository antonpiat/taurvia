import React, { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";
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

  const refresh = useCallback(async () => {
    try {
      const exists = await walletApi.walletExists();
      setWalletExists(exists);
      if (!exists) {
        setUnlocked(false);
        setPublicKey(null);
        setSolBalance(null);
        setTokens([]);
        return;
      }

      const isUnlocked = await walletApi.isUnlocked();
      setUnlocked(isUnlocked);
      const key = await walletApi.getPublicKey();
      setPublicKey(key);

      if (isUnlocked) {
        const [balance, tokenBalances] = await Promise.all([
          walletApi.getSolBalance(),
          walletApi.getTokenBalances(),
        ]);
        setSolBalance(balance);
        setTokens(tokenBalances);
      }
    } catch (err) {
      const apiError = err as ApiError;
      setError(apiError.message ?? "Failed to load wallet state");
    }
  }, []);

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
      await refresh();
    },
    [refresh],
  );

  const lock = useCallback(async () => {
    await walletApi.lockWallet();
    setUnlocked(false);
    setSolBalance(null);
    setTokens([]);
    await refresh();
  }, [refresh]);

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
      unlock,
      lock,
      clearError: () => setError(null),
    }),
    [loading, walletExists, unlocked, publicKey, solBalance, tokens, error, refresh, unlock, lock],
  );

  return <WalletContext.Provider value={value}>{children}</WalletContext.Provider>;
}

export function useWallet() {
  const ctx = useContext(WalletContext);
  if (!ctx) throw new Error("useWallet must be used within WalletProvider");
  return ctx;
}
