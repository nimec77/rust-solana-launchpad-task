# Summary: ML-4 — Generate Own Program IDs

**Ticket:** ML-4
**Phase:** 4 (Iteration 4)
**Date:** 2026-03-14
**Status:** Complete

---

## Overview

ML-4 was the fourth iteration of the Solana Mini-Launchpad project. It resolved a mismatch between the hardcoded program IDs in the source code (belonging to the original author's deployment) and the locally generated keypair files in `program/target/deploy/`. All references to the old program IDs (`sol_usd_oracle = 4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU`, `token_minter = E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE`) were replaced with the student's own keypair-derived public keys (`sol_usd_oracle = GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3`, `token_minter = 3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn`) across 11 source files, ensuring builds, tests, scripts, and the frontend all reference the correct programs.

---

## Changes Made

### 1. Synchronized `declare_id!` macros and Anchor.toml via `anchor keys sync`

The `anchor keys sync` command (or equivalent manual updates) read the existing keypair files from `program/target/deploy/` and updated the three Anchor-managed locations:

- **`program/programs/sol_usd_oracle/src/lib.rs`** -- `declare_id!` updated to `GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3`
- **`program/programs/token_minter/src/lib.rs`** -- `declare_id!` updated to `3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn`
- **`program/Anchor.toml`** -- Both `[programs.localnet]` and `[programs.devnet]` sections updated with the new IDs for both programs

### 2. Updated frontend configuration

**File:** `frontend/app/config.ts`

`ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID` constants were replaced with the new public keys derived from the local keypair files.

### 3. Updated init and utility scripts

- **`program/scripts/init-local.js`** -- Both `ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID` `PublicKey` constructors updated
- **`program/scripts/get-oracle-pda.js`** -- `ORACLE_PROGRAM_ID` `PublicKey` constructor updated

### 4. Updated LiteSVM test files

- **`program/tests/oracle.litesvm.ts`** -- `ORACLE_PROGRAM_ID` updated so the test loads the `.so` binary against the correct program ID
- **`program/tests/minter.litesvm.ts`** -- Both `ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID` updated

### 5. Updated migration script

**File:** `program/migrations/deploy.ts` -- Both program ID `PublicKey` constructors updated.

### 6. Updated documentation and templates

- **`backend/.env.example`** -- Both `ORACLE_PROGRAM_ID=` and `MINTER_PROGRAM_ID=` values updated so that copying the template to `.env` produces correct defaults
- **`CLAUDE.md`** -- Key Constants section updated to reflect the new program IDs

### 7. Minor backend import reordering

**File:** `backend/src/main.rs` -- The `use` statements were reordered alphabetically (a cosmetic change with no behavioral impact).

---

## Decisions

- **Extended scope beyond the 4-location spec:** The phase-4 specification listed only 4 locations for update (`declare_id!` in both programs, `Anchor.toml`, `frontend/app/config.ts`). However, the old IDs appeared in 11 source files total. Leaving the additional 7 files stale would have caused runtime failures in tests, init scripts, migration, and backend configuration. All 11 locations were updated to deliver a fully working system.
- **Used existing keypairs:** The keypair files in `program/target/deploy/` already existed with public keys different from the hardcoded source IDs. No new keypair generation was needed -- the source was simply synchronized to match the existing keypairs.
- **No new keypair generation via `solana-keygen`:** Since usable keypair files were already present, regenerating them would have been unnecessary and would invalidate any prior local deployments.

---

## Verification

The following acceptance criteria from the tasklist were met:

1. **Program IDs consistent across all 11 files** -- All references to `sol_usd_oracle` use `GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3` and all references to `token_minter` use `3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn`.
2. **New IDs differ from original IDs** -- The old IDs (`4cuvLFF...`, `E5erGza...`) have been fully replaced.
3. **No stale references remain** -- A codebase-wide search confirms old IDs appear only in historical documentation files (`docs/idea.md`, `docs/phase/phase-4.md`, `docs/prd/ML-4.prd.md`, `docs/plan/ML-4.md`) where they serve as reference for what was replaced.
4. **`anchor build` succeeds** -- The build completes with exit code 0 using the new program IDs.
5. **LiteSVM tests pass** -- Both `oracle.litesvm.ts` and `minter.litesvm.ts` test suites pass with the updated program IDs.

---

## Impact on Downstream Work

With consistent program IDs across the entire codebase:
- **ML-5 (local E2E):** The full local cycle is unblocked -- `make deploy`, `make init`, `make backend`, and `make frontend` all reference the same programs, enabling end-to-end testing on the local validator.
- **ML-6 (devnet):** The devnet program IDs in `Anchor.toml` match the local ones, so the same keypairs can be used for devnet deployment.
- **PDA addresses change:** Since PDAs are derived from the program ID, the `ORACLE_STATE_PUBKEY` has a new value. After running `make init`, the student must copy the new PDA from the init output into `backend/.env`.
