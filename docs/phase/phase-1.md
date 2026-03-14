# Iteration 1: Fix Oracle Program

**Goal:** Implement the oracle price update logic and fix the broken test assertion.

## Tasks

- [ ] 1.1 Implement `apply_price_update()` in `program/programs/sol_usd_oracle/src/lib.rs`
  - Set `oracle.price = new_price` and `oracle.last_updated_slot = current_slot`
- [ ] 1.2 Fix decimals assertion in `program/tests/oracle.litesvm.ts` (`8` → `6`)

## Acceptance Criteria

**Test:** `anchor build && anchor test --skip-build` — oracle tests pass

## Dependencies

- None (first iteration)

## Implementation Notes

- Oracle price uses 6 decimal places (not 8)
- `apply_price_update()` is a TODO stub that students must fill in
- The test file has an intentional wrong assertion (`decimals == 8` should be `decimals == 6`)
