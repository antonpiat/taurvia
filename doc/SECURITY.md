# Security Policy

Taurvia is a **non-custodial desktop wallet**. Your keys stay on your machine. Because this software handles cryptocurrency, we take security reports seriously.

## Supported versions

| Version | Supported |
| ------- | --------- |
| 0.5.x   | Yes       |
| < 0.5   | No        |

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
- Solana network, RPC provider, Esplora, Etherscan-compatible APIs, or other third-party infrastructure bugs
- Loss of funds from user error (wrong address, leaked seed written on paper, etc.)
- Issues in unreleased or modified builds not matching an official GitHub Release

## Security model (summary)

```
React UI  →  Tauri IPC  →  wallet-core (Rust)  →  crypto / storage / family crates
```

The IPC boundary is the trust boundary. The React UI never receives mnemonics, private keys, or raw signers. Specta types are public: addresses, balances, previews, txids. Seed reveal is the only exception, still password + re-decrypt from disk, still not stored in session.

- Private keys and mnemonics are encrypted at rest on disk (`wallet.json`) in **one envelope**. New chains do not get separate wallet files or weaker encryption.
- Unlock from a mnemonic derives a **family keyring** (Solana ed25519 + EVM secp256k1 + Bitcoin mainnet/testnet + Sui SLIP-0010 ed25519), then drops the mnemonic. A key-only import holds **that family only**. Unlock returns immediately; the portfolio snapshot fetches **enabled** families in parallel.
- Revealing the recovery phrase always re-authenticates with the wallet password and re-decrypts from disk (ephemeral); plaintext is not stored back into the session. Key-only wallets cannot reveal a seed.
- Signing happens only in Rust after password verification for send/swap. Last-used network is metadata for Send/Receive (no password), not an exclusive mode.
- On `lock()`, the session is dropped. EVM, Bitcoin, and Sui secrets use `zeroize`; Solana key material is dropped with the keyring.
- **Address-family checks** live in Rust (`validate_recipient`): a 40-hex `0x` address is rejected on Solana/Sui, a 64-hex Sui address is rejected on Ethereum, `bc1` on Ethereum, base58 on Bitcoin. Receive-page copy is not the control.
- **EIP-155 `chain_id` is taken from the network descriptor**, never from the UI, so a Base payload cannot be signed as Ethereum.
- Swap is allowed when the **from-asset chain** is an enabled mainnet with a backend (Jupiter / 0x / Thorchain). Quotes and signatures stay in Rust.

See the [README](../README.md#security) for the full architecture diagram.

## Out of scope (infrastructure)

RPC providers, Esplora, Sui fullnodes, Etherscan-compatible APIs, Jupiter, 0x, Thorchain, and CoinGecko are **out of scope** as third-party infrastructure. Compromise of an RPC can lie about balances or pending state; it cannot extract keys from a locked wallet. Do not treat a custom RPC as a security boundary.

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

Taurvia depends on Rust crates (Solana SDK, alloy, bitcoin, `aes-gcm`, `argon2`, etc.) and npm packages (Tauri, React). We update dependencies as part of regular maintenance. Report supply-chain or dependency issues through the same private channel above.

## Disclosure policy

- We will credit reporters in the release notes unless they prefer to remain anonymous.
- Please allow reasonable time to patch before public disclosure (typically 90 days or coordinated earlier if a fix is ready).

## Disclaimer

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND. See the [MIT License](../LICENSE).
