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

- Private keys and mnemonics are encrypted at rest on disk.
- Signing happens only in Rust after password verification.
- The frontend does not receive raw secrets.

See the [README](../README.md#security) for the full architecture diagram.

## User responsibilities

- **Back up your seed phrase** offline. Taurvia cannot recover it if lost.
- **Use a strong wallet password** and keep your OS updated.
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
