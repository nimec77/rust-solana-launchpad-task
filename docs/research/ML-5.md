# Research: ML-5 — Local End-to-End Cycle

**Ticket:** ML-5
**Phase:** 5 (Iteration 5)
**Date:** 2026-03-15

---

## Summary

ML-5 is an integration/validation iteration with no code changes required. The goal is to run the full local stack (validator, both on-chain programs, backend, frontend) and execute a complete token mint via the browser UI, verifying that the backend logs a `TokenCreated` event in JSON format. All preceding iterations (1-4) must be complete before this iteration can succeed.

---

## Current State

### Iteration 1-4 Status

All four prerequisite iterations are marked complete in `docs/tasklist.md`:
- **ML-1:** Oracle `apply_price_update()` implemented, test assertion fixed (decimals 8 -> 6)
- **ML-2:** Minter `compute_fee_lamports()` implemented, test fee formula fixed
- **ML-3:** Backend `to_fixed_6()` implemented, test assertion fixed (truncation, not rounding)
- **ML-4:** Program IDs updated to locally generated keys across codebase

### Program ID Consistency

New program IDs (from `program/target/deploy/` keypairs):
- `sol_usd_oracle`: `GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3`
- `token_minter`: `3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn`

**Verified consistent in all primary source locations:**

| Location | Oracle ID | Minter ID | Status |
|----------|-----------|-----------|--------|
| `program/programs/sol_usd_oracle/src/lib.rs` (line 8) | Correct | N/A | OK |
| `program/programs/token_minter/src/lib.rs` (line 16) | N/A | Correct | OK |
| `program/Anchor.toml` (localnet + devnet) | Correct | Correct | OK |
| `frontend/app/config.ts` (lines 2-3) | Correct | Correct | OK |
| `program/scripts/init-local.js` (lines 22-23) | Correct | Correct | OK |
| `program/tests/oracle.litesvm.ts` | Correct | N/A | OK |
| `program/tests/minter.litesvm.ts` | Correct | Correct | OK |
| `program/migrations/deploy.ts` | Correct | Correct | OK |
| `program/scripts/get-oracle-pda.js` | Correct | N/A | OK |
| `backend/.env` (gitignored, lines 10-11) | Correct | Correct | OK |
| `CLAUDE.md` (line 106) | Correct | Correct | OK |

### Backend `.env` State

The `backend/.env` file exists (gitignored) with correct program IDs but has `ORACLE_STATE_PUBKEY=<PDA from "oracle_state" seed>` as a placeholder on line 16. This must be set to the actual PDA value output by `make init` before the backend can start.

---

## DEVIATIONS

### DEVIATION: `backend/.env.example` still contains old program IDs

`backend/.env.example` lines 10-11 still reference the old author-owned program IDs:
```
ORACLE_PROGRAM_ID=4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU
MINTER_PROGRAM_ID=E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE
```

The PRD explicitly warns about this in the Risks table: "The `.env.example` still contains old program IDs (`4cuvLFF...`, `E5erG...`). After copying, the student must update `ORACLE_PROGRAM_ID`, `MINTER_PROGRAM_ID`, and `ORACLE_STATE_PUBKEY`."

**Impact on ML-5:** If the student naively copies `.env.example` to `.env` (Task 5.3), the backend will use wrong program IDs and fail. The existing `backend/.env` already has the correct IDs (updated during ML-4), so the student should either:
1. Keep the existing `backend/.env` and only update `ORACLE_STATE_PUBKEY`, or
2. Copy `.env.example` and update all three values.

This is a UX hazard but not a blocker since the PRD documents the risk. The `.env.example` should ideally be updated to match the new IDs to reduce confusion, but this is outside the strict scope of ML-5 (which specifies no code changes).

---

## Component Analysis

### Task 5.1: Validator with Metaplex

**Command:** `make validator-metaplex`

Runs: `solana-test-validator --clone-upgradeable-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s --url https://api.mainnet-beta.solana.com`

**Requirements:**
- Internet access to clone the Metaplex Token Metadata program from mainnet
- No existing `test-ledger/` directory conflicts (or clean previous validator state)
- Port 8899 (HTTP) and 8900 (WebSocket) must be available

**Why Metaplex is required:** The `token_minter` program's `mint_token` instruction invokes `CreateMetadataAccountV3CpiBuilder` from the Metaplex Token Metadata program (`metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s`). Without this program deployed on the local validator, minting with non-empty `name` will fail. The frontend has a "Metaplex" checkbox that controls whether metadata is attached. If unchecked, the empty `name` branch skips the CPI, but metadata won't be displayed in wallets.

### Task 5.2: Deploy and Initialize

**Commands:** `make deploy && make init`

**Deploy flow:**
1. `anchor build` compiles both programs (produces `.so` binaries in `program/target/deploy/`)
2. `anchor deploy --provider.cluster localnet` deploys both to the running validator

**Init flow** (`program/scripts/init-local.js`):
1. Derives oracle PDA from seed `"oracle_state"` + oracle program ID
2. Derives minter PDA from seed `"minter_config"` + minter program ID
3. Calls `initialize_oracle` with admin = payer wallet, then `update_price` with initial price `120_000_000` ($120.00)
4. Calls `initialize_minter` with treasury = payer wallet, fee = `5_000_000` ($5.00), oracle state PDA, oracle program ID
5. Outputs `ORACLE_STATE_PUBKEY=<pda>` twice (once before init, once after)

**Both calls are idempotent:** If already initialized, the script catches the "already in use" error and continues.

### Task 5.3: Configure Backend `.env`

**Current `backend/.env` state:**
- `SOLANA_RPC_HTTP=http://127.0.0.1:8899` -- correct for localnet
- `SOLANA_RPC_WS=ws://127.0.0.1:8900` -- correct for localnet
- `ORACLE_PROGRAM_ID=GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3` -- correct (updated in ML-4)
- `MINTER_PROGRAM_ID=3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn` -- correct (updated in ML-4)
- `ORACLE_STATE_PUBKEY=<PDA from "oracle_state" seed>` -- **MUST BE UPDATED** from `make init` output
- `BACKEND_KEYPAIR_PATH=~/.config/solana/id.json` -- correct (supports `~` expansion in code)
- `PRICE_POLL_INTERVAL_SEC=600` -- 10 minute interval

**Only action needed:** Replace the `ORACLE_STATE_PUBKEY` placeholder with the PDA value from `make init` output.

### Task 5.4: Run Backend + Frontend

**Backend** (`make backend` = `cd backend && cargo run`):
- Loads `backend/.env` via `dotenvy`
- Spawns two `tokio` tasks:
  1. **Price updater:** Runs immediately on startup, then every `PRICE_POLL_INTERVAL_SEC`. Fetches price (Binance or mock), submits `update_price` tx to oracle.
  2. **Event listener:** WebSocket subscription to minter program logs, parses `TokenCreated` events via regex, outputs JSON to stdout.
- If `MOCK_PRICE` is not set, backend uses Binance live price API. For local testing, `MOCK_PRICE=120000000` can be uncommented to avoid external network dependency.

**Frontend** (`make frontend` = `cd frontend && npm run dev`):
- Starts Remix dev server on port 7001
- `make frontend` first kills any existing process on port 7001 via `make kill-frontend`
- Wallet adapter configured for Phantom and Solflare
- Default network is `localnet` (`http://127.0.0.1:8899`)
- Polls oracle PDA (price at byte offset 40) and minter PDA (treasury at bytes 40-72, fee at byte offset 72) every 5 seconds

### Task 5.5: Mint a Token via UI

**Minting flow from the frontend perspective:**

1. User connects wallet (Phantom/Solflare) configured for localnet RPC
2. Frontend displays oracle price and mint fee (polled from chain every 5s)
3. User fills form: token name (up to 32 chars), symbol (up to 10 chars), image URL, optional metadata URI, decimals (0-9), initial supply
4. Optional: enable Metaplex checkbox to write on-chain metadata
5. Frontend builds `mint_token` instruction via `buildMintTokenInstruction()`:
   - Generates fresh `Keypair` for the new mint account
   - Derives PDAs: minter config, oracle state, metadata PDA
   - Computes user's ATA for the new mint
   - Manually encodes Borsh data: discriminator (8 bytes) + decimals (1 byte) + supply (8 bytes LE) + name/symbol/uri as Borsh strings
   - Account order: config PDA, user, treasury, oracle program, oracle PDA, mint, user ATA, metadata program, metadata PDA, token program, associated token program, system program, rent sysvar
6. Transaction is partially signed (mint keypair), then sent to wallet for user signature
7. On success, frontend shows transaction signature with explorer link

**On-chain minting flow (`token_minter::mint_token`):**

1. Validates oracle price > 0, oracle decimals == 6
2. Computes `fee_lamports = mint_fee_usd * LAMPORTS_PER_SOL / price` (u128 intermediate)
3. Transfers fee in SOL from user to treasury via `system_program::transfer`
4. Mints `initial_supply` tokens to user's ATA via `token::mint_to`
5. If `name` is non-empty: creates Metaplex metadata account via `CreateMetadataAccountV3CpiBuilder`
6. Emits `TokenCreated` event with: creator, mint, decimals, initial_supply, fee_lamports, sol_usd_price, slot

**Backend event capture:**

The event listener's regex pattern matches:
```
TokenCreated \{ creator: ([A-Za-z0-9]+), mint: ([A-Za-z0-9]+), decimals: (\d+), initial_supply: (\d+), fee_lamports: (\d+), sol_usd_price: (\d+), slot: (\d+) \}
```

Parsed fields are serialized to JSON and printed to stdout, fulfilling the acceptance criterion.

---

## Existing Patterns

### Environment Configuration

The backend uses `dotenvy` to load `backend/.env` at startup. The `Config::from_env()` function reads all required environment variables with descriptive error messages via `anyhow::Context`. The `~` expansion for `BACKEND_KEYPAIR_PATH` is handled manually (line 54-58 of `main.rs`).

### PDA Derivation

PDAs are derived consistently across all components:
- **Oracle PDA:** `findProgramAddressSync([Buffer.from("oracle_state")], ORACLE_PROGRAM_ID)`
- **Minter PDA:** `findProgramAddressSync([Buffer.from("minter_config")], MINTER_PROGRAM_ID)`
- **Metadata PDA:** `findProgramAddressSync([Buffer.from("metadata"), MPL_METADATA_PK, mint.pubkey], MPL_METADATA_PK)`

### Raw Byte Offsets for PDA State

The frontend reads PDA account data at specific byte offsets (after 8-byte Anchor discriminator):
- **Oracle:** price at offset 40 (= 8 discriminator + 32 admin pubkey)
- **Minter:** treasury at offset 40-72 (= 8 discriminator + 32 admin pubkey), fee at offset 72 (= 40 + 32 treasury pubkey)

These offsets are consistent with the `OracleState` and `MinterConfig` struct layouts in Rust.

### Instruction Encoding

The frontend uses manual Borsh encoding (not Anchor client) to construct the `mint_token` instruction. The discriminator `[172, 137, 183, 14, 207, 110, 234, 56]` is the first 8 bytes of SHA-256 of `"global:mint_token"`.

---

## Dependencies and Ordering

The tasks must be executed in strict sequence:

```
5.1 Start validator (with Metaplex)
  |
  v
5.2 Deploy & init (builds, deploys, initializes oracle+minter, outputs ORACLE_STATE_PUBKEY)
  |
  v
5.3 Configure backend/.env (set ORACLE_STATE_PUBKEY from init output)
  |
  v
5.4 Run backend + frontend (in separate terminals, concurrently)
  |
  v
5.5 Mint a token via UI (requires all above running)
```

Each step depends on the previous one. The validator must be running before deploy. Deploy must succeed before init. Init must output the PDA before `.env` can be configured. Backend and frontend must be running before minting.

---

## Risks and Limitations

| Risk | Assessment | Mitigation |
|------|-----------|------------|
| **`.env.example` has stale program IDs** | HIGH -- If the student copies `.env.example` to `.env`, they get wrong `ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID`. The existing `.env` has correct IDs from ML-4, but `ORACLE_STATE_PUBKEY` is still a placeholder. | Either (a) keep existing `.env` and only update `ORACLE_STATE_PUBKEY`, or (b) update `.env.example` first to match new IDs. |
| **Metaplex clone requires internet** | MEDIUM -- `make validator-metaplex` clones the program from mainnet, requiring internet access and a responsive mainnet RPC. | If offline, use `make validator` instead and disable the Metaplex checkbox in the UI. Minting will still work but without on-chain metadata. |
| **Browser wallet localhost routing** | MEDIUM -- Phantom and other wallets may route RPC through remote servers, causing `localhost:8899` to resolve incorrectly. | Per `docs/vision.md`, Backpack wallet is recommended for localnet. Alternatively, use a browser extension configured with `http://127.0.0.1:8899` as custom RPC. |
| **Backend price updater hits Binance API** | LOW -- If `MOCK_PRICE` is not set, the backend will try to fetch live SOL/USD from Binance. This requires internet access and is unnecessary for local testing. | Uncomment `MOCK_PRICE=120000000` in `.env` for local testing, or leave it unset to use live price. The init script already sets an initial price of $120. |
| **Port conflicts** | LOW -- Port 7001 (frontend) or 8899/8900 (validator) may be in use. | `make frontend` includes `make kill-frontend` to free port 7001. For validator ports, stop any running `solana-test-validator` processes. |
| **Admin keypair mismatch** | MEDIUM -- The `BACKEND_KEYPAIR_PATH` must point to the same keypair that was used during `make init` (which becomes the oracle admin). | Default is `~/.config/solana/id.json`, which is also the default Anchor wallet in `Anchor.toml`. As long as both use the default keypair, this is not an issue. |
| **ORACLE_STATE_PUBKEY not updated** | HIGH -- The backend will crash on startup if `ORACLE_STATE_PUBKEY` is still the placeholder string `<PDA from "oracle_state" seed>`. | Must copy the actual PDA value from `make init` output before running `make backend`. |

---

## Resolved Questions

The PRD listed no open questions. The user confirmed to proceed with default requirements.

---

## New Technical Questions Discovered

1. **Should `backend/.env.example` be updated with the new program IDs as part of ML-5?** The `.env.example` still has old IDs from the original author (`4cuvLFF...`, `E5erG...`). While the PRD documents this as a known risk, updating it would reduce friction for Task 5.3 and any future setup. This is a minor configuration change, not a code change. However, ML-5 is defined as a no-code-changes iteration.

2. **Should `MOCK_PRICE` be uncommented in `.env` for reliable local testing?** Without `MOCK_PRICE`, the backend will try to fetch from Binance on each poll interval. If the student lacks internet or Binance is unreachable, the price updater will log errors (but the event listener will still work since the init script already set an initial price). For a fully offline-capable local test, `MOCK_PRICE=120000000` should be set.

3. **Which wallet should be recommended for localnet testing?** `docs/vision.md` recommends Backpack for localnet because Phantom/Solflare may route RPC calls through remote servers. However, the PRD mentions Phantom and Solflare. The student should be aware that wallet choice matters for localnet.

---

## Key File Paths

| Component | File | Purpose |
|-----------|------|---------|
| Makefile | `/Users/comrade77/RustroverProjects/rust-solana-launchpad-task/Makefile` | All make targets |
| Backend .env | `/Users/comrade77/RustroverProjects/rust-solana-launchpad-task/backend/.env` | Backend runtime config (gitignored) |
| Backend .env.example | `/Users/comrade77/RustroverProjects/rust-solana-launchpad-task/backend/.env.example` | Config template (stale IDs) |
| Backend main.rs | `/Users/comrade77/RustroverProjects/rust-solana-launchpad-task/backend/src/main.rs` | Price updater + event listener |
| Init script | `/Users/comrade77/RustroverProjects/rust-solana-launchpad-task/program/scripts/init-local.js` | Oracle + minter initialization |
| Frontend config | `/Users/comrade77/RustroverProjects/rust-solana-launchpad-task/frontend/app/config.ts` | Program IDs, seeds, networks |
| Mint instruction | `/Users/comrade77/RustroverProjects/rust-solana-launchpad-task/frontend/app/mintInstruction.ts` | Manual Borsh instruction builder |
| Terminal UI | `/Users/comrade77/RustroverProjects/rust-solana-launchpad-task/frontend/app/components/TerminalMint.tsx` | Main minting UI component |
| Wallet providers | `/Users/comrade77/RustroverProjects/rust-solana-launchpad-task/frontend/app/components/TerminalApp.tsx` | Wallet adapter setup |
| Oracle program | `/Users/comrade77/RustroverProjects/rust-solana-launchpad-task/program/programs/sol_usd_oracle/src/lib.rs` | Oracle on-chain program |
| Minter program | `/Users/comrade77/RustroverProjects/rust-solana-launchpad-task/program/programs/token_minter/src/lib.rs` | Minter on-chain program |
| Oracle state | `/Users/comrade77/RustroverProjects/rust-solana-launchpad-task/program/programs/sol_usd_oracle/src/state.rs` | OracleState struct definition |
| Phase spec | `/Users/comrade77/RustroverProjects/rust-solana-launchpad-task/docs/phase/phase-5.md` | Phase 5 task list |
| Task list | `/Users/comrade77/RustroverProjects/rust-solana-launchpad-task/docs/tasklist.md` | Overall iteration progress |
| Anchor config | `/Users/comrade77/RustroverProjects/rust-solana-launchpad-task/program/Anchor.toml` | Program IDs, cluster, wallet |
