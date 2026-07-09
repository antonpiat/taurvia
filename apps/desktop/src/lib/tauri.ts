import { commands } from "@/bindings";
import type { Result } from "@/bindings";
export type {
  ActivityItem,
  ApiError,
  SendPreview,
  SendResult,
  TokenBalance,
  WalletFile,
  WalletSnapshot,
} from "@/bindings";

async function unwrap<T>(promise: Promise<Result<T, unknown>> | Promise<T>): Promise<T> {
  const value = await promise;
  if (
    value !== null &&
    typeof value === "object" &&
    "status" in value &&
    ((value as Result<T, unknown>).status === "ok" ||
      (value as Result<T, unknown>).status === "error")
  ) {
    const result = value as Result<T, unknown>;
    if (result.status === "ok") {
      return result.data;
    }
    throw result.error;
  }
  return value as T;
}

export const walletApi = {
  getWalletSnapshot: () => unwrap(commands.getWalletSnapshot()),
  walletExists: () => commands.walletExists(),
  generateMnemonic: () => unwrap(commands.generateMnemonic()),
  createWallet: (mnemonic: string, password: string) =>
    unwrap(commands.createWallet(mnemonic, password)),
  importWallet: (mnemonic: string, password: string) =>
    unwrap(commands.importWallet(mnemonic, password)),
  unlockWallet: (password: string) => unwrap(commands.unlockWallet(password)),
  lockWallet: () => commands.lockWallet(),
  isUnlocked: () => commands.isUnlocked(),
  getPublicKey: () => commands.getPublicKey(),
  revealMnemonic: (password: string) => unwrap(commands.revealMnemonic(password)),
  getSolBalance: () => unwrap(commands.getSolBalance()),
  getTokenBalances: () => unwrap(commands.getTokenBalances()),
  getActivity: (limit: number) => unwrap(commands.getActivity(limit)),
  previewSolSend: (to: string, amountSol: number) =>
    unwrap(commands.previewSolSend(to, amountSol)),
  previewSplSend: (mint: string, to: string, amount: number) =>
    unwrap(commands.previewSplSend(mint, to, amount)),
  sendSol: (password: string, to: string, amountSol: number) =>
    unwrap(commands.sendSol(password, to, amountSol)),
  sendSpl: (password: string, mint: string, to: string, amount: number) =>
    unwrap(commands.sendSpl(password, mint, to, amount)),
};
