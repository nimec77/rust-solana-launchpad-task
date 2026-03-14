# Iteration 3: Fix Backend

**Goal:** Implement the price parsing function and fix the broken test assertion.

## Tasks

- [ ] 3.1 Implement `to_fixed_6()` in `backend/src/main.rs`
  - Parse decimal string → `u64` with 6 fixed decimals; truncate, don't round
- [ ] 3.2 Fix test `to_fixed_6_truncates_fraction_to_six_digits` — change expected `1_123_457` → `1_123_456`

## Acceptance Criteria

**Test:** `cargo test` — all backend tests pass

## Dependencies

- Iteration 2 complete (minter program must work for full integration)

## Implementation Notes

- `to_fixed_6()` converts a decimal string (e.g., "123.456789") to a fixed-point u64 with 6 decimal places
- Truncate fractional digits beyond 6 (do not round)
- The test has an intentionally wrong expected value (`1_123_457` should be `1_123_456`)
