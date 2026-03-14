# Tasklist: ML-4 — Generate Own Program IDs

**Ticket:** ML-4
**Phase:** 4 (Iteration 4)
**Status:** REVIEW_OK

---

## Context

The codebase ships with hardcoded program IDs (`sol_usd_oracle = 4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU`, `token_minter = E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE`) belonging to the original author. Existing keypair files in `program/target/deploy/` already contain different keys (`GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3` and `3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn`), causing a mismatch. All 11 source files referencing the old IDs must be updated to the keypair-derived pubkeys so that builds, tests, scripts, and frontend all reference the correct programs consistently.

---

## Tasks

- [x] **Task 1: Synchronize program IDs via `anchor keys sync`**
  - **Directory:** `program/`
  - Run `anchor keys sync` to read existing keypair files from `program/target/deploy/` and automatically update the `declare_id!` macros and `Anchor.toml` entries.
  - Extract the new pubkeys for subsequent manual updates: `solana-keygen pubkey target/deploy/sol_usd_oracle-keypair.json` and `solana-keygen pubkey target/deploy/token_minter-keypair.json`.
  - **Fallback:** If `anchor keys sync` is unavailable, manually extract pubkeys and update the three files by hand.
  - **Acceptance Criteria:**
    - `declare_id!` in `program/programs/sol_usd_oracle/src/lib.rs` matches the pubkey from `sol_usd_oracle-keypair.json`
    - `declare_id!` in `program/programs/token_minter/src/lib.rs` matches the pubkey from `token_minter-keypair.json`
    - Both `[programs.localnet]` and `[programs.devnet]` sections in `program/Anchor.toml` contain the new IDs for both programs

- [x] **Task 2: Update frontend config**
  - **File:** `frontend/app/config.ts`
  - Replace `ORACLE_PROGRAM_ID` value (`"4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU"`) with the new oracle pubkey.
  - Replace `MINTER_PROGRAM_ID` value (`"E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE"`) with the new minter pubkey.
  - **Acceptance Criteria:**
    - `ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID` in `frontend/app/config.ts` match the keypair-derived pubkeys
    - The new IDs differ from the original hardcoded IDs

- [x] **Task 3: Update init and utility scripts**
  - Replace old program ID strings with new pubkeys in:
    - `program/scripts/init-local.js` (lines 22-23): Both `ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID`
    - `program/scripts/get-oracle-pda.js` (line 6): `ORACLE_PROGRAM_ID`
  - **Acceptance Criteria:**
    - `init-local.js` contains the new oracle and minter pubkeys in its `PublicKey(...)` constructors
    - `get-oracle-pda.js` contains the new oracle pubkey in its `PublicKey(...)` constructor
    - `make init` will not fail due to stale program IDs (PDA derivation uses correct program ID)

- [x] **Task 4: Update LiteSVM test files**
  - Replace old program ID strings with new pubkeys in:
    - `program/tests/oracle.litesvm.ts` (line 16): `ORACLE_PROGRAM_ID`
    - `program/tests/minter.litesvm.ts` (lines 24-25): Both `ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID`
  - **Acceptance Criteria:**
    - `oracle.litesvm.ts` loads the `.so` binary against the correct oracle program ID
    - `minter.litesvm.ts` loads both `.so` binaries against the correct program IDs
    - Program IDs in test files match the keypair-derived pubkeys

- [x] **Task 5: Update migration script**
  - **File:** `program/migrations/deploy.ts` (lines 10-11)
  - Replace both `ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID` `new PublicKey(...)` values with the new pubkeys.
  - **Acceptance Criteria:**
    - Both program IDs in `migrations/deploy.ts` match the keypair-derived pubkeys
    - `anchor migrate` will not fail due to stale program IDs

- [x] **Task 6: Update documentation and templates**
  - Replace old program IDs with new pubkeys in:
    - `backend/.env.example` (lines 10-11): Both `ORACLE_PROGRAM_ID=` and `MINTER_PROGRAM_ID=` values
    - `CLAUDE.md` (line 106): Both `sol_usd_oracle = ...` and `token_minter = ...` entries in the Key Constants section
  - **Acceptance Criteria:**
    - `backend/.env.example` contains the new program IDs so that copying it to `.env` yields correct values
    - `CLAUDE.md` Key Constants section reflects the new program IDs

- [x] **Task 7: Verify build succeeds**
  - Run `anchor build` from `program/` directory.
  - **Acceptance Criteria:**
    - `anchor build` completes with exit code 0
    - Both `.so` binaries are regenerated in `target/deploy/`
    - No compilation errors related to `declare_id!` or CPI references

- [x] **Task 8: Verify no stale references remain**
  - Run a codebase-wide search for both old program ID strings (`4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU` and `E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE`) across all source code, config, and active documentation files.
  - **Acceptance Criteria:**
    - Old IDs appear only in historical documentation files (`docs/idea.md`, `docs/phase/phase-4.md`, `docs/prd/ML-4.prd.md`, `docs/research/ML-4.md`, `docs/plan/ML-4.md`) where they serve as reference
    - Zero matches in any source code (`.rs`, `.ts`, `.js`), configuration (`.toml`, `.env*`), or active documentation (`CLAUDE.md`) files

- [x] **Task 9: Verify LiteSVM tests pass**
  - Run `node --import tsx/esm node_modules/.bin/mocha -t 1000000 "tests/**/*.ts"` from `program/` directory.
  - **Acceptance Criteria:**
    - Both `oracle.litesvm.ts` and `minter.litesvm.ts` test suites pass
    - Tests load `.so` files against the correct (new) program IDs
