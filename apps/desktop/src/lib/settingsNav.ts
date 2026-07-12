export type SettingsSectionId =
  | "view"
  | "wallet"
  | "security"
  | "transactions"
  | "network"
  | "advanced"
  | "danger";

export const SETTINGS_SECTIONS: Array<{
  id: SettingsSectionId;
  label: string;
  hint: string;
}> = [
  { id: "view", label: "View", hint: "Layout" },
  { id: "wallet", label: "Wallet", hint: "Session & tokens" },
  { id: "security", label: "Security", hint: "Password & backup" },
  { id: "transactions", label: "Transactions", hint: "Slippage & explorer" },
  { id: "network", label: "Network", hint: "Cluster & RPC" },
  { id: "advanced", label: "Advanced", hint: "Overrides" },
  { id: "danger", label: "Danger zone", hint: "Remove wallet" },
];

export const DEFAULT_SETTINGS_SECTION: SettingsSectionId = "view";

export function isSettingsSectionId(value: string | undefined): value is SettingsSectionId {
  return SETTINGS_SECTIONS.some((section) => section.id === value);
}
