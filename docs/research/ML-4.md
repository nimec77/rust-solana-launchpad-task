# Research: ML-4 — Generate Own Program IDs

**Ticket:** ML-4
**Phase:** 4 (Iteration 4)
**Date:** 2026-03-14

---

## Summary

ML-4 requires generating fresh keypairs for both Solana programs (`sol_usd_oracle` and `token_minter`) and replacing all references to the old author-owned program IDs throughout the codebase. The phase spec lists 4 locations, but the old IDs appear in 11 source files total.

---

## Current State

### Old Program IDs (Author's)

| Program | Old ID |
|---------|--------|
| `sol_usd_oracle` | `4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU` |
| `token_minter` | `E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE` |

### Existing Deploy Keypairs

Keypair files already exist at `program/target/deploy/` and currently produce **different** public keys from those hardcoded in the source:

| Keypair file | Current pubkey |
|---|---|
| `program/target/deploy/sol_usd_oracle-keypair.json` | `GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3` |
| `program/target/deploy/token_minter-keypair.json` | `3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn` |

This means the keypair files have already been regenerated at some point but the source code was never updated to match. This is the exact mismatch ML-4 is meant to fix.

---

## All Locations Containing Old Program IDs

### Primary (4 locations specified in phase-4.md)

| # | File | Line(s) | ID(s) | Notes |
|---|------|---------|-------|-------|
| 1 | `program/programs/sol_usd_oracle/src/lib.rs` | 8 | Oracle | `declare_id!("4cuv...")` |
| 2 | `program/programs/token_minter/src/lib.rs` | 16 | Minter | `declare_id!("E5er...")` |
| 3 | `program/Anchor.toml` | 9-10, 13-14 | Both | `[programs.localnet]` and `[programs.devnet]` sections |
| 4 | `frontend/app/config.ts` | 2-3 | Both | `ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID` exports |

### Additional (not in phase-4 scope, but contain hardcoded old IDs)

| # | File | Line(s) | ID(s) | Impact if left stale |
|---|------|---------|-------|---------------------|
| 5 | `program/scripts/init-local.js` | 22-23 | Both | `make init` / `make init-devnet` will fail: PDA derivation and instruction routing use wrong program ID |
| 6 | `program/scripts/get-oracle-pda.js` | 6 | Oracle | Utility script outputs wrong PDA |
| 7 | `program/tests/oracle.litesvm.ts` | 16 | Oracle | LiteSVM test loads `.so` against wrong program ID, will fail |
| 8 | `program/tests/minter.litesvm.ts` | 24-25 | Both | LiteSVM test loads both `.so` files against wrong program IDs, will fail |
| 9 | `program/migrations/deploy.ts` | 10-11 | Both | `anchor migrate` will fail |
| 10 | `backend/.env.example` | 10-11 | Both | Template for backend `.env`; misleading if stale |
| 11 | `CLAUDE.md` | 106 | Both | Documentation reference; misleading if stale |

### Documentation-only references (not requiring update for correctness)

| File | Notes |
|------|-------|
| `docs/idea.md` (line 68) | Historical context, documents the original IDs |
| `docs/phase/phase-4.md` (line 24) | Phase spec noting "current" IDs for reference |
| `docs/prd/ML-4.prd.md` (multiple lines) | PRD documenting the task itself |

---

## Implementation Approach

### Task 4.1: Generate New Keypairs

**Recommended method:** Use `anchor keys sync` from the `program/` directory.

`anchor keys sync` performs the following:
1. Reads existing keypair files from `target/deploy/<program_name>-keypair.json`
2. Extracts the public key from each
3. Updates `declare_id!()` in the corresponding program's `lib.rs`
4. Updates `Anchor.toml` entries under all `[programs.*]` sections

Since the keypair files already exist with new keys (`GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3` and `3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn`), running `anchor keys sync` will update locations 1-3 automatically.

**Alternative method** (if `anchor keys sync` is unavailable or behaves unexpectedly):
```bash
# Generate new keypairs:
solana-keygen new -o program/target/deploy/sol_usd_oracle-keypair.json --no-bip39-passphrase --force
solana-keygen new -o program/target/deploy/token_minter-keypair.json --no-bip39-passphrase --force

# Extract new public keys:
solana-keygen pubkey program/target/deploy/sol_usd_oracle-keypair.json
solana-keygen pubkey program/target/deploy/token_minter-keypair.json
```
Then manually update all locations.

### Task 4.2: Update Program IDs

**After running `anchor keys sync` (handles locations 1-3):**

Location 4 (`frontend/app/config.ts`) must be updated manually — `anchor keys sync` does not touch frontend files.

### Additional locations (beyond phase-4 scope)

The following files will break at runtime or during testing if not updated:

**Critical for functionality:**
- `program/scripts/init-local.js` (lines 22-23) — Used by `make init` and `make init-devnet`
- `program/tests/oracle.litesvm.ts` (line 16) — Used by `make test` / `anchor test`
- `program/tests/minter.litesvm.ts` (lines 24-25) — Used by `make test` / `anchor test`

**Used by migration script:**
- `program/migrations/deploy.ts` (lines 10-11) — Used by `anchor migrate`

**Utility / documentation:**
- `program/scripts/get-oracle-pda.js` (line 6) — Utility to derive oracle PDA
- `backend/.env.example` (lines 10-11) — Template for backend configuration
- `CLAUDE.md` (line 106) — AI assistant reference documentation

---

## Existing Patterns

### How program IDs are referenced

1. **Rust programs** use `declare_id!("...")` macro at module top level. This macro defines the program's on-chain address and is checked by the Anchor framework at deploy time.

2. **TypeScript/JavaScript files** use `new PublicKey("...")` constructor, typically assigned to a `const` at file top level.

3. **TOML config** (`Anchor.toml`) uses string values under `[programs.localnet]` and `[programs.devnet]` sections.

4. **`.env.example`** uses `KEY=VALUE` format without quotes.

5. **`CLAUDE.md`** references IDs inline in Markdown prose.

### Cross-program dependency

`token_minter` depends on `sol_usd_oracle` via the Cargo dependency `sol-usd-oracle = { path = "../sol_usd_oracle", features = ["cpi"] }`. The `declare_id!` in the oracle is used by the minter for CPI account validation. Both programs must be updated and then rebuilt together so that the minter's CPI validation references the correct oracle program ID.

### Build verification

The acceptance criterion is `anchor build` succeeding. This verifies:
- `declare_id!` macros compile with valid base58 keys
- The token_minter's CPI reference to `sol_usd_oracle` uses the oracle's updated `declare_id!`
- Both `.so` binaries are regenerated with the new IDs embedded

---

## Dependencies and Ordering

1. **Iteration 3 must be complete** — Backend fixes (ML-3) are a prerequisite per the phase spec.
2. **Keypair generation before source updates** — The public keys must be known before updating source files.
3. **`anchor build` after all Rust source updates** — Both `declare_id!` macros must be updated before building, since `token_minter` depends on `sol_usd_oracle`'s ID via the CPI feature.
4. **Frontend update is independent** — `frontend/app/config.ts` can be updated at any point once the new IDs are known.

---

## Risks and Limitations

| Risk | Assessment |
|------|-----------|
| **Missed references cause runtime failures** | HIGH — Tests, init scripts, and migration script all hardcode old IDs. Leaving them stale means `make test`, `make init`, and `anchor migrate` will all fail. The phase spec lists only 4 locations, but updating the additional locations is essential for a working system. |
| **`anchor keys sync` behavior** | LOW — The command is well-documented for Anchor 0.32.1. If it fails, manual update is straightforward. |
| **PDA addresses change** | EXPECTED — PDAs are derived from program ID + seeds. When program IDs change, all PDAs change. The `ORACLE_STATE_PUBKEY` in `backend/.env` (and `.env.example`) must be recalculated after the new oracle program ID is set. The init script outputs the new PDA value. |
| **Backend `.env` must be regenerated** | MEDIUM — The backend reads `ORACLE_PROGRAM_ID`, `MINTER_PROGRAM_ID`, and `ORACLE_STATE_PUBKEY` from `.env`. After updating IDs, the `.env` file must be recreated from the new `.env.example` template. The `ORACLE_STATE_PUBKEY` PDA must be recalculated. |
| **IDL files regenerated by build** | NONE — `anchor build` regenerates `target/idl/*.json` which embed the new program IDs. No manual IDL editing needed. |

---

## Resolved Questions

The PRD listed no open questions. The requirements are fully specified.

**Scope decision:** The phase-4 spec explicitly lists 4 locations. However, per the Risks section of the PRD itself, leaving additional locations stale "will cause runtime failures in tests, scripts, and backend." The PRD recommends the implementer decide whether to update all locations or strictly follow the 4-location scope. Given that the acceptance criterion is just `anchor build` succeeding, the minimum scope is the 4 listed locations. However, for a functioning system, all 11 source file locations should be updated.

---

## New Technical Questions Discovered

1. **Should `CLAUDE.md` be updated with the new IDs?** — Line 106 documents the program IDs as project constants. After ID regeneration, this documentation becomes inaccurate. Updating it keeps the AI assistant context correct. This is not critical for build success but matters for documentation accuracy.

2. **Should `docs/idea.md` be updated?** — Line 68 documents the old IDs as a historical reference. This is purely descriptive documentation of the original project state and likely should be left as-is.

---

## DEVIATIONS

**DEVIATION: Existing keypair files already contain different keys than source code.**
The deploy keypair files at `program/target/deploy/` already produce public keys (`GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3` and `3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn`) that differ from the hardcoded IDs in all source files. This means either a previous partial attempt at this task was made, or the keypairs were regenerated without updating references. Running `anchor keys sync` will synchronize the source code to match these existing keypairs. If fresh keypairs are desired instead, the existing keypair files should be deleted first and new ones generated.
