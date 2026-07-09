const PHRASE_KEYS = ["mnemonic", "seedPhrase", "recoveryPhrase", "phrase"] as const;

export function normalizeRecoveryPhrase(raw: string): string {
  return raw.trim().toLowerCase().replace(/\s+/g, " ");
}

function isAegisWalletFile(value: unknown): boolean {
  if (!value || typeof value !== "object") return false;
  const crypto = (value as { crypto?: unknown }).crypto;
  if (!crypto || typeof crypto !== "object") return false;
  return typeof (crypto as { ciphertext?: unknown }).ciphertext === "string";
}

function extractFromJson(value: unknown): string {
  if (typeof value === "string") {
    return value;
  }

  if (Array.isArray(value)) {
    if (value.every((item) => typeof item === "string")) {
      return value.join(" ");
    }
    throw new Error("JSON array must contain only recovery words");
  }

  if (!value || typeof value !== "object") {
    throw new Error("Unsupported JSON recovery format");
  }

  if (isAegisWalletFile(value)) {
    throw new Error(
      "This looks like an Aegis wallet file, not a recovery phrase. Use Import with your seed phrase instead.",
    );
  }

  const record = value as Record<string, unknown>;
  for (const key of PHRASE_KEYS) {
    const candidate = record[key];
    if (typeof candidate === "string" && candidate.trim()) {
      return candidate;
    }
    if (Array.isArray(candidate) && candidate.every((item) => typeof item === "string")) {
      return candidate.join(" ");
    }
  }

  throw new Error(
    "Could not find a recovery phrase in JSON. Expected mnemonic, seedPhrase, recoveryPhrase, phrase, or a word array.",
  );
}

export function parseRecoveryFile(filename: string, contents: string): string {
  const lower = filename.toLowerCase();
  if (lower.endsWith(".txt")) {
    return normalizeRecoveryPhrase(contents);
  }
  if (lower.endsWith(".json")) {
    let parsed: unknown;
    try {
      parsed = JSON.parse(contents);
    } catch {
      throw new Error("Invalid JSON file");
    }
    return normalizeRecoveryPhrase(extractFromJson(parsed));
  }
  throw new Error("Unsupported file type. Use a .txt or .json recovery file.");
}
