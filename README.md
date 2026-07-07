<p align="center">
  <img src="doc/AEGIS.png" alt="Aegis Wallet" width="300" />
</p>

<h1 align="center">Aegis</h1>

<p align="center">
  <strong>Secure. Simple. On Solana.</strong>
</p>

<p align="center">
  A non-custodial Solana desktop wallet for retail users — keys stay on your machine, signing stays in Rust.
</p>

<p align="center">
  <a href="#getting-started">Getting started</a> ·
  <a href="#features">Features</a> ·
  <a href="#security">Security</a> ·
  <a href="#architecture">Architecture</a> ·
  <a href="doc/CHANGELOG.md">Changelog</a> ·
  <a href="doc/SECURITY.md">Security policy</a>
</p>

---

## Why Aegis

Most wallets ask you to trust a browser tab or a hosted service. Aegis is a **native desktop app**: your seed phrase and private keys never leave your device, and every signature is produced inside a Rust core the UI cannot bypass.

Built with **Tauri v2** for a small footprint and **Solana SDK 4** for mainnet-ready transactions.

## Features

| | |
|---|---|
| **Create & import** | New wallet or recover from a 12/24-word seed phrase |
| **Balances** | SOL and SPL token holdings via RPC |
| **Send** | SOL and SPL transfers with fee preview and confirmation |
| **Receive** | Address display and QR code |
| **Activity** | Recent on-chain history |
| **Lock screen** | Password-gated unlock, signing, and seed reveal |

## Security

Aegis is designed so the frontend never becomes a secret keeper.

```mermaid
flowchart TB
  UI["React UI<br/>apps/desktop<br/><i>balances · forms · QR</i><br/><b>no private keys</b>"]
  WC["wallet-core<br/><i>unlock · sign · send</i>"]
  CRYPTO["crypto<br/>Argon2id · AES-256-GCM"]
  SOL["solana<br/>RPC · txs · SPL"]
  STORE["storage<br/>encrypted wallet file"]

  UI -->|Tauri IPC| WC
  WC --> CRYPTO
  WC --> SOL
  WC --> STORE
```

- **At rest:** Argon2id key derivation + AES-256-GCM encryption
- **In memory:** keys exist only while the wallet is unlocked
- **At sign time:** transactions are built and signed in Rust, not JavaScript
- **Seed reveal:** requires password verification every time

## Stack

| Layer | Technology |
|-------|------------|
| Shell | Tauri v2 |
| Core | Rust workspace — `crypto`, `storage`, `solana`, `wallet-core`, `models` |
| UI | React 19, TypeScript, Vite, Tailwind CSS 4 |
| Chain | Solana SDK 4, SPL token interfaces |
| Package manager | pnpm |

## Getting started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 20+
- [pnpm](https://pnpm.io/)
- [Tauri system dependencies](https://v2.tauri.app/start/prerequisites/) (Linux)

### Install & run

```bash
git clone <your-repo-url>
cd aegis/apps/desktop
pnpm install
pnpm tauri dev
```

### Optional: custom RPC

By default Aegis uses the public Solana mainnet RPC. For better reliability, point at your own endpoint:

```bash
cp ../../.env.example ../../.env
# AEGIS_RPC_URL=https://mainnet.helius-rpc.com/?api-key=YOUR_KEY
```

### Build

Production bundles (`.deb`, `.rpm`, `.AppImage` on Linux):

```bash
cd apps/desktop
pnpm tauri build
```

Output lands in `target/release/bundle/`.

### Test the Rust workspace

```bash
cd aegis
export TMPDIR=$PWD/.tmp
cargo test
```

## Architecture

```mermaid
flowchart LR
  UI["React UI"] -->|invoke| WC["wallet-core"]
  WC --> CRYPTO["crypto"]
  WC --> STORE["storage"]
  WC --> SOL["solana / RPC"]
  STORE --> DISK[("~/.local/share/com.aegis.wallet")]
```

## Project structure

```
aegis/
├── apps/desktop/          # Tauri shell + React frontend
│   └── src-tauri/         # Rust commands, config, icons
├── crates/
│   ├── crypto/            # Argon2id + AES-256-GCM
│   ├── models/            # shared types
│   ├── solana/            # RPC client, transfers, keypairs
│   ├── storage/           # encrypted wallet file I/O
│   └── wallet-core/       # session, signing, snapshots
└── doc/                   # project docs
    ├── AEGIS.png          # README branding
    ├── CHANGELOG.md       # release history
    └── SECURITY.md        # vulnerability reporting
```

## License

MIT
