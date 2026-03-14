# Tasklist: ML-3 — Fix Backend

**Ticket:** ML-3
**Phase:** 3 (Iteration 3)
**Status:** IMPLEMENT_STEP_OK

---

## Context

The Rust backend (`backend/src/main.rs`) has a TODO stub in the `to_fixed_6()` price parsing function that panics at runtime via `todo!()`, and the unit test `to_fixed_6_truncates_fraction_to_six_digits` has an intentionally wrong expected value (`1_123_457` instead of `1_123_456` — rounding instead of truncation). Both must be fixed so that the backend can correctly parse decimal price strings from the Binance API into fixed-point u64 values with 6 decimal places, and all 7 backend unit tests pass.

---

## Tasks

- [x] **Task 1: Implement `to_fixed_6()` in the backend**
  - **File:** `backend/src/main.rs`
  - Replace the TODO stub (lines 302-311) with string-based decimal parsing:
    - Use `split_once('.')` to separate integer and fractional parts; if no dot, treat as integer-only (fractional part is empty)
    - Parse integer part as u64, multiply by 1_000_000
    - Truncate fractional part to at most 6 characters (drop extras, do not round)
    - Right-pad fractional part with zeros to exactly 6 characters
    - Parse padded fractional part as u64 and add to scaled integer part
    - Return `Err` for non-numeric input via `?` on `.parse::<u64>()`
  - **Acceptance Criteria:**
    - The function parses decimal strings into fixed-point u64 with 6 decimal places using string splitting and integer arithmetic only (no floating-point)
    - Extra fractional digits beyond the 6th are truncated (dropped), never rounded
    - No `todo!()` macro or `let _ = ...` suppression remains in the function body
    - The function signature remains unchanged: `fn to_fixed_6(txt: &str) -> Result<u64>`
    - All PRD scenarios produce correct results: `"120"` -> `120_000_000`, `"120.12"` -> `120_120_000`, `"0.000001"` -> `1`, `"1.1234569"` -> `1_123_456`, `"99999.999999"` -> `99_999_999_999`, `"abc"` -> `Err`

- [x] **Task 2: Fix test assertion in `to_fixed_6_truncates_fraction_to_six_digits`**
  - **File:** `backend/src/main.rs`
  - Change the expected value from `1_123_457` to `1_123_456` in the `assert_eq!` on line 343
  - **Acceptance Criteria:**
    - The expected value is `1_123_456` (truncation of "1.1234569" at 6 fractional digits), not `1_123_457` (rounded)
    - No other test assertions are modified

- [x] **Task 3: Verify all backend tests pass**
  - Run `cd backend && cargo test` to confirm all 7 tests pass:
    - `to_fixed_6` tests (3):
      1. `to_fixed_6_parses_integer_and_fractional_part`
      2. `to_fixed_6_truncates_fraction_to_six_digits` (now with corrected expected value)
      3. `to_fixed_6_rejects_invalid_input`
    - Unaffected tests (4):
      4. `parse_token_created_reads_expected_fields`
      5. `parse_token_created_returns_none_for_unrelated_logs`
      6. `price_source_prefers_mock_over_url`
      7. `price_source_uses_default_url_when_no_override`
  - **Acceptance Criteria:**
    - `cargo test` completes with all 7 tests passing
    - Only one file was modified: `backend/src/main.rs`
