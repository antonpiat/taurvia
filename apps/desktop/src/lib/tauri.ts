import { commands } from "@/bindings";
import type { AppSettings, Network, Result } from "@/bindings";
export type {
  ActivityItem,
  ApiError,
  AppSettings,
  ExplorerKind,
  Network,
  RuntimeConfig,
  SendPreview,
  SwapQuote,
  TokenBalance,
  TokenInfo,
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
  generateMnemonic: () => unwrap(commands.generateMnemonic()),
  validateMnemonic: (mnemonic: string) => unwrap(commands.validateMnemonic(mnemonic)),
  createWallet: (mnemonic: string, password: string) =>
    unwrap(commands.createWallet(mnemonic, password)),
  importWallet: (mnemonic: string, password: string) =>
    unwrap(commands.importWallet(mnemonic, password)),
  importWalletBackup: (walletJson: string, password: string) =>
    unwrap(commands.importWalletBackup(walletJson, password)),
  unlockWallet: (password: string) => unwrap(commands.unlockWallet(password)),
  lockWallet: () => commands.lockWallet(),
  revealMnemonic: (password: string) => unwrap(commands.revealMnemonic(password)),
  deviceProtectionEnabled: () => commands.deviceProtectionEnabled(),
  enableDeviceProtection: (password: string) =>
    unwrap(commands.enableDeviceProtection(password)),
  disableDeviceProtection: (password: string) =>
    unwrap(commands.disableDeviceProtection(password)),
  removeWallet: (password: string) => unwrap(commands.removeWallet(password)),
  resetLocalWallet: () => unwrap(commands.resetLocalWallet()),
  changeWalletPassword: (oldPassword: string, newPassword: string) =>
    unwrap(commands.changeWalletPassword(oldPassword, newPassword)),
  exportWalletToPath: (password: string, path: string) =>
    unwrap(commands.exportWalletToPath(password, path)),
  changeWalletNetwork: (network: Network) =>
    unwrap(commands.changeWalletNetwork(network)),
  getActivity: (limit: number) => unwrap(commands.getActivity(limit)),
  previewSolSend: (to: string, amountSol: number) =>
    unwrap(commands.previewSolSend(to, amountSol)),
  previewSplSend: (mint: string, to: string, amount: number) =>
    unwrap(commands.previewSplSend(mint, to, amount)),
  sendSol: (password: string, to: string, amountSol: number) =>
    unwrap(commands.sendSol(password, to, amountSol)),
  sendSpl: (password: string, mint: string, to: string, amount: number) =>
    unwrap(commands.sendSpl(password, mint, to, amount)),
  resolveToken: (mint: string) => unwrap(commands.resolveToken(mint)),
  searchTokens: (query: string) => unwrap(commands.searchTokens(query)),
  previewSwapQuote: (
    inputMint: string,
    outputMint: string,
    amountUi: number,
    slippageBps: number,
  ) => unwrap(commands.previewSwapQuote(inputMint, outputMint, amountUi, slippageBps)),
  executeSwap: (
    password: string,
    inputMint: string,
    outputMint: string,
    amountUi: number,
    slippageBps: number,
  ) => unwrap(commands.executeSwap(password, inputMint, outputMint, amountUi, slippageBps)),
  getAppSettings: () => unwrap(commands.getAppSettings()),
  updateAppSettings: (settings: AppSettings) =>
    unwrap(commands.updateAppSettings(settings)),
  getManagedDefaultRpcUrl: (network?: Network | null) =>
    commands.getManagedDefaultRpcUrl(network ?? null),
  setOnboardingDraft: (mnemonic: string, mode: string) =>
    unwrap(commands.setOnboardingDraft(mnemonic, mode)),
  getOnboardingDraft: () => unwrap(commands.getOnboardingDraft()),
  clearOnboardingDraft: () => unwrap(commands.clearOnboardingDraft()),
};
