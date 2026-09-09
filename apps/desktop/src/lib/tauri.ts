import { commands } from "@/bindings";
import type { AppSettings, OnboardingDraft, Result } from "@/bindings";
export type {
  ActivityItem,
  ApiError,
  AppSettings,
  ChainSnapshot,
  ExplorerKind,
  ImportKind,
  NetworkInfo,
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
  createWallet: (mnemonic: string, password: string, accountName: string) =>
    unwrap(commands.createWallet(mnemonic, password, accountName)),
  importWallet: (mnemonic: string, password: string, accountName: string) =>
    unwrap(commands.importWallet(mnemonic, password, accountName)),
  importPrivateKey: (secret: string, password: string, accountName: string) =>
    unwrap(commands.importPrivateKey(secret, password, accountName)),
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
  resetLocalWallet: () => unwrap(commands.resetLocalWallet()),
  changeWalletPassword: (oldPassword: string, newPassword: string) =>
    unwrap(commands.changeWalletPassword(oldPassword, newPassword)),
  exportWalletToPath: (password: string, path: string) =>
    unwrap(commands.exportWalletToPath(password, path)),
  changeWalletNetwork: (network: string) =>
    unwrap(commands.changeWalletNetwork(network)),
  setEnabledNetworks: (networks: string[]) =>
    unwrap(commands.setEnabledNetworks(networks)),
  listNetworks: () => commands.listNetworks(),
  getActivity: (limit: number) => unwrap(commands.getActivity(limit)),
  previewSend: (to: string, amount: number, asset: string | null) =>
    unwrap(commands.previewSend(to, amount, asset)),
  sendTransfer: (password: string, to: string, amount: number, asset: string | null) =>
    unwrap(commands.sendTransfer(password, to, amount, asset)),
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
  getManagedDefaultRpcUrl: (network?: string | null) =>
    commands.getManagedDefaultRpcUrl(network ?? null),
  setOnboardingDraft: (draft: OnboardingDraft) =>
    unwrap(commands.setOnboardingDraft(draft)),
  getOnboardingDraft: () => unwrap(commands.getOnboardingDraft()),
  clearOnboardingDraft: () => unwrap(commands.clearOnboardingDraft()),
};
