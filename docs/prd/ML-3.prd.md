# PRD: ML-3 — Fix Backend

**Status:** PRD_READY
**Ticket:** ML-3
**Phase:** 3 (Iteration 3)

---

## Context / Idea

**Arguments:** ML-3 docs/phase/phase-3.md

This is the third iteration of the Solana Mini-Launchpad project. The Rust backend (`backend/src/main.rs`) has a TODO stub in the `to_fixed_6()` price parsing function and one of its unit tests has an intentionally wrong expected value. Both must be fixed so that the backend can correctly parse decimal price strings from the Binance API (or other sources) into fixed-point u64 values with 6 decimal places.

### Source: docs/phase/phase-3.md

> **Goal:** Implement the price parsing function and fix the broken test assertion.
>
> **Tasks:**
> - Implement `to_fixed_6()` in `backend/src/main.rs`
>   - Parse decimal string -> `u64` with 6 fixed decimals; truncate, don't round
> - Fix test `to_fixed_6_truncates_fraction_to_six_digits` — change expected `1_123_457` -> `1_123_456`
>
> **Acceptance Criteria:**
> `cargo test` — all backend tests pass
>
> **Dependencies:**
> - Iteration 2 complete (minter program must work for full integration)
>
> **Implementation Notes:**
> - `to_fixed_6()` converts a decimal string (e.g., "123.456789") to a fixed-point u64 with 6 decimal places
> - Truncate fractional digits beyond 6 (do not round)
> - The test has an intentionally wrong expected value (`1_123_457` should be `1_123_456`)

---

## Goals

1. **Implement price parsing function** — The `to_fixed_6()` function in `backend/src/main.rs` must parse a decimal string (e.g., "120.12") into a u64 with 6 fixed decimal places (e.g., 120_120_000), truncating (not rounding) any digits beyond the 6th decimal place.
2. **Fix broken test assertion** — The test `to_fixed_6_truncates_fraction_to_six_digits` asserts `1_123_457` but the correct truncated value of "1.1234569" at 6 decimals is `1_123_456`. The expected value must be corrected.
3. **Establish a passing backend test baseline** — All backend unit tests (`cargo test`) must pass so that subsequent iterations (local end-to-end, devnet deployment) can rely on a working backend price updater.

---

## User Stories

1. **As a backend operator**, I want `to_fixed_6()` to correctly parse decimal price strings from the Binance API so that the oracle is updated with accurate SOL/USD prices in the project's 6-decimal fixed-point format.
2. **As a developer**, I want the backend test suite to pass with correct assertions so that I can trust the tests as a regression safety net for the price parsing logic.
3. **As a downstream consumer (oracle program)**, I need the backend to supply correctly scaled u64 price values so that `update_price` transactions store meaningful prices that the minter can use for fee calculation.

---

## Scenarios

### Scenario 1: Parse integer-only price string

- **Given** the input string is "120" (no decimal point)
- **When** `to_fixed_6("120")` is called
- **Then** the result is `120_000_000` (120 * 10^6)

### Scenario 2: Parse price string with fractional part shorter than 6 digits

- **Given** the input string is "120.12" (2 decimal digits)
- **When** `to_fixed_6("120.12")` is called
- **Then** the result is `120_120_000` (fractional part zero-padded to 6 digits)

### Scenario 3: Parse price string with exactly 6 fractional digits

- **Given** the input string is "0.000001"
- **When** `to_fixed_6("0.000001")` is called
- **Then** the result is `1`

### Scenario 4: Parse price string with more than 6 fractional digits — truncate, do not round

- **Given** the input string is "1.1234569" (7 decimal digits)
- **When** `to_fixed_6("1.1234569")` is called
- **Then** the result is `1_123_456` (the 7th digit "9" is truncated, NOT rounded up to `1_123_457`)

### Scenario 5: Invalid input rejected

- **Given** the input string is "abc" (non-numeric)
- **When** `to_fixed_6("abc")` is called
- **Then** the function returns an `Err`

### Scenario 6: Price with high value

- **Given** the input string is "99999.999999" (typical high price)
- **When** `to_fixed_6("99999.999999")` is called
- **Then** the result is `99_999_999_999` and fits within u64

---

## Metrics

| Metric | Target |
|--------|--------|
| Backend unit tests passing | 7/7 (all tests green) |
| `cargo test` succeeds | Yes |
| `to_fixed_6` handles all documented example inputs correctly | Yes |

---

## Constraints

1. **Minimal change scope** — Only one file is modified: `backend/src/main.rs`. No other files should be touched.
2. **No architectural changes** — The `to_fixed_6()` function signature (`fn to_fixed_6(txt: &str) -> Result<u64>`) must remain unchanged.
3. **Truncation, not rounding** — Extra fractional digits beyond the 6th decimal place must be truncated (dropped), not rounded. This is explicitly specified in the requirements.
4. **6 decimal places** — The output is a u64 representing a fixed-point number with 6 decimal places, consistent with the oracle's `PRICE_DECIMALS = 6`.
5. **Rust toolchain 1.89.0** — Must compile with the project's pinned Rust version.
6. **Iteration 2 dependency** — Minter program must be working (Iteration 2 complete) for full integration, though the backend unit tests can run independently.

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Overflow when multiplying integer part by 10^6 | Very Low | High | u64 max is ~1.8 * 10^19; for overflow the integer part would need to exceed ~1.8 * 10^13, which is not a realistic SOL/USD price |
| Edge case with empty fractional part (e.g., "120.") | Low | Low | Treat empty fractional part as zero; test coverage should include this case or document behavior |
| Locale-specific decimal separators (comma vs period) | Low | Low | The Binance API always uses period as decimal separator; no locale handling needed |
| Test fix masks a real bug | Very Low | Medium | The wrong expected value (`1_123_457`) is a deliberate rounding-vs-truncation error in the test, not a code bug |

---

## Open Questions

None. The requirements are fully specified and self-contained. Implementation can proceed.

---

## Implementation Reference

### File 1: `backend/src/main.rs`

**Location:** `to_fixed_6()` function (line 302-311)

**Current code (TODO stub):**
```rust
fn to_fixed_6(txt: &str) -> Result<u64> {
    // TODO(student): parse a decimal string into an integer with 6 fixed decimals.
    // Examples:
    // - "120" -> 120_000_000
    // - "120.12" -> 120_120_000
    // - "0.000001" -> 1
    // Extra digits after the 6th decimal place should be truncated, not rounded.
    let _ = txt;
    todo!("student task: implement fixed-6 parser")
}
```

**Required:** Parse the input string by splitting on the decimal point. Convert the integer part and multiply by 10^6. Take up to 6 characters of the fractional part (truncating any beyond 6), right-pad with zeros if fewer than 6 digits, parse as u64, and add to the scaled integer part. Return an error for non-numeric input.

### File 2: `backend/src/main.rs`

**Location:** `to_fixed_6_truncates_fraction_to_six_digits` test (line 340-344)

**Current code (wrong expected value):**
```rust
#[test]
fn to_fixed_6_truncates_fraction_to_six_digits() {
    // TODO(student): this assertion is intentionally wrong.
    // The parser is expected to truncate after 6 digits instead of rounding.
    assert_eq!(to_fixed_6("1.1234569").unwrap(), 1_123_457);
}
```

**Required:** Change `1_123_457` to `1_123_456` to match the truncation behavior (the 7th digit "9" is dropped, not used for rounding).
