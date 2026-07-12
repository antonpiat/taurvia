/**
 * Phantom-style password rules for wallet encryption passwords.
 * At least 8 characters with upper, lower, number, and special character.
 */

export type PasswordRequirementId =
  | "length"
  | "uppercase"
  | "lowercase"
  | "number"
  | "special";

export type PasswordRequirement = {
  id: PasswordRequirementId;
  label: string;
  met: boolean;
};

const SPECIAL_CHAR = /[^A-Za-z0-9]/;

export function getPasswordRequirements(password: string): PasswordRequirement[] {
  return [
    {
      id: "length",
      label: "At least 8 characters",
      met: password.length >= 8,
    },
    {
      id: "uppercase",
      label: "1 uppercase letter",
      met: /[A-Z]/.test(password),
    },
    {
      id: "lowercase",
      label: "1 lowercase letter",
      met: /[a-z]/.test(password),
    },
    {
      id: "number",
      label: "1 number",
      met: /[0-9]/.test(password),
    },
    {
      id: "special",
      label: "1 special character",
      met: SPECIAL_CHAR.test(password),
    },
  ];
}

export function isPasswordStrong(password: string): boolean {
  return getPasswordRequirements(password).every((req) => req.met);
}

export function passwordStrengthError(password: string): string | null {
  if (isPasswordStrong(password)) return null;
  return "Password must contain at least 8 characters including 1 uppercase letter, 1 lowercase letter, 1 number, and 1 special character";
}
