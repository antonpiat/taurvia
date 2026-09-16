# Changelog

All notable changes to Taurvia (formerly Aegis) are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Sui portfolio used Mysten public fullnodes, which no longer serve JSON-RPC, so activating Sui never appeared on the dashboard. Default RPC is PublicNode (`sui-rpc.publicnode.com`); activated chains still show when an RPC is down.

---

## [0.5.2] - 2026-09-16

### Added

- Sui Mainnet + Testnet: SLIP-0010 Ed25519 from the same seed, JSON-RPC balances and PaySui transfers, Suiscan links. New mnemonic wallets activate Sui with the other mainnets. Swap stays off (`features.swap = false`). Private-key import accepts `suiprivkey1…`.

---

## [0.5.1] - 2026-09-01

### Added

- Phantom-style onboarding: create seed then name + password (no 3-word quiz); restore via recovery phrase, private key, or JSON backup; hardware wallet listed as coming soon
- Wallet file v3: `account_name`, `import_kind`, `enabled_networks`; `import_private_key` for Solana (base58/JSON), Ethereum (`0x` hex), and Bitcoin WIF
- Multi-chain portfolio: activate Solana, Ethereum, and Bitcoin together; dashboard total USD plus per-chain sections
- Swap on Ethereum (0x) and Bitcoin (Thorchain inbound + PSBT); Solana remains Jupiter
- Shared `TokenIcon` with bundled SOL/ETH/BTC majors, chain badges, and initials fallback

### Changed

- Last-used network is for Send/Receive, not an exclusive mode
- Sidebar shows account name; key-only wallets cannot reveal a seed or activate other families
- Swap is available when the from-asset chain is an enabled mainnet, not Solana-only

### Security

- Key-only sessions hold that family only; a raw key cannot derive the other curves
- Quotes and swap signatures stay in Rust; the UI never sees private keys

---

## [0.5.0] - 2026-08-31

### Added

- Ethereum (Mainnet + Sepolia) and Bitcoin (Mainnet + testnet) behind the same Rust signing core — one seed, family keyring, descriptor registry
- `list_networks()` data-driven picker; Polygon, Base, and Sui ship as disabled stubs
- Rust address-family checks (reject `0x` on Solana, `bc1` on Ethereum, and so on)
- BIP39 / BIP44 / BIP84 derivation tests (Phantom/MetaMask/BIP-0084 vectors)

### Changed

- GitHub Actions: [CI](../.github/workflows/ci.yml) validates every PR and `main` push; [Release](../.github/workflows/release.yml) packages only a `vX.Y.Z` tag after green CI (no more auto-build when `tauri.conf.json` changes)
- Product copy: **Secure. Simple. On your machine.** Solana remains the swap chain
- Settings → Network is family-grouped; Advanced RPC is per network id (`rpc_urls`)
- Send preview always includes network name, full recipient, amount, and fee from Rust
- Wallet file v2 stores public addresses per family; v1 files upgrade on first unlock
- Dashboard/activity/preview paths copy addresses only (not signing keys); Bitcoin fee preview no longer builds a signed transaction
- Hide-balances toggle updates the UI immediately, then persists

### Removed

- Dead IPC that the UI never called: `remove_wallet`, `export_wallet` (in-memory JSON), `get_sol_balance`, `get_token_balances`, `preview_sol_send` / `preview_spl_send`, `send_sol` / `send_spl`. Use `reset_local_wallet`, `export_wallet_to_path`, `get_wallet_snapshot`, `preview_send`, and `send_transfer`

### Security

- Session holds derived family signers only; mnemonic is not kept in RAM; lock drops the keyring
- EIP-155 `chain_id` comes from the network descriptor, never the UI
- Swap remains Solana Mainnet-only in Rust (`features.swap` is not enough by itself)

### Fixed

- Swap token search: restore HTTPS for Jupiter by enabling `reqwest` `rustls` (0.13 drop of TLS features broke remote search)
- Token picker shows an error when remote search fails instead of a silent empty list

---

## [0.4.3] - 2026-07-13

### Added

- Enhanced device protection (optional): OS keychain–bound wallet encryption so JSON + password alone cannot decrypt off-device
- Import from backup: restore from exported encrypted wallet JSON + password
- Welcome / onboarding split: Import from backup vs Import from recovery phrase
- Unlock: Forgot password — reset wallet (destructive local wipe without password)

### Changed

- Unlocked session keeps keypair only — recovery phrase is not held in RAM
- Reveal recovery phrase requires password re-auth and re-decrypts from disk
- Settings → Security: device-protection toggle; Reveal is last and destructive-styled
- Export copy points at Import from backup; portable export needs protection off first when device-bound
- Danger zone remove wallet: warning + agreement checkbox (no password); same local wipe as unlock reset

### Security

- Document device-bound threat model, OS-reset / device-replacement risk, and seed vs backup restore matrix in `doc/SECURITY.md`
- Device-bound backup import fails without the local keychain secret (by design)

---

## [0.4.2] - 2026-07-12

### Added

- Settings → Network: switch Solana Mainnet ↔ Devnet (same address; no password)
- Managed Devnet RPC default (`https://api.devnet.solana.com`)
- Phantom-style password strength checks on create/change (upper, lower, number, special)
- Bundled icons for curated Swap tokens (SOL, USDC, USDT, JUP, BONK) for instant paint
- Keyword token search in Swap picker (local filter + debounced Jupiter search)
- Persist Swap favorite tokens in app settings
- Settings → Wallet: manage (remove) user-added Swap tokens

### Changed

- RPC resolution is network-aware (managed default follows active cluster)
- Swap is Mainnet-only (UI + Rust enforcement; Jupiter has no Devnet path here)
- Recovery phrase modal shows seed only while unlocked (no password re-prompt)
- Desktop CI builds only when `tauri.conf.json` product version increases on `main`
- Swap mint paste moved under Advanced; search-in-picker is the primary add path

### Fixed

- `update_app_settings` can no longer desync cluster from the wallet file

### Security

- Wallet/config/backup files written with owner-only permissions on Unix (`0600`)
- Export backup requires an absolute path with an existing parent directory
- Wallet decrypt rejects unknown file version / KDF / cipher metadata

---

## [0.4.1] - 2026-07-11

### Added

- Settings: auto-lock timeout; Dashboard hide-balances toggle (default hidden)
- Settings: change wallet password and export encrypted wallet backup
- Settings: default swap slippage and block explorer preference (Solscan / Solana Explorer)
- Settings: View layout preference (Desktop / Compact / Phone; tracks window size)
- Explorer links on Activity, Send, and Swap confirmations
- GitHub Actions desktop CI builds unsigned Linux, Windows, and macOS packages on `main` pushes (or manual dispatch)

### Changed

- Track `Cargo.lock` for reproducible CI builds
- Desktop CI uses Node.js 24

---

## [0.4.0] - 2026-07-10

### Changed

- Product rebrand: **Aegis → Taurvia** (app name, bundle id `com.taurvia.wallet`, crates, env vars)
- Crate `aegis-solana` renamed to `taurvia-solana`; desktop package `taurvia-desktop`
- Dev env vars: `TAURVIA_RPC_URL`, `TAURVIA_JUPITER_API_KEY` (replacing `AEGIS_*`)
- One-time migration copies wallets/config from legacy `com.aegis.wallet` app data into `com.taurvia.wallet`

---

## [0.3.2] - 2026-07-09

### Fixed

- Pinned the left sidebar to viewport height so the Active wallet panel stays visible on tall pages (Swap, Settings)
- Main content scrolls independently; nav and wallet controls no longer get pushed off-screen

---

## [0.3.1] - 2026-07-08

### Changed

- Responsive shell: bottom nav + compact top bar on narrow windows, icon sidebar on medium, full sidebar on wide
- Lowered minimum window size to `420×560`
- Dashboard, Send, Swap, Receive, Activity, Settings, and dialogs wrap cleanly on smaller widths
- Dashboard shows loading skeletons while balances and market data load

---

## [0.3.0] - 2026-07-08

### Changed

- Unlock returns immediately after password verification; balances and market data load in the background
- Jupiter price/metadata requests use a short timeout budget so a slow API cannot stall the dashboard
- Snapshot enrichment runs metadata, token prices, and SOL price in parallel
- Curated majors (SOL, USDC, USDT, JUP, BONK) resolve locally before remote metadata returns

### Fixed

- Opening the wallet after unlock no longer waits on full Jupiter enrichment
- Failed Jupiter lookups are no longer cached as shortened-mint fallbacks for an hour

---

## [0.2.4] - 2026-07-05

### Added

- Interactive Active wallet footer: copy address, portfolio USD, Mainnet badge, Lock, and Manage
- Manage accounts page (`/accounts`) as an advanced entry point for future ATA tools
- Settings → Advanced link to Manage accounts

---

## [0.2.3] - 2026-07-04

### Added

- SPL send preview detects whether the recipient ATA exists
- Confirm dialog note when a token account will be created automatically: “A token account will be created for this asset.”

### Changed

- SPL send preview token label prefers resolved symbol over shortened mint

---

## [0.2.2] - 2026-07-03

### Fixed

- Token symbols and logos for curated majors no longer depend solely on Jupiter search success
- Send page token picker replaced with a working dropdown (held SPL tokens selectable)
- Swap confirm dialog shows clear You pay / You receive / Route rows with symbols

### Changed

- Shared `TokenDropdown` component used by Send and Swap

---

## [0.2.1] - 2026-07-02

### Changed

- Swap UI redesigned for retail use: custom token dropdowns with logo, symbol, and name
- Amount shortcuts (25% / 50% / Max) and dynamic `Amount (SYMBOL)` label
- Slippage and custom mint moved under Advanced settings (default 0.5%)
- Clearer validation: same-token error, disabled Get quote until inputs are valid
- Friendlier copy for quote review and local signing
- Failed swap errors stay in the confirm dialog instead of sticking on the swap board

---

## [0.2.0] - 2026-07-01

### Added

- Jupiter-backed USD prices, token metadata, and portfolio totals on the dashboard
- Any-to-any swap quote and password-gated execute via Jupiter Swap API
- Optional `AEGIS_JUPITER_API_KEY` for higher Jupiter rate limits (keyless by default)
- Swap page (`/swap`) with held tokens, curated majors, and paste-mint support
- Models: `TokenInfo`, `SwapQuote`, `SwapResult`; USD fields on balances and snapshot
- CSP `img-src https:` for remote token logos

### Changed

- Wallet snapshot enrichment joins RPC balances with metadata and prices
- Tauri commands: `resolve_token`, `preview_swap_quote`, `execute_swap`

---

## [0.1.2] - 2026-06-28

### Added

- Tokio async runtime for Solana RPC and wallet-core network work
- Typed TypeScript bindings generated with `tauri-specta` (`apps/desktop/src/bindings.ts`)
- Specta `Type` derives on shared `models` DTOs for end-to-end command typing

### Changed

- Switched Solana RPC to `solana_client::nonblocking::rpc_client::RpcClient` with Tokio
- Made `wallet-core` RPC methods and Tauri RPC commands fully async
- Replaced thread/`rayon` RPC parallelism with `tokio::join!` and bounded `buffer_unordered` concurrency
- Share a single `Arc<RpcClient>` instead of creating a client (and hidden runtime) per request
- Standardized package name to kebab-case `wallet-core` (Rust import remains `wallet_core`)
- Split `wallet-core` into focused modules (`session`, `wallet_file`, `balances`, `send`)
- Split Tauri IPC into `commands/{wallet,balances,send}.rs`
- Frontend `tauri.ts` now wraps generated bindings instead of hand-rolled invoke helpers

### Fixed

- Long send/confirm and activity fetches no longer block Tauri worker threads
- Clearer separation between wallet session logic and Tauri command glue

---

## [0.1.0] - 2026-06-20

First public MVP release.

### Added

- Create and import Solana wallets from a BIP39 seed phrase
- Password-protected unlock and lock screen
- Encrypted local wallet storage (Argon2id + AES-256-GCM)
- SOL and SPL token balance display
- Send SOL and SPL transfers with fee preview and confirmation
- Receive page with address copy and QR code
- On-chain activity history via Solana RPC
- Password-gated seed phrase reveal
- Tauri v2 desktop shell with React + TypeScript UI
- Linux release bundles: `.AppImage`, `.deb`, `.rpm`

### Fixed

- Production AppImage blank page (Vite `base`, HashRouter, CSP, WebKit GTK workarounds)
- Solana SDK 4.x compatibility and SPL interface crate linkage
- RPC performance: batched mint lookups, parallel balance/activity fetches

### Security

- Signing and key handling confined to the Rust `wallet-core` crate
- Frontend never receives raw private keys or mnemonics

### Known limitations

- Linux desktop only in this release
- No professional security audit yet — use at your own risk
- Public Solana RPC by default; configure `AEGIS_RPC_URL` for production use

---

## Release checklist

When cutting a new version:

1. Move items from **Unreleased** into a new `## [x.y.z] - YYYY-MM-DD` section.
2. Update the version in all workspace `Cargo.toml` packages, `apps/desktop/package.json`, and `apps/desktop/src-tauri/tauri.conf.json`.
3. Refresh `Cargo.lock` with `cargo check` / `cargo test`.
4. Update `README.md` version note if present.
5. Merge to `main` and wait for [CI](../.github/workflows/ci.yml) to pass.
6. Tag and push: `git tag vX.Y.Z && git push origin vX.Y.Z` (tag must match `tauri.conf.json`).
7. [Release](../.github/workflows/release.yml) builds unsigned installers and attaches them to the GitHub Release. Edit the notes if needed.

[Unreleased]: https://github.com/antonpiat/taurvia/compare/v0.5.2...HEAD
[0.5.2]: https://github.com/antonpiat/taurvia/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/antonpiat/taurvia/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/antonpiat/taurvia/compare/v0.4.3...v0.5.0
[0.4.3]: https://github.com/antonpiat/taurvia/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/antonpiat/taurvia/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/antonpiat/taurvia/releases/tag/v0.4.1
[0.4.0]: https://github.com/antonpiat/taurvia/releases/tag/v0.4.0
[0.3.2]: https://github.com/antonpiat/taurvia/releases/tag/v0.3.2
[0.3.1]: https://github.com/antonpiat/taurvia/releases/tag/v0.3.1
[0.3.0]: https://github.com/antonpiat/taurvia/releases/tag/v0.3.0
[0.2.4]: https://github.com/antonpiat/taurvia/releases/tag/v0.2.4
[0.2.3]: https://github.com/antonpiat/taurvia/releases/tag/v0.2.3
[0.2.2]: https://github.com/antonpiat/taurvia/releases/tag/v0.2.2
[0.2.1]: https://github.com/antonpiat/taurvia/releases/tag/v0.2.1
[0.2.0]: https://github.com/antonpiat/taurvia/releases/tag/v0.2.0
[0.1.2]: https://github.com/antonpiat/taurvia/releases/tag/v0.1.2
[0.1.0]: https://github.com/antonpiat/taurvia/releases/tag/v0.1.0
[0.1.0]: https://github.com/antonpiat/taurvia/releases/tag/v0.1.0
