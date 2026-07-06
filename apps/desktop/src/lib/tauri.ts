import { invoke } from "@tauri-apps/api/core";

export interface ApiError {
  code: string;
  message: string;
}

export interface TokenBalance {
  mint: string;
  symbol: string;
  name: string;
  amount: string;
  decimals: number;
  ui_amount: number;
}

export interface ActivityItem {
  signature: string;
  timestamp: number | null;
  status: string;
  direction: string;
  amount_sol: number | null;
  description: string;
}

export interface SendPreview {
  from: string;
  to: string;
  token: string;
  amount: string;
  estimated_fee_lamports: number;
  estimated_fee_sol: number;
}

export interface SendResult {
  signature: string;
  status: string;
}

async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    if (typeof error === "object" && error !== null && "code" in error && "message" in error) {
      throw error as ApiError;
    }
    throw { code: "unknown", message: String(error) } satisfies ApiError;
  }
}

export const walletApi = {
  walletExists: () => invokeCommand<boolean>("wallet_exists"),
  generateMnemonic: () => invokeCommand<string>("generate_mnemonic"),
  createWallet: (mnemonic: string, password: string) =>
    invokeCommand("create_wallet", { mnemonic, password }),
  importWallet: (mnemonic: string, password: string) =>
    invokeCommand("import_wallet", { mnemonic, password }),
  unlockWallet: (password: string) => invokeCommand<string>("unlock_wallet", { password }),
  lockWallet: () => invokeCommand<void>("lock_wallet"),
  isUnlocked: () => invokeCommand<boolean>("is_unlocked"),
  getPublicKey: () => invokeCommand<string | null>("get_public_key"),
  revealMnemonic: (password: string) => invokeCommand<string>("reveal_mnemonic", { password }),
  getSolBalance: () => invokeCommand<number>("get_sol_balance"),
  getTokenBalances: () => invokeCommand<TokenBalance[]>("get_token_balances"),
  getActivity: (limit: number) => invokeCommand<ActivityItem[]>("get_activity", { limit }),
  previewSolSend: (to: string, amountSol: number) =>
    invokeCommand<SendPreview>("preview_sol_send", { to, amountSol }),
  previewSplSend: (mint: string, to: string, amount: number) =>
    invokeCommand<SendPreview>("preview_spl_send", { mint, to, amount }),
  sendSol: (password: string, to: string, amountSol: number) =>
    invokeCommand<SendResult>("send_sol", { password, to, amountSol }),
  sendSpl: (password: string, mint: string, to: string, amount: number) =>
    invokeCommand<SendResult>("send_spl", { password, mint, to, amount }),
};
