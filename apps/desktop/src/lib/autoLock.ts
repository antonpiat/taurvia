/** Default idle minutes before the wallet locks. */
export const DEFAULT_AUTO_LOCK_MINUTES = 5;

/**
 * Normalize auto-lock minutes from settings.
 * - missing / null / invalid → 5 (default)
 * - 0 → off
 * - positive → that many minutes
 */
export function normalizeAutoLockMinutes(value: unknown): number {
  if (value === 0 || value === "0") return 0;
  if (typeof value === "number" && Number.isFinite(value) && value > 0) {
    return Math.floor(value);
  }
  if (typeof value === "string" && value.trim() !== "") {
    const parsed = Number(value);
    if (Number.isFinite(parsed) && parsed > 0) return Math.floor(parsed);
    if (parsed === 0) return 0;
  }
  return DEFAULT_AUTO_LOCK_MINUTES;
}
