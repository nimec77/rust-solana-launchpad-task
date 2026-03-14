# Mini-Launchpad: Capstone Project Summary

## Project Overview

Build a **mini-launchpad** on Solana: a system where users pay a USD-denominated fee (converted to SOL at the current market rate) to mint SPL tokens. The project consists of three components:

- **SOL/USD Oracle** — an on-chain price feed program
- **Token Minter** — an on-chain program that reads the oracle price, collects a fee, and mints tokens
- **Backend Service** — an off-chain Rust service that updates the oracle price and listens for minting events

A **Remix frontend** is provided pre-built for interacting with the system.

### Repository Structure

```
program/             — Anchor workspace (two on-chain programs + tests)
  programs/sol_usd_oracle  — stores SOL/USD price (decimals = 6)
  programs/token_minter    — mints SPL tokens for a SOL fee using oracle price
  tests/                   — Anchor TypeScript tests (LiteSVM)
backend/             — Rust service: price updater + event listener
frontend/            — Remix frontend (React Router, terminal-styled UI)
```

## Prerequisites

Students are expected to be familiar with:

- **Oracles** — how on-chain price feeds work (PDA state, admin-only updates, staleness checks)
- **Token Factory** — SPL token minting, Associated Token Accounts, Metaplex metadata
- **Security** — PDA validation, signer checks, safe integer math (u128 intermediates to avoid overflow)
- **Backend Feeder** — off-chain services that submit transactions (price updates, retry logic, error classification)
- **Observability** — event emission, WebSocket log subscription, Anchor event parsing

## Architecture

```
┌──────────────────┐       CPI read        ┌──────────────────┐
│  sol_usd_oracle   │◄─────────────────────│   token_minter    │
│  (price feed PDA) │                       │  (mint + fee PDA) │
└────────▲─────────┘                       └────────▲─────────┘
         │ update_price tx                          │ mint_token tx
         │                                          │
┌────────┴─────────┐                       ┌────────┴─────────┐
│   Rust Backend    │                       │  Remix Frontend   │
│ (price updater +  │                       │ (wallet + manual  │
│  event listener)  │                       │  Borsh encoding)  │
└──────────────────┘                       └──────────────────┘
```

### sol_usd_oracle

Singleton PDA (`seed: "oracle_state"`) storing SOL/USD price as `u64` with 6 decimal places. Only the designated admin can call `update_price`. Emits events on price changes. Independent program with no external dependencies.

### token_minter

Mints SPL tokens with optional Metaplex metadata. Reads the oracle price (via CPI account validation) to compute the USD→lamports fee. Transfers the fee to a treasury wallet, mints tokens to the user's ATA, and emits a `TokenCreated` event. Config stored in singleton PDA (`seed: "minter_config"`).

**Fee formula**: `fee_lamports = mint_fee_usd * LAMPORTS_PER_SOL / oracle_price` (both values use 10^6 scaling).

### Rust Backend

Single-file async Rust service (`backend/src/main.rs`) running two concurrent tokio tasks:
1. **Price updater** — polls SOL/USD price (Binance API or mock), submits `update_price` transactions to the oracle on a configurable interval
2. **Event listener** — WebSocket subscription to minter program logs, parses `TokenCreated` events, outputs structured JSON

### Key Constants

- Program IDs: `sol_usd_oracle = 4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU`, `token_minter = E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE`
- Price decimals: 6 (both oracle price and minter fee use 10^6 scaling)
- All fee math is integer-only: `fee_lamports = mint_fee_usd * LAMPORTS_PER_SOL / price`
- Access to `update_price` restricted to oracle admin (backend keypair)
- `mint_token` fails if `price == 0` or fee/supply are invalid
- Anchor version: 0.32.1

---

## Step-by-Step Task Breakdown

### Two Phases of Work

The assignment is structured in two phases:

1. **Fix `program/` and `backend/`** — complete TODO stubs and fix broken tests so all tests pass
2. **Deploy under your own program IDs** — generate new keypairs, update IDs everywhere, deploy to devnet, and verify the full cycle

### Step 0: Prepare the Repository

- **Create a public repository on GitHub** and upload the project's source code (the first commit)
- **Commit discipline**: make all subsequent changes as separate commits — this helps the reviewer track your progress
- Review the project structure: `program/`, `backend/`, `frontend/`
- Read through `CLAUDE.md` and the existing code to understand the architecture
- Run `make install` to install all dependencies (runs `yarn install` in `program/` and `npm install` in `frontend/`)

### Step 1: Fix the On-Chain Programs (`program/`)

> *From `program/README_TASK.md`: This is the educational version of `program/` for students. TODO stubs are left in oracle price update logic and minter fee calculation. Tests contain intentionally broken assertions marked `TODO(student)`. The project builds but some tests fail — students must fix both the Rust code and the tests.*

**Goal**: Complete the TODO stubs in both Anchor programs and fix the broken tests.

| File | What to fix | Details |
|------|-------------|---------|
| `program/programs/sol_usd_oracle/src/lib.rs` | Implement `apply_price_update()` | Set `oracle.price = new_price` and `oracle.last_updated_slot = current_slot`. The function currently has `let _ = (oracle, new_price, current_slot);` as a placeholder. |
| `program/programs/token_minter/src/lib.rs` | Implement `compute_fee_lamports()` | Convert USD fee to lamports: `fee_lamports = mint_fee_usd * LAMPORTS_PER_SOL / price`. Use u128 intermediate to prevent overflow. Both `mint_fee_usd` and `price` use 6 decimal places. |
| `program/tests/oracle.litesvm.ts` | Fix decimals assertion | Test expects `decoded.decimals == 8` but the oracle stores 6 decimals. Change to `expect(decoded.decimals).to.eq(6)`. |
| `program/tests/minter.litesvm.ts` | Fix fee formula assertion | Fee formula is inverted: test computes `PRICE * LAMPORTS_PER_SOL / FEE_USD` but should be `FEE_USD * LAMPORTS_PER_SOL / PRICE`. The fee gets smaller when SOL/USD price gets larger. |

**Expected initial state**: project builds, some tests fail.
**Expected final state**: all tests pass.

**Verify**:
```bash
cd program
anchor build
yarn run ts-mocha -p ./tsconfig.json -t 1000000 "tests/**/*.ts"
# Or: anchor test --skip-build
```

### Step 2: Fix the Backend (`backend/`)

> *From `backend/README_TASK.md`: This is the educational version of `backend/` for students. A TODO is left in the `to_fixed_6` function. One unit test is intentionally broken and marked `TODO(student)`. The code compiles but some tests fail — after implementing the helper function and fixing the test, everything should pass.*

**Goal**: Complete the backend helper function and fix the broken unit test.

| File | What to fix | Details |
|------|-------------|---------|
| `backend/src/main.rs` — `to_fixed_6()` | Implement decimal string parser | Parse a decimal string into a `u64` with 6 fixed decimals. Examples: `"120"` → `120_000_000`, `"120.12"` → `120_120_000`, `"120.123456789"` → `120_123_456` (truncate, don't round). |
| `backend/src/main.rs` — test `to_fixed_6_truncates_fraction_to_six_digits` | Fix wrong assertion | Test asserts `to_fixed_6("1.1234569") == 1_123_457` (rounded), but the parser should truncate after 6 digits, giving `1_123_456`. |

**Expected initial state**: code compiles, some tests fail.
**Expected final state**: all tests pass.

**Verify**:
```bash
cd backend
cargo test
```

### Step 3: Generate Own Keypairs and Update Program IDs

- Generate new keypairs for both programs using `solana-keygen grind` or `anchor keys sync`
- Update program IDs in all locations:
  - `program/programs/sol_usd_oracle/src/lib.rs` (`declare_id!`)
  - `program/programs/token_minter/src/lib.rs` (`declare_id!`)
  - `program/Anchor.toml`
  - `frontend/app/config.ts`
- Rebuild with `anchor build` to verify

### Step 4: Configure and Run the Backend

- Copy `backend/.env.example` to `backend/.env`
- Fill in configuration values:

| Variable | Description | Default |
|----------|-------------|---------|
| `SOLANA_RPC_HTTP` | HTTP RPC endpoint | `http://127.0.0.1:8899` |
| `SOLANA_RPC_WS` | WebSocket RPC endpoint | `ws://127.0.0.1:8900` |
| `ORACLE_PROGRAM_ID` | Oracle program ID (from `anchor keys list`) | `4cuvLFFq...` |
| `MINTER_PROGRAM_ID` | Minter program ID (from `anchor keys list`) | `E5erGzax...` |
| `ORACLE_STATE_PUBKEY` | PDA from `"oracle_state"` seed (output of `make init`) | — must be set |
| `BACKEND_KEYPAIR_PATH` | Admin keypair path (supports `~`) | `~/.config/solana/id.json` |
| `PRICE_POLL_INTERVAL_SEC` | How often to update price (seconds) | `600` |
| `MOCK_PRICE` | Fixed price for testing (e.g., `120000000` = $120) | unset (uses Binance API) |
| `PRICE_API_URL` | Custom price API URL | Binance SOL/USDT |

- Run `make backend` to start the price updater and event listener

### Step 5: Full Local Cycle Verification

1. Start `solana-test-validator` (or `make validator-metaplex` for Metaplex token metadata display)
2. `make deploy` — build and deploy both programs to localnet
3. `make init` — initialize oracle (price=$120) and minter (fee=$5, treasury=wallet)
4. Copy `ORACLE_STATE_PUBKEY` from init output to `backend/.env`
5. `make backend` — start the backend service
6. `make frontend` — open the web UI at `http://localhost:7001`
7. Connect a wallet and mint a token
8. Verify the backend logs show the `TokenCreated` event in JSON

**Metaplex metadata**: when minting, you can pass `name`, `symbol`, and `uri` — the contract creates Metaplex Token Metadata (name, ticker, image in wallet). If you pass an empty name, no metadata is created (suitable for localnet without Metaplex). For wallet display, run `make validator-metaplex` to clone the Metaplex program from mainnet.

**Wallet for localnet**: For testing with local RPC (`http://localhost:8899`), Backpack is recommended — it connects to your local validator correctly. Most other wallets (including Phantom and Solflare) route RPC requests through remote servers, so `localhost` resolves to the server itself, not your machine. For details on localhost RPC limitations and tunneling (ngrok, etc.), see the Backpack documentation.

### Step 6: Deploy to Devnet and Submit Results

1. Switch Solana CLI to Devnet:
   ```bash
   solana config set --url devnet
   solana airdrop 2
   ```
2. Deploy both programs to Devnet:
   ```bash
   make deploy-devnet
   ```
3. Initialize oracle and minter on Devnet:
   ```bash
   make init-devnet
   ```
4. Update `backend/.env` with Devnet RPC URLs (`https://api.devnet.solana.com` / `wss://api.devnet.solana.com`)
5. Run the backend pointing to Devnet
6. In the frontend, switch to **Devnet** network; in the wallet, switch to Devnet
7. Mint at least one token via the frontend
8. **Update `README.md`** with your deployed contract addresses and 2–3 Devnet transaction links as proof
9. **Submit**: GitHub repository URL (with updated README) + Devnet transaction links

### How to Submit Your Project

1. Attach the **GitHub repository link** to your submission
2. The reviewer will create **issues** for any problems found
3. Fix each issue with a **separate commit** — include the issue number in the commit message (e.g., `fix: correct fee calculation #3`)
4. Each fix should be its own commit so the reviewer can track progress

---

## Completion Criteria

- All program tests pass (`anchor test --skip-build`)
- All backend tests pass (`cargo test`)
- Both programs deployed to Devnet with your own program IDs
- Full minting cycle works end-to-end (oracle update → fee payment → token mint → event captured)
- Devnet transaction links provided as proof

## Skills Tested

- **PDA derivation and validation** — oracle and minter config accounts
- **Cross-Program Invocation (CPI)** — minter reading oracle price via account validation
- **Safe math** — u128 intermediates for fee calculation to prevent overflow
- **Off-chain integration** — async Rust service interacting with on-chain programs
- **Anchor testing with LiteSVM** — lightweight test environment without a running validator
- **Devnet deployment** — real network deployment and configuration
- **Fixed-point arithmetic** — decimal string parsing to u64 with 6-decimal precision

---

## Makefile Quick Reference

```bash
make build               # anchor build
make deploy              # build + deploy to localnet
make deploy-devnet       # build + deploy to devnet
make init                # initialize oracle + minter on localnet
make init-devnet         # initialize oracle + minter on devnet
make test                # run program tests (LiteSVM)
make backend             # cargo run in backend/
make frontend            # npm run dev in frontend/
make validator           # start solana-test-validator
make validator-metaplex  # validator with Metaplex clone from mainnet
```
