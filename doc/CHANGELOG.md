# Changelog

All notable changes to Taurvia (formerly Aegis) are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Fixed

### Security

---

## [0.4.0] - 2026-07-11

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
5. Build and test: `cargo test` and `pnpm tauri build`.
6. Create a GitHub Release tagged `vx.y.z` and attach binaries from `target/release/bundle/`.
7. Copy the new section into the GitHub Release notes.

[Unreleased]: https://github.com/antonpiat/taurvia/compare/v0.4.0...HEAD
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
