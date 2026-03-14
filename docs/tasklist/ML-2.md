# Tasklist: ML-2 — Fix Minter Program

**Ticket:** ML-2
**Phase:** 2 (Iteration 2)
**Status:** IMPLEMENT_STEP_OK

---

## Context

The token minter program (`token_minter`) has a TODO stub in `compute_fee_lamports()` that panics at runtime via `todo!()`, and the minter test has an intentionally inverted fee formula assertion (`PRICE * LAMPORTS_PER_SOL / FEE_USD` instead of the correct `FEE_USD * LAMPORTS_PER_SOL / PRICE`). Both must be fixed so that the fee calculation works correctly and all 7 LiteSVM tests pass (4 oracle + 3 minter).

---

## Tasks

- [x] **Task 1: Implement `compute_fee_lamports()` in the minter program**
  - **File:** `program/programs/token_minter/src/lib.rs`
  - Replace the TODO stub (lines 169-174) with u128 intermediate arithmetic:
    - Cast `mint_fee_usd` and `LAMPORTS_PER_SOL_U64` to u128
    - Multiply using `checked_mul`, returning `MinterError::MathOverflow` on failure
    - Divide by `price` (as u128) — safe because the `require!(price > 0)` guard precedes it
    - Cast result back to u64 and return `Ok(fee as u64)`
  - **Acceptance Criteria:**
    - The function computes `fee_lamports = mint_fee_usd * LAMPORTS_PER_SOL / price` using u128 intermediate arithmetic with `checked_mul` overflow protection
    - No `todo!()` macro or `let _ = ...` suppression remains in the function body
    - The function signature remains unchanged: `fn compute_fee_lamports(mint_fee_usd: u64, price: u64) -> Result<u64>`
    - For inputs `mint_fee_usd = 5_000_000` and `price = 120_000_000`, the function returns `41_666_666` lamports

- [x] **Task 2: Fix fee formula assertion in the minter test**
  - **File:** `program/tests/minter.litesvm.ts`
  - At line 172, change `PRICE.mul(new BN(anchor.web3.LAMPORTS_PER_SOL)).div(FEE_USD)` to `FEE_USD.mul(new BN(anchor.web3.LAMPORTS_PER_SOL)).div(PRICE)`
  - **Acceptance Criteria:**
    - The `expectedFee` computation uses the correct formula: `FEE_USD * LAMPORTS_PER_SOL / PRICE`
    - The expected fee value equals `41_666_666` lamports for the test inputs ($5.00 fee at $120.00/SOL)
    - No other test assertions are modified

- [x] **Task 3: Verify build and all tests pass**
  - Run `anchor build` to confirm both programs compile successfully
  - Run `anchor test --skip-build` (or `make test`) to confirm all 7 tests pass:
    - Oracle tests (4, unchanged from ML-1):
      1. `initialize_oracle sets admin and defaults`
      2. `update_price updates price only for admin`
      3. `rejects update_price from non-admin signer`
      4. `rejects zero price update`
    - Minter tests (3):
      1. `initialize oracle + minter and mint token with fee` — fee correctly computed as 41_666_666 lamports, assertion matches
      2. `rejects mint when initial supply is zero` — rejected by `InvalidSupply`
      3. `rejects mint when decimals exceed allowed range` — rejected by `InvalidDecimals`
  - **Acceptance Criteria:**
    - `anchor build` completes without errors
    - All 7 LiteSVM tests pass (4 oracle + 3 minter)
    - Only two files were modified: `program/programs/token_minter/src/lib.rs` and `program/tests/minter.litesvm.ts`
