# Aegis

Aegis is a security-first, non-custodial Solana desktop wallet built with Tauri and Rust.

## Features (MVP)

- Create and import Solana wallets from seed phrase
- Local encrypted wallet storage (Argon2id + AES-256-GCM)
- SOL and basic SPL token balances
- Send SOL and SPL tokens with transaction confirmation
- Receive via QR code and address copy
- On-chain activity history via Solana RPC
- Password-gated unlock, signing, and seed reveal

## Stack

- **Desktop shell:** Tauri v2
- **Backend:** Rust workspace (`crypto`, `storage`, `solana`, `wallet-core`, `models`)
- **Frontend:** React + TypeScript + Vite + Tailwind + shadcn-style components
- **Package manager:** pnpm

## Development

### Prerequisites

- Rust (stable)
- Node.js 20+
- pnpm
- Linux system dependencies for Tauri ([docs](https://v2.tauri.app/start/prerequisites/))

### Setup

```bash
cd apps/desktop
pnpm install
```

Optional RPC configuration:

```bash
cp ../../.env.example ../../.env
# edit AEGIS_RPC_URL if needed
```

### Run

From `apps/desktop`:

```bash
pnpm tauri dev
```

Or build the workspace crates:

```bash
cd ../..
export TMPDIR=$PWD/.tmp
cargo test
```

### Build

```bash
cd apps/desktop
pnpm tauri build
```

## Security model

- Private keys and mnemonics are encrypted at rest
- Signing happens only in the Rust core
- Frontend never receives raw private keys
- Seed phrase reveal requires password verification
- Send operations require password + confirmation screen

## Project structure

```
aegis/
├── crates/
│   ├── models/       # shared types
│   ├── crypto/       # Argon2id + AES-GCM
│   ├── storage/      # encrypted wallet file I/O
│   ├── solana/       # RPC + transaction building
│   └── wallet-core/  # wallet orchestration
└── apps/desktop/     # Tauri + React UI
```

## License

MIT
