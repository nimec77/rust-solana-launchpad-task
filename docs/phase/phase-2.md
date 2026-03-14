# Iteration 2: Fix Minter Program

**Goal:** Implement the fee calculation logic and fix the broken test assertion.

## Tasks

- [ ] 2.1 Implement `compute_fee_lamports()` in `program/programs/token_minter/src/lib.rs`
  - Formula: `fee_lamports = mint_fee_usd * LAMPORTS_PER_SOL / price` (use u128 intermediate)
- [ ] 2.2 Fix fee formula in `program/tests/minter.litesvm.ts` — invert the assertion formula

## Acceptance Criteria

**Test:** `anchor test --skip-build` — all program tests pass

## Dependencies

- Iteration 1 complete (oracle program must work for CPI price reads)

## Implementation Notes

- Fee formula uses 6 decimal scaling for both `mint_fee_usd` and `price`
- Use `u128` intermediate to avoid overflow during multiplication
- The test assertion has an intentionally inverted fee formula that students must fix
