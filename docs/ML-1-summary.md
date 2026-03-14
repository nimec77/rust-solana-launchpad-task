# Summary: ML-1 — Fix Oracle Program

**Ticket:** ML-1
**Phase:** 1 (Iteration 1)
**Date:** 2026-03-14
**Status:** Complete

---

## Overview

ML-1 was the first iteration of the Solana Mini-Launchpad project. It addressed two intentional defects in the `sol_usd_oracle` program that blocked all downstream work (token minter, backend, deployment).

---

## Changes Made

### 1. Implemented `apply_price_update()` in the oracle program

**File:** `program/programs/sol_usd_oracle/src/lib.rs`

The `apply_price_update()` function was a TODO stub containing a `todo!()` macro that would panic at runtime. It was replaced with the actual state mutation logic:

- `oracle.price = new_price` — persists the latest SOL/USD price to the on-chain PDA.
- `oracle.last_updated_slot = current_slot` — records the slot at which the price was last refreshed.
- Returns `Ok(())` on success.

Both assignments are direct `u64` field writes with no arithmetic, eliminating any overflow risk.

### 2. Fixed decimals assertion in the oracle test

**File:** `program/tests/oracle.litesvm.ts`

The "initialize_oracle sets admin and defaults" test asserted `decoded.decimals == 8`, but the oracle program defines `PRICE_DECIMALS = 6`. The assertion was corrected to `expect(decoded.decimals).to.eq(6)`.

---

## Decisions

- **Minimal scope:** Only two files were modified, as specified by the PRD constraints. No changes to account structures, error codes, instruction signatures, or program constants.
- **No architectural changes:** The `apply_price_update()` function signature remained unchanged. The fix was a straightforward implementation of the intended behavior.
- **No new dependencies:** The implementation required no new crates, libraries, or tooling changes.

---

## Verification

- All 4 oracle LiteSVM tests are expected to pass:
  1. `initialize_oracle sets admin and defaults` — decimals assertion now matches `PRICE_DECIMALS = 6`
  2. `update_price updates price only for admin` — `apply_price_update()` now persists state correctly
  3. `rejects update_price from non-admin signer` — unchanged (rejected before `apply_price_update`)
  4. `rejects zero price update` — unchanged (rejected by `require!(new_price > 0)`)

---

## Test Coverage Gaps (non-blocking)

- No test asserts `last_updated_slot` value after `update_price` (difficult to predict slot in LiteSVM).
- No test for `u64::MAX` price edge case (safe due to direct assignment).
- No test for double initialization (covered by Anchor's `init` constraint).

---

## Impact on Downstream Work

The working oracle is a prerequisite for:
- **ML-2 (token minter):** `compute_fee_lamports()` reads `oracle.price` via CPI account validation to calculate minting fees.
- **Backend:** The price updater service calls `update_price` to keep the oracle current.
- **Frontend:** Polls oracle state to display the current SOL/USD price.
