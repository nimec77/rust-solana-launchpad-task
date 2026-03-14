# Summary: ML-3 — Fix Backend

**Ticket:** ML-3
**Phase:** 3 (Iteration 3)
**Date:** 2026-03-14
**Status:** Complete

---

## Overview

ML-3 was the third iteration of the Solana Mini-Launchpad project. It addressed two intentional defects in the Rust backend (`backend/src/main.rs`): a TODO stub in the `to_fixed_6()` price parsing function that panicked at runtime, and a unit test with an intentionally wrong expected value that asserted rounding behavior instead of truncation. With both on-chain programs fixed in ML-1 and ML-2, this iteration completes the backend price parsing layer -- the backend can now correctly convert decimal price strings from the Binance API into fixed-point u64 values for oracle updates.

---

## Changes Made

### 1. Implemented `to_fixed_6()` in the backend

**File:** `backend/src/main.rs`

The `to_fixed_6()` function was a TODO stub containing `todo!("student task: implement fixed-6 parser")` that would panic at runtime. It was replaced with string-based decimal parsing logic:

- Uses `split_once('.')` to separate the input into integer and fractional parts. If no decimal point is present, the fractional part defaults to an empty string.
- Parses the integer part as u64 and multiplies by 1,000,000 to scale to 6 decimal places.
- Truncates the fractional part to at most 6 characters, dropping any extra digits without rounding.
- Right-pads the fractional part with zeros to exactly 6 characters using `format!("{:0<6}", ...)`.
- Parses the padded fractional part as u64 and adds it to the scaled integer part.
- Returns `Err(anyhow::Error)` for non-numeric input via `.map_err()` on `.parse::<u64>()`.

No floating-point arithmetic is used anywhere in the implementation -- all conversion is done through string manipulation and integer parsing, ensuring deterministic results consistent with the oracle's `PRICE_DECIMALS = 6`.

### 2. Fixed test assertion in `to_fixed_6_truncates_fraction_to_six_digits`

**File:** `backend/src/main.rs`

The test asserted that `to_fixed_6("1.1234569")` returns `1_123_457`, which represents a rounded result (the 7th digit "9" would cause rounding up). The expected value was corrected to `1_123_456` to match the truncation behavior specified in the requirements: the 7th digit is simply dropped. The TODO comment marking the intentional breakage was also removed.

### 3. Code formatting

Several `rustfmt`-driven formatting adjustments were applied throughout the file (line wrapping of long function signatures and argument lists). These are cosmetic changes with no behavioral impact.

---

## Decisions

- **String-based parsing over floating-point:** The implementation avoids `f64` entirely, using string splitting and integer parsing. This eliminates floating-point precision issues that could cause subtle off-by-one errors in the 6th decimal place.
- **Truncation semantics:** Extra fractional digits beyond the 6th are dropped via string slicing (`&frac_str[..6]`) before any numeric operation, making it impossible for rounding to occur.
- **Error messages distinguish integer vs fractional failures:** Two separate `.map_err()` calls provide clear diagnostics (`"invalid integer part: ..."` vs `"invalid fractional part: ..."`).
- **Minimal scope:** Only one file was modified (`backend/src/main.rs`), consistent with the PRD constraint.

---

## Verification

All 7 backend unit tests are expected to pass with `cargo test`:

**`to_fixed_6` tests (3):**
1. `to_fixed_6_parses_integer_and_fractional_part` -- verifies basic integer+fraction parsing
2. `to_fixed_6_truncates_fraction_to_six_digits` -- now correctly expects `1_123_456` (truncated, not rounded)
3. `to_fixed_6_rejects_invalid_input` -- confirms `Err` for non-numeric input like `"abc"`

**Unaffected tests (4):**
4. `parse_token_created_reads_expected_fields`
5. `parse_token_created_returns_none_for_unrelated_logs`
6. `price_source_prefers_mock_over_url`
7. `price_source_uses_default_url_when_no_override`

---

## Impact on Downstream Work

With the backend price parsing now functional:
- **ML-5 (local E2E):** The backend price updater can correctly fetch SOL/USD prices from Binance (or use `MOCK_PRICE`), convert them to fixed-point u64, and submit `update_price` transactions to the oracle. The full local cycle (backend -> oracle -> minter -> frontend) is unblocked.
- **ML-6 (devnet):** The backend is deployment-ready for devnet operation with real Binance price data.
