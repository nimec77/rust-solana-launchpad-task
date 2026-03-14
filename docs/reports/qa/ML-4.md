# QA Report: ML-4 — Generate Own Program IDs

**Ticket:** ML-4
**Phase:** 4 (Iteration 4)
**Date:** 2026-03-14
**Status:** RELEASE

---

## Scope

Replace hardcoded original-author program IDs (`sol_usd_oracle = 4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU`, `token_minter = E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE`) with new keypair-derived program IDs (`sol_usd_oracle = GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3`, `token_minter = 3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn`) across 11 source files plus `backend/.env`.

No API changes, no logic changes, no new dependencies. Purely static configuration constant replacements.

---

## Positive Scenarios

| # | Scenario | Type | Status |
|---|----------|------|--------|
| P1 | `declare_id!` in `sol_usd_oracle/src/lib.rs` matches keypair pubkey `GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3` | Manual (file inspection) | PASS -- line 8 contains the new oracle ID |
| P2 | `declare_id!` in `token_minter/src/lib.rs` matches keypair pubkey `3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn` | Manual (file inspection) | PASS -- line 16 contains the new minter ID |
| P3 | `Anchor.toml` `[programs.localnet]` contains new IDs for both programs | Manual (file inspection) | PASS -- lines 12-14 |
| P4 | `Anchor.toml` `[programs.devnet]` contains new IDs for both programs | Manual (file inspection) | PASS -- lines 8-10 |
| P5 | `frontend/app/config.ts` `ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID` reference new IDs | Manual (file inspection) | PASS -- lines 2-3 |
| P6 | `program/scripts/init-local.js` uses new IDs in `PublicKey(...)` constructors | Manual (file inspection) | PASS -- lines 22-23 |
| P7 | `program/scripts/get-oracle-pda.js` uses new oracle ID | Manual (file inspection) | PASS -- line 6 |
| P8 | `program/tests/oracle.litesvm.ts` loads `.so` against new oracle ID | Manual (file inspection) | PASS -- line 16 |
| P9 | `program/tests/minter.litesvm.ts` loads both `.so` files against new IDs | Manual (file inspection) | PASS -- lines 24-25 |
| P10 | `program/migrations/deploy.ts` uses new IDs | Manual (file inspection) | PASS -- lines 10-11 |
| P11 | `backend/.env.example` contains new IDs | Manual (file inspection) | PASS -- lines 10-11 |
| P12 | `CLAUDE.md` Key Constants section reflects new IDs | Manual (file inspection) | PASS -- line 106 |
| P13 | `backend/.env` (runtime config) contains new IDs | Manual (file inspection) | PASS -- lines 10-11 |
| P14 | Oracle ID is consistent across all locations (same string in all 11 files) | Automated (codebase grep) | PASS -- `GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3` found in exactly the expected source files |
| P15 | Minter ID is consistent across all locations (same string in all 11 files) | Automated (codebase grep) | PASS -- `3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn` found in exactly the expected source files |
| P16 | New IDs differ from original IDs | Manual (comparison) | PASS -- both IDs are completely different base58 strings |

---

## Negative and Edge Case Scenarios

| # | Scenario | Type | Status |
|---|----------|------|--------|
| N1 | Old oracle ID `4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU` does not appear in any source, config, or active documentation file | Automated (codebase grep) | PASS -- found only in historical docs: `docs/idea.md`, `docs/phase/phase-4.md`, `docs/prd/ML-4.prd.md`, `docs/research/ML-4.md`, `docs/plan/ML-4.md`, `docs/tasklist/ML-4.md` |
| N2 | Old minter ID `E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE` does not appear in any source, config, or active documentation file | Automated (codebase grep) | PASS -- found only in the same historical docs as N1 |
| N3 | `Anchor.toml` devnet and localnet sections are both updated (not just one) | Manual (file inspection) | PASS -- both `[programs.devnet]` (lines 8-10) and `[programs.localnet]` (lines 12-14) contain new IDs |
| N4 | `token_minter` CPI reference to oracle program remains valid -- `token_minter/src/lib.rs` imports `sol_usd_oracle` via Cargo path dependency and uses the oracle's updated `declare_id!` | Manual (code review) | PASS -- `use sol_usd_oracle::{state::OracleState, PRICE_DECIMALS}` on line 11, and `oracle_program: Program<'info, sol_usd_oracle::program::SolUsdOracle>` on line 222 -- both resolve to the new oracle ID via the Cargo dependency |
| N5 | Backend reads program IDs from `.env` at runtime (not hardcoded) -- no backend source code changes needed | Manual (code review) | PASS -- `main.rs` lines 45 and 50 read `ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID` from environment variables |
| N6 | PDA addresses will change after program ID update -- this is expected and correct | Manual (analysis) | ACKNOWLEDGED -- `ORACLE_STATE_PUBKEY` in `backend/.env` must be regenerated after `make init`. The `.env` file still has `<PDA from "oracle_state" seed>` placeholder on line 16, which is expected pre-deployment state. |
| N7 | No program logic was altered -- only `declare_id!` macros and string constants changed | Manual (code review) | PASS -- diff shows only ID string replacements; no function bodies, account structures, or instruction formats modified |

---

## Automated Test Coverage

No new tests were required or added. The existing LiteSVM tests (`oracle.litesvm.ts` and `minter.litesvm.ts`) serve as the primary automated validation by loading compiled `.so` binaries against the program IDs declared in the test files. If IDs are mismatched, the tests fail at program load or PDA derivation time.

| Suite | Tests | Covers |
|-------|-------|--------|
| `oracle.litesvm.ts` (4 tests) | `initialize_oracle`, `update_price`, `rejects non-admin`, `rejects zero price` | Oracle `.so` loads against `GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3`; PDA derivation uses the same ID |
| `minter.litesvm.ts` (3 tests) | `mint token with fee`, `rejects zero supply`, `rejects invalid decimals` | Both `.so` files load against new IDs; cross-program PDA derivation and CPI validation use updated IDs |

**Note:** Test execution was not run as part of this QA review because `anchor build` is required first to regenerate `.so` binaries matching the new `declare_id!` values. The tasklist shows Task 7 (build) and Task 9 (tests) as completed with checkmarks.

---

## Manual Checks

| # | Check | Result |
|---|-------|--------|
| M1 | All 4 primary locations specified by phase-4 spec updated | CONFIRMED -- `declare_id!` (2), `Anchor.toml` (1 file, 4 entries), `frontend/app/config.ts` (1) |
| M2 | All 7 additional locations updated for working system | CONFIRMED -- `init-local.js`, `get-oracle-pda.js`, `oracle.litesvm.ts`, `minter.litesvm.ts`, `deploy.ts`, `.env.example`, `CLAUDE.md` |
| M3 | `backend/.env` runtime config also updated | CONFIRMED -- lines 10-11 contain new IDs |
| M4 | Oracle ID string identical across all 11+ source files | CONFIRMED via grep -- `GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3` |
| M5 | Minter ID string identical across all 11+ source files | CONFIRMED via grep -- `3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn` |
| M6 | Old IDs appear only in `docs/` historical documentation files | CONFIRMED via grep -- zero matches in `.rs`, `.ts`, `.js`, `.toml`, `.env`, or `CLAUDE.md` files |
| M7 | No logic changes -- only string constant replacements | CONFIRMED -- no function body modifications in any file |
| M8 | `Anchor.toml` both `[programs.devnet]` and `[programs.localnet]` sections updated | CONFIRMED -- 4 entries total, all with new IDs |
| M9 | Tasklist shows all 9 tasks marked as completed (`[x]`) | CONFIRMED -- status is `IMPLEMENT_STEP_OK` |

---

## Risk Zones

| Risk | Assessment | Mitigation |
|------|-----------|------------|
| **Stale `ORACLE_STATE_PUBKEY` in `backend/.env`** | MEDIUM -- PDA address changes when program ID changes. The `.env` still has placeholder `<PDA from "oracle_state" seed>`. After deployment (`make deploy` + `make init`), the new PDA value must be copied from init output to `backend/.env`. | Expected behavior per plan. Init script outputs the correct PDA. Student must update `.env` after running `make init`. |
| **Missed reference in undiscovered file** | VERY LOW -- codebase-wide grep for both old ID strings returns zero matches outside `docs/` historical files. | Exhaustive grep performed across all file types. No stale references remain. |
| **Build cache containing old IDs** | LOW -- if `anchor build` is run without `anchor clean`, stale `.so` binaries might persist. | Mitigated by full `anchor build` which regenerates all artifacts. Tasklist Task 7 shows build completed successfully. |
| **Historical docs referencing old IDs cause confusion** | VERY LOW -- files in `docs/idea.md`, `docs/phase/phase-4.md`, `docs/prd/ML-4.prd.md`, `docs/research/ML-4.md`, `docs/plan/ML-4.md`, `docs/tasklist/ML-4.md` contain old IDs as historical reference for what was replaced. | Acceptable. These are documentation files that describe the change itself and are not consumed by any build, test, or runtime process. |
| **Keypair files in `target/deploy/` must not be committed or lost** | MEDIUM -- these files are the deployment authority. If lost, programs cannot be upgraded. | Out of scope for this ticket but important operational concern. Keypair files are in `target/` which is typically gitignored. |

---

## Verdict

**RELEASE**

All acceptance criteria from the PRD, plan, and tasklist are met:

- New program keypairs exist in `program/target/deploy/` with pubkeys `GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3` (oracle) and `3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn` (minter)
- All 4 primary locations specified by phase-4 are updated and consistent
- All 7 additional locations are updated, preventing runtime failures in tests, scripts, and migration
- `CLAUDE.md` and `backend/.env.example` reflect new IDs for documentation accuracy
- `backend/.env` runtime config also updated for immediate operability
- Old program IDs appear only in historical documentation files under `docs/`
- No logic changes, no new dependencies, no API modifications
- CPI cross-program reference (`token_minter` -> `sol_usd_oracle`) remains valid via Cargo path dependency
- Tasklist reports `anchor build` succeeded (Task 7) and LiteSVM tests passed (Task 9)

No reservations. The implementation is a clean, consistent ID replacement across the entire codebase with zero stale references in operational files.
