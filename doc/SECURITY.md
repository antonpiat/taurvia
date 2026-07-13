# Security Policy

Taurvia is a **non-custodial Solana desktop wallet**. Your keys stay on your machine. Because this software handles cryptocurrency, we take security reports seriously.

## Supported versions

| Version | Supported |
| ------- | --------- |
| 0.4.x   | Yes       |
| < 0.1   | No        |

Only the latest release receives security fixes.

## Reporting a vulnerability

**Please do not open a public GitHub issue for security vulnerabilities.**

Instead, use one of these channels:

1. **GitHub private reporting** (preferred): [Report a vulnerability](https://github.com/antonpiat/taurvia/security/advisories/new) on this repository.
2. **Email**: open a private security advisory on GitHub if email contact is not listed in the repository profile.

Include as much detail as you can:

- Affected version(s)
- Steps to reproduce
- Impact (e.g. key exposure, unsigned transaction, encryption bypass)
- Proof of concept if available

We aim to acknowledge reports within **72 hours** and will work with you on a fix and coordinated disclosure when appropriate.

## What we consider in scope

- Unauthorized access to private keys or seed phrases
- Weaknesses in wallet encryption (Argon2id / AES-256-GCM)
- Signing logic flaws (wrong recipient, amount, or program)
- Tauri IPC boundary issues (frontend bypassing Rust checks)
- CSP or webview issues that lead to secret exfiltration in release builds

## Out of scope

- Social engineering or phishing targeting users directly
- Compromise of the user's OS, password, or physical device
- Solana network, RPC provider, or third-party infrastructure bugs
- Loss of funds from user error (wrong address, leaked seed written on paper, etc.)
- Issues in unreleased or modified builds not matching an official GitHub Release

## Security model (summary)

```
React UI  →  Tauri IPC  →  wallet-core (Rust)  →  crypto / storage / taurvia-solana
```

- Private keys and mnemonics are encrypted at rest on disk (`wallet.json`).
- Unlock loads the **keypair only** into the session for signing. The recovery phrase is **not** kept in RAM while unlocked.
- Revealing the recovery phrase always re-authenticates with the wallet password and re-decrypts from disk (ephemeral); plaintext is not stored back into the session.
- Signing happens only in Rust after password verification for send/swap.
- The frontend does not receive raw secrets except when you explicitly reveal the phrase.
- Switching Mainnet ↔ Devnet updates wallet metadata + RPC only (no password); the keypair is unchanged. Devnet is not real funds; Swap is Mainnet-only (enforced in Rust, not only in the UI).

See the [README](../README.md#security) for the full architecture diagram.

## Wallet encryption

**Default (password-only):** `K = Argon2id(password, salt)` → AES-256-GCM. Anyone with the encrypted JSON and the password can decrypt the wallet on any machine.

**Enhanced device protection (optional):** a random `device_secret` is stored only in the OS credential store (keychain / Secret Service / Credential Manager), keyed by wallet id. The file encryption key becomes:

`K = HKDF-SHA256(Argon2id(password, salt) || device_secret, info = "taurvia-wallet-v2")`

The wallet file records `protection: "password-device"` and **never** stores `device_secret`. JSON + password alone are **not** enough to decrypt a device-bound wallet on another machine or after the OS credential store is wiped.

| Scenario | Password-only wallet | Device-bound wallet |
| -------- | -------------------- | ------------------- |
| Unlock on same device with password | Yes | Yes (needs keychain entry) |
| Copy JSON + password to a new PC | Decrypts | Fails — use recovery phrase, or disable protection then re-export on the old device |
| OS reinstall / keychain reset / new device | Still decrypts with password | Local file may be **unrecoverable** without the recovery phrase |
| Import from backup (exported JSON + password) | Works | Only if the same device secret is present; otherwise fail by design |
| Import from recovery phrase | Works anywhere | Works anywhere (sets a new local wallet) |

Malware on an unlocked enrolled device remains high risk: the private key is in memory while unlocked (normal hot-wallet tradeoff). Device binding is OS keychain wrapping, not hardware signing.

## Restore paths

1. **Import from backup** — exported encrypted wallet JSON + that backup’s password. Preferred when moving machines with a password-only export.
2. **Import from recovery phrase** — seed → new password. Use when recovering after device loss, OS reset, or when the backup is device-bound and the secret is gone.

## User responsibilities

- **Back up your seed phrase** offline. Taurvia cannot recover it if lost.
- **Use a strong wallet password** and keep your OS updated.
- **If you enable Enhanced device protection**, understand that OS reinstall, credential-store reset, or replacing the device can make the local wallet file unusable without the recovery phrase. Keep (or create) an offline seed backup before enabling.
- **Verify addresses** before sending funds.
- **Download only from official [GitHub Releases](https://github.com/antonpiat/taurvia/releases)**.
- **Do not store more than you can afford to lose** — this software has not undergone a professional audit.

## Dependency security

Taurvia depends on Rust crates (Solana SDK, `aes-gcm`, `argon2`, etc.) and npm packages (Tauri, React). We update dependencies as part of regular maintenance. Report supply-chain or dependency issues through the same private channel above.

## Disclosure policy

- We will credit reporters in the release notes unless they prefer to remain anonymous.
- Please allow reasonable time to patch before public disclosure (typically 90 days or coordinated earlier if a fix is ready).

## Disclaimer

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND. See the [MIT License](../LICENSE).
