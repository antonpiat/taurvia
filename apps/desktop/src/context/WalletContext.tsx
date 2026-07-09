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
  walletExists: boolean;
  unlocked: boolean;
  publicKey: string | null;
  solBalance: number | null;
  tokens: TokenBalance[];
  refresh: () => Promise<void>;
  refreshBalances: () => Promise<void>;
  unlock: (password: string) => Promise<void>;
  lock: () => Promise<void>;
}

const WalletContext = createContext<WalletContextValue | null>(null);

export function WalletProvider({ children }: { children: React.ReactNode }) {
  const [loading, setLoading] = useState(true);
  const [walletExists, setWalletExists] = useState(false);
  const [unlocked, setUnlocked] = useState(false);
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [solBalance, setSolBalance] = useState<number | null>(null);
  const [tokens, setTokens] = useState<TokenBalance[]>([]);
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

  const refresh = useCallback(async () => {
    if (refreshPromise.current) {
      return refreshPromise.current;
    }

    const run = (async () => {
      try {
        applySnapshot(await walletApi.getWalletSnapshot());
      } finally {
        refreshPromise.current = null;
      }
    })();

    refreshPromise.current = run;
    return run;
  }, [applySnapshot]);

  const refreshBalances = useCallback(async () => {
    if (!walletExists || !unlocked) return;
    const [balance, tokenBalances] = await Promise.all([
      walletApi.getSolBalance(),
      walletApi.getTokenBalances(),
    ]);
    setSolBalance(balance);
    setTokens(tokenBalances);
  }, [walletExists, unlocked]);

  useEffect(() => {
    void (async () => {
      setLoading(true);
      await refresh();
      setLoading(false);
    })();
  }, [refresh]);

  const unlock = useCallback(
    async (password: string) => {
      await walletApi.unlockWallet(password);
      await refresh();
    },
    [refresh],
  );

  const lock = useCallback(async () => {
    await walletApi.lockWallet();
    setUnlocked(false);
    setPublicKey(null);
    setSolBalance(null);
    setTokens([]);
  }, []);

  const value = useMemo(
    () => ({
      loading,
      walletExists,
      unlocked,
      publicKey,
      solBalance,
      tokens,
      refresh,
      refreshBalances,
      unlock,
      lock,
    }),
    [
      loading,
      walletExists,
      unlocked,
      publicKey,
      solBalance,
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
