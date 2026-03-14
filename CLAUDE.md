# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Educational Solana mini-launchpad: two Anchor on-chain programs (SOL/USD oracle + token minter), a Rust backend for price updates and event listening, and a Remix frontend. The project contains intentional `TODO` stubs and broken tests for students to fix.

## Build & Test Commands

### Programs (from `program/`)
```bash
anchor build                    # Build both programs
anchor deploy --provider.cluster localnet  # Deploy to local validator
anchor test --skip-build        # Run LiteSVM tests (no network needed)
yarn run ts-mocha -p ./tsconfig.json -t 1000000 "tests/**/*.ts"  # Run tests directly
node scripts/init-local.js      # Initialize oracle + minter on localnet
```

### Backend (from `backend/`)
```bash
cargo test                      # Run unit tests
cargo run                       # Start price updater + event listener
```

### Frontend (from `frontend/`)
```bash
npm run dev                     # Dev server on http://localhost:7001
npm test                        # Vitest unit tests
```

### Top-level Makefile shortcuts
```bash
make install             # yarn install (program/) + npm install (frontend/)
make build               # anchor build
make deploy              # build + deploy to localnet
make deploy-devnet       # build + deploy to devnet
make deploy-oracle       # deploy oracle only (localnet)
make deploy-minter       # deploy minter only (localnet)
make init                # initialize oracle + minter on localnet
make init-devnet         # initialize on devnet
make test                # run program tests (LiteSVM)
make backend             # cargo run in backend/
make backend-devnet      # backend with devnet RPC
make frontend            # npm run dev in frontend/
make kill-frontend       # kill process on port 7001
make validator           # solana-test-validator
make validator-metaplex  # validator with Metaplex clone from mainnet
```

## Architecture

### Two Anchor Programs (`program/programs/`)

**sol_usd_oracle** — Singleton PDA (`seed: "oracle_state"`) storing SOL/USD price (u64, 6 decimals). Only the admin can call `update_price`. Independent program with no dependencies.

**token_minter** — Mints SPL tokens with optional Metaplex metadata. Reads oracle price via CPI to compute USD→lamports fee. Transfers fee to treasury, mints tokens to user's ATA, emits `TokenCreated` event. Singleton config PDA (`seed: "minter_config"`).

**Cross-program relationship**: token_minter depends on sol_usd_oracle (reads price via CPI account validation, not instruction CPI). The oracle program is independent.

**Fee formula**: `fee_lamports = mint_fee_usd * LAMPORTS_PER_SOL / price` (both values use 6 decimals).

### Rust Backend (`backend/src/main.rs`)

Single-file service running two concurrent tokio tasks:
1. **Price updater** — Polls price (Binance API or mock), submits `update_price` tx to oracle on interval
2. **Event listener** — WebSocket subscription to minter logs, parses `TokenCreated` events via regex, outputs JSON

Configuration via `backend/.env` (see `.env.example`). Supports `~` in `BACKEND_KEYPAIR_PATH`.

### Remix Frontend (`frontend/`)

Terminal-styled React app with Solana Wallet Adapter (Phantom, Solflare). Polls oracle + minter PDA state every 5s. Constructs `mint_token` instruction client-side with manual Borsh encoding (`mintInstruction.ts`). Supports localnet/devnet network switching.

Key files: `config.ts` (program IDs, seeds), `mintInstruction.ts` (instruction builder), `TerminalMint.tsx` (main UI + minting logic), `TerminalApp.tsx` (wallet providers).

## Student TODO Locations

These are intentional stubs/bugs for the educational task:

1. **`program/programs/sol_usd_oracle/src/lib.rs`** — `apply_price_update()`: implement setting `oracle.price` and `oracle.last_updated_slot`
2. **`program/programs/token_minter/src/lib.rs`** — `compute_fee_lamports()`: implement USD→lamports conversion
3. **`program/tests/oracle.litesvm.ts`** — Assertion expects `decimals == 8` but code sets 6
4. **`program/tests/minter.litesvm.ts`** — Fee formula in assertion is inverted
5. **`backend/src/main.rs`** — `to_fixed_6()`: parse decimal string to fixed-6 u64
6. **`backend/src/main.rs`** — One unit test has intentionally wrong assertion

## Key Constants

- Program IDs: `sol_usd_oracle = 4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU`, `token_minter = E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE`
- Price decimals: 6 (both oracle price and minter fee use 10^6 scaling)
- Anchor version: 0.32.1
- Rust toolchain: 1.89.0 (defined in `program/rust-toolchain.toml`, components: rustfmt, clippy)

## Local Development Flow

1. `solana-test-validator` (or `make validator-metaplex` for token metadata display)
2. `make deploy` — builds and deploys both programs
3. `make init` — initializes oracle (price=$120) and minter (fee=$5, treasury=wallet)
4. Copy `ORACLE_STATE_PUBKEY` from init output to `backend/.env`
5. `make backend` — starts price updater + event listener
6. `make frontend` — starts web UI on port 7001

## Workflow

Development follows `docs/workflow.md`: strict iteration-by-iteration execution per `docs/tasklist.md`. Each iteration goes through **Propose → Approve → Implement → Verify → Commit**. No skipping, no scope creep. Architecture details in `docs/vision.md`, task specs in `docs/idea.md`.

## Conventions

- README and comments are in Russian
- Backend is a single `main.rs` file (no module splitting)
- Frontend uses manual Borsh encoding (not Anchor client) for instruction construction
- Tests use LiteSVM (lightweight SVM, no network required) with TypeScript Mocha
- Oracle state raw bytes: price at offset 40 (u64 LE), minter config: fee at offset 72, treasury at offset 40-72
