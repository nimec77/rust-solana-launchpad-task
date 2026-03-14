# Tasklist: ML-1 — Fix Oracle Program

**Ticket:** ML-1
**Phase:** 1 (Iteration 1)
**Status:** IMPLEMENT_STEP_OK

---

## Context

The oracle program (`sol_usd_oracle`) has a TODO stub in `apply_price_update()` that panics at runtime, and one test asserts the wrong decimals value (`8` instead of `6`). Both issues must be resolved so the oracle functions correctly and all 4 LiteSVM tests pass, establishing the foundation for downstream work (minter, backend).

---

## Tasks

- [x] **Task 1: Implement `apply_price_update()` in the oracle program**
  - **File:** `program/programs/sol_usd_oracle/src/lib.rs`
  - Replace the TODO stub (lines 10-16) with the actual state mutation logic:
    - Set `oracle.price = new_price`
    - Set `oracle.last_updated_slot = current_slot`
    - Return `Ok(())`
  - **Acceptance Criteria:**
    - The function assigns `new_price` to `oracle.price` and `current_slot` to `oracle.last_updated_slot`, then returns `Ok(())`
    - The function signature remains unchanged: `fn apply_price_update(oracle: &mut OracleState, new_price: u64, current_slot: u64) -> Result<()>`
    - No `todo!()` macro or `let _ = ...` suppression remains in the function body

- [x] **Task 2: Fix decimals assertion in the oracle test**
  - **File:** `program/tests/oracle.litesvm.ts`
  - In the "initialize_oracle sets admin and defaults" test (line 89), change `expect(decoded.decimals).to.eq(8)` to `expect(decoded.decimals).to.eq(6)`
  - **Acceptance Criteria:**
    - The assertion checks `decoded.decimals` equals `6`, matching `PRICE_DECIMALS = 6` defined in the program
    - No other test assertions are modified

- [x] **Task 3: Verify build and all oracle tests pass**
  - Run `anchor build` to confirm the program compiles successfully
  - Run `anchor test --skip-build` (or `make test`) to confirm all 4 oracle tests pass:
    1. `initialize_oracle sets admin and defaults` -- decimals assertion now correct
    2. `update_price updates price only for admin` -- apply_price_update now persists state
    3. `rejects update_price from non-admin signer` -- rejected before apply_price_update
    4. `rejects zero price update` -- rejected by `require!(new_price > 0)`
  - **Acceptance Criteria:**
    - `anchor build` completes without errors
    - All 4 oracle LiteSVM tests pass (green)
    - Only two files were modified: `program/programs/sol_usd_oracle/src/lib.rs` and `program/tests/oracle.litesvm.ts`
