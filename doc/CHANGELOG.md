# Changelog

All notable changes to Aegis are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Fixed

### Security

---

## [0.1.1] - 2026-07-09

### Changed

- Switched Solana RPC to `solana_client::nonblocking::rpc_client::RpcClient` with Tokio
- Made wallet-core RPC methods and Tauri RPC commands fully async
- Replaced thread/`rayon` RPC parallelism with `tokio::join!` and bounded `buffer_unordered` concurrency
- Share a single `Arc<RpcClient>` instead of creating a client (and hidden runtime) per request

### Fixed

- Long send/confirm and activity fetches no longer block Tauri worker threads

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
2. Update the version in `apps/desktop/package.json`, `apps/desktop/src-tauri/tauri.conf.json`, and `apps/desktop/src-tauri/Cargo.toml`.
3. Build and test: `cargo test` and `pnpm tauri build`.
4. Create a GitHub Release tagged `vx.y.z` and attach binaries from `target/release/bundle/`.
5. Copy the new section into the GitHub Release notes.

[Unreleased]: https://github.com/antonpiat/aegis/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/antonpiat/aegis/releases/tag/v0.1.1
[0.1.0]: https://github.com/antonpiat/aegis/releases/tag/v0.1.0
