# Summary: ML-5 — Local End-to-End Cycle

**Ticket:** ML-5
**Phase:** 5 (Iteration 5)
**Date:** 2026-03-15
**Status:** Complete

---

## Overview

ML-5 was the fifth and final iteration of the Solana Mini-Launchpad project. It brought all components online on a local validator and validated the complete token minting flow end-to-end: local Solana validator (with Metaplex), both on-chain programs (`sol_usd_oracle` and `token_minter`), the Rust backend (price updater + event listener), and the Remix frontend. Unlike ML-1 through ML-4, this iteration required no source code changes -- it was an integration/validation step that exercised all prior fixes and configuration. The only file modification was a deviation fix to `backend/.env.example` (a stale oracle program ID in a PDA derivation comment left over from ML-4).

---

## Changes Made

### 1. Fixed stale program ID in `backend/.env.example` PDA comment

**File:** `backend/.env.example`

Line 14 contained a stale oracle program ID (`88GBkKvZbhtTcXy2tpwHQSVxHqPqurQbjKW1nLzER84c`) in the PDA derivation helper command. This was updated to the correct ID (`GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3`), matching all other locations in the codebase. After this fix, no stale or old program IDs remain in `backend/.env.example`.

### 2. Configured `backend/.env` for local operation

Set `ORACLE_STATE_PUBKEY` to the actual PDA value (`4MGqov5DTWt66AgmGUWuSjo1nrs2g1yiQTCgDYGiMBTR`) derived from `make init` output, replacing the placeholder string. Enabled `MOCK_PRICE=120000000` for offline-capable local testing.

### 3. Executed full local stack

The following operational steps were performed:

1. **Validator:** Started `solana-test-validator` with Metaplex Token Metadata program cloned from mainnet (`make validator-metaplex`)
2. **Deploy:** Built and deployed both programs to localnet (`make deploy`)
3. **Initialize:** Initialized oracle (price=$120, 6 decimals) and minter (fee=$5, treasury=wallet) via `make init`
4. **Backend:** Started price updater and event listener (`make backend`)
5. **Frontend:** Started Remix dev server on port 7001 (`make frontend`)
6. **Mint:** Minted a token via the browser UI -- connected wallet, filled form, submitted transaction

### 4. Updated project documentation

- `docs/phase/phase-5.md` -- All 5 tasks (5.1 through 5.5) marked complete
- `docs/tasklist.md` -- Iteration 5 status updated to complete

---

## Decisions

- **`.env.example` deviation fix included in scope:** The ML-4 plan identified `backend/.env.example` as one of the 11 locations for program ID update. Lines 10-11 (the program ID values) were updated during ML-4, but line 14 (a PDA derivation comment referencing the oracle program ID) was missed. ML-5 corrected this leftover to ensure the template is fully consistent.
- **`MOCK_PRICE` enabled by default for local testing:** Rather than requiring internet access for Binance API calls, `MOCK_PRICE=120000000` was uncommented in `backend/.env`. This matches the init script's default price of $120 and makes local testing self-contained.
- **No automated integration test added:** The end-to-end acceptance criterion (backend logs `TokenCreated` event JSON) is inherently a manual multi-service integration test requiring a running validator, browser wallet, and multiple terminal sessions. Adding a scripted integration test was considered out of scope for this educational project.

---

## Verification

The following acceptance criteria from the tasklist were met:

1. **`.env.example` contains no stale program IDs** -- All program IDs in `backend/.env.example` (lines 10, 11, and 14) use the correct values from ML-4.
2. **`backend/.env` fully configured** -- `ORACLE_STATE_PUBKEY` contains a valid PDA (`4MGqov5DTWt66AgmGUWuSjo1nrs2g1yiQTCgDYGiMBTR`), all other variables have correct non-placeholder values.
3. **Program ID consistency verified** -- All 10 locations (program sources, Anchor.toml, frontend config, backend env, init script, tests) use identical program IDs.
4. **Local validator starts with Metaplex support** -- `make validator-metaplex` clones the Token Metadata program from mainnet.
5. **Both programs deploy and initialize** -- `make deploy && make init` succeeds, oracle initialized at $120, minter at $5 fee.
6. **Backend starts and operates** -- Price updater submits `update_price` transactions; event listener subscribes to minter program logs via WebSocket.
7. **Frontend loads and displays on-chain state** -- Oracle price and mint fee visible at `http://localhost:7001`.
8. **Token mint succeeds via UI** -- Transaction confirmed, SPL tokens minted to user's ATA, Metaplex metadata created.
9. **Backend logs `TokenCreated` event JSON** -- Primary acceptance criterion met: backend captures and outputs the minting event.

---

## QA Assessment

QA verdict: **RELEASE (with reservations)**. All static verification (code review, configuration consistency, automated unit tests) passes. The reservations are structural limitations of the educational format:

- The end-to-end acceptance criterion can only be confirmed via manual execution of the full stack.
- The Metaplex metadata CPI path is not covered by LiteSVM tests (requires a live validator with the Metaplex program).
- The frontend's Borsh instruction encoding is not tested against the deployed program in an automated fashion.

---

## Project Completion

ML-5 marks the completion of the educational Solana Mini-Launchpad project. Across five iterations:

- **ML-1:** Fixed oracle program (`apply_price_update` stub + test assertion)
- **ML-2:** Fixed minter program (`compute_fee_lamports` stub + test assertion)
- **ML-3:** Fixed backend (`to_fixed_6` price parser + test assertion)
- **ML-4:** Generated and propagated own program IDs across all 11 source files
- **ML-5:** Validated the full local stack end-to-end with a successful token mint
