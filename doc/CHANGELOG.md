# Changelog

All notable changes to Aegis are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Jupiter-backed USD prices, token metadata, and portfolio totals on the dashboard
- Any-to-any swap quote and password-gated execute via Jupiter Swap API
- Optional `AEGIS_JUPITER_API_KEY` for higher Jupiter rate limits (keyless by default)

### Changed

### Fixed

### Security

---

## [0.1.2] - 2026-07-09

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

## [0.1.0] - 2026-07-06

First public MVP release.

### Added

- Create and import Solana wallets from a BIP39 seed phrase
- Password-protected unlock and lock screen
- Encrypted local wallet storage (Argon2id + AES-256-GCM)
- SOL and SPL token balance display
- Send SOL and SPL tokens with fee preview and confirmation
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

[Unreleased]: https://github.com/antonpiat/aegis/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/antonpiat/aegis/releases/tag/v0.1.2
[0.1.0]: https://github.com/antonpiat/aegis/releases/tag/v0.1.0
