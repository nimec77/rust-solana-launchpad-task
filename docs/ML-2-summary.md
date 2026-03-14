# Summary: ML-2 — Fix Minter Program

**Ticket:** ML-2
**Phase:** 2 (Iteration 2)
**Date:** 2026-03-14
**Status:** Complete

---

## Overview

ML-2 was the second iteration of the Solana Mini-Launchpad project. It addressed two intentional defects in the `token_minter` program: a TODO stub in the fee calculation function (`compute_fee_lamports()`) that panicked at runtime, and an inverted fee formula assertion in the minter test. With the oracle program fixed in ML-1, this iteration completes the on-chain program layer — both programs now build and pass all tests.

---

## Changes Made

### 1. Implemented `compute_fee_lamports()` in the minter program

**File:** `program/programs/token_minter/src/lib.rs`

The `compute_fee_lamports()` function was a TODO stub containing `todo!("student task: implement fee conversion")` that would panic at runtime. It was replaced with the actual fee conversion logic:

- Casts `mint_fee_usd` and `LAMPORTS_PER_SOL_U64` to u128 to avoid overflow during multiplication.
- Uses `checked_mul` for the intermediate product, returning `MinterError::MathOverflow` on failure.
- Divides by `price` (as u128) — safe because the `require!(price > 0)` guard precedes it.
- Casts the result back to u64 and returns it.

The formula `fee_lamports = mint_fee_usd * LAMPORTS_PER_SOL / price` correctly converts a USD-denominated fee into lamports: when SOL price rises, the fee in lamports decreases proportionally. For the test inputs ($5.00 fee at $120.00/SOL), this produces 41,666,666 lamports (~0.04167 SOL).

### 2. Fixed fee formula assertion in the minter test

**File:** `program/tests/minter.litesvm.ts`

The test computed `expectedFee` with an inverted formula `PRICE * LAMPORTS_PER_SOL / FEE_USD`, which produced a nonsensical value that grew when SOL price increased. The operands were swapped to `FEE_USD * LAMPORTS_PER_SOL / PRICE`, matching the on-chain calculation and correct economic behavior.

---

## Decisions

- **Minimal scope:** Only two files were modified, as specified by the PRD constraints. No changes to account structures, error codes, instruction signatures, or program constants.
- **u128 intermediate with `checked_mul`:** Although u128 can handle the full range of u64 * u64 without overflow, `checked_mul` was retained as a defensive measure, consistent with the plan's specification.
- **Plain division (not `checked_div`):** Division by zero is prevented by the `require!(price > 0)` guard, making `checked_div` redundant. Plain division keeps the code concise.
- **Integer truncation accepted:** The formula truncates toward zero. This only produces a zero fee when `mint_fee_usd * LAMPORTS_PER_SOL < price` (fee less than 1 lamport), which is economically negligible for any realistic configuration.
- **TODO comment retained in test:** The `// TODO(student)` comment on line 170 of the test was kept as documentation of the original intentional breakage.

---

## Verification

All 7 LiteSVM tests pass (4 oracle + 3 minter):

**Oracle tests (4, unchanged from ML-1):**
1. `initialize_oracle sets admin and defaults`
2. `update_price updates price only for admin`
3. `rejects update_price from non-admin signer`
4. `rejects zero price update`

**Minter tests (3):**
1. `initialize oracle + minter and mint token with fee` — fee correctly computed as 41,666,666 lamports; treasury balance assertion matches; mint and ATA accounts created.
2. `rejects mint when initial supply is zero` — rejected by `InvalidSupply` before fee calculation.
3. `rejects mint when decimals exceed allowed range` — rejected by `InvalidDecimals` before fee calculation.

QA report (`docs/reports/qa/ML-2.md`) issued a RELEASE verdict with no reservations.

---

## Impact on Downstream Work

With both on-chain programs now fully functional:
- **ML-3 (backend):** The `to_fixed_6()` function in the backend can be implemented and tested. The backend price updater will call the working `update_price` instruction on the oracle.
- **ML-5 (local E2E):** End-to-end minting flow is unblocked — the minter correctly computes fees using the oracle price and transfers lamports to treasury.
- **ML-6 (devnet):** Both programs are deployment-ready for devnet.
