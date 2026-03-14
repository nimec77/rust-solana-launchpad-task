# Research: ML-3 — Fix Backend

**Ticket:** ML-3
**Phase:** 3 (Iteration 3)
**Date:** 2026-03-14

---

## Summary

Two targeted fixes in the Rust backend (`backend/src/main.rs`): implement the `to_fixed_6()` price parsing function (decimal string to fixed-point u64 with 6 decimal places, truncation not rounding) and correct a broken test assertion that expects a rounded value instead of a truncated one. After these changes, `cargo test` should pass all 7 backend unit tests.

---

## Existing Code Analysis

### File 1: `backend/src/main.rs` — `to_fixed_6()` function

**Location:** Lines 302-311

The function currently contains a TODO stub:

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

**Function signature:** `fn to_fixed_6(txt: &str) -> Result<u64>` — uses `anyhow::Result` (imported at line 4). Must remain unchanged per PRD constraint.

**Caller context (lines 101-113):** The `PriceSource::fetch_price()` method calls `to_fixed_6(&resp.price)` where `resp.price` is a JSON string from the Binance API (e.g., `"120.12345678"`). The returned u64 is then passed to `submit_price()` which sends an `update_price` instruction to the oracle program.

**Data flow:**
```
Binance API JSON -> resp.price (String) -> to_fixed_6() -> u64 -> submit_price() -> Oracle PDA
```

**Required implementation algorithm:**

1. Split the input string on `.` (decimal point)
2. Parse the integer part as u64, multiply by 1_000_000 (10^6)
3. If there is a fractional part:
   - Take at most 6 characters (truncating any beyond the 6th)
   - Right-pad with zeros if fewer than 6 characters (e.g., "12" becomes "120000")
   - Parse the padded fractional part as u64
4. Add the scaled integer part and the fractional part
5. Return `Err` for non-numeric input

**Error handling:** The function returns `anyhow::Result<u64>`. The `anyhow!` macro (imported at line 4) and the `?` operator with `.parse::<u64>()` can be used for error propagation. Non-numeric input naturally causes `.parse()` to fail.

**Edge cases to consider:**
- No decimal point: "120" -> integer only, fractional part is 0
- Trailing dot: "120." -> fractional part is empty string, treated as 0
- More than one dot: "1.2.3" -> after splitting, only the first two parts matter; however, this would be invalid if the integer or fraction part contains a dot. Using `split_once('.')` avoids ambiguity.
- Empty string: parse would fail -> Err

### File 2: `backend/src/main.rs` — broken test assertion

**Location:** Lines 340-344

```rust
#[test]
fn to_fixed_6_truncates_fraction_to_six_digits() {
    // TODO(student): this assertion is intentionally wrong.
    // The parser is expected to truncate after 6 digits instead of rounding.
    assert_eq!(to_fixed_6("1.1234569").unwrap(), 1_123_457);
}
```

The input "1.1234569" has 7 fractional digits. With truncation at 6 digits, the fractional part becomes "123456" (the 7th digit "9" is dropped). The result should be `1 * 1_000_000 + 123_456 = 1_123_456`.

The current expected value `1_123_457` represents a rounded result (123456.9 rounded up to 123457). This contradicts the truncation requirement.

**Required fix:** Change `1_123_457` to `1_123_456`.

### All Backend Tests (7 total)

| # | Test Name | Lines | Current Status | After Fix |
|---|-----------|-------|----------------|-----------|
| 1 | `to_fixed_6_parses_integer_and_fractional_part` | 333-337 | Fails (todo!() panic) | Passes |
| 2 | `to_fixed_6_truncates_fraction_to_six_digits` | 340-344 | Fails (todo!() panic + wrong expected) | Passes |
| 3 | `to_fixed_6_rejects_invalid_input` | 347-349 | Fails (todo!() panic) | Passes |
| 4 | `parse_token_created_reads_expected_fields` | 352-369 | Passes | Passes (unaffected) |
| 5 | `parse_token_created_returns_none_for_unrelated_logs` | 372-379 | Passes | Passes (unaffected) |
| 6 | `price_source_prefers_mock_over_url` | 382-389 | Passes | Passes (unaffected) |
| 7 | `price_source_uses_default_url_when_no_override` | 392-401 | Passes | Passes (unaffected) |

Tests 1-3 all currently panic at the `todo!()` macro. Tests 4-7 are unrelated to `to_fixed_6` and currently pass.

---

## Layers and Dependencies

### Build Dependencies
- **Rust toolchain:** 1.89.0 (pinned in `program/rust-toolchain.toml`)
- **anyhow:** 1.0.93 (for `Result` type and error handling)
- **sol_usd_oracle crate:** path dependency `../program-task/programs/sol_usd_oracle` with `no-entrypoint` feature (used for `accounts::UpdatePrice` and `instruction::UpdatePrice` in `submit_price()`, not in `to_fixed_6`)

### Data Flow Context
```
to_fixed_6() -> u64 price
     |
     v
submit_price() -> update_price instruction -> Oracle PDA (price field: u64, 6 decimals)
     |
     v
token_minter reads Oracle PDA -> compute_fee_lamports() -> fee in lamports
```

The `to_fixed_6()` function is the entry point for price data into the entire system. Its output must be a u64 representing a price with 6 decimal places (e.g., 120_000_000 = $120.00), matching the oracle's `PRICE_DECIMALS = 6` constant defined in `program/programs/sol_usd_oracle/src/lib.rs` line 3.

### Iteration 2 Dependency (CONFIRMED SATISFIED)
The minter program's `compute_fee_lamports()` function has been implemented (commit `7d6fdd1`). The minter test assertion has been fixed. All program tests pass. The backend can be fixed independently since its unit tests do not require a running validator or deployed programs.

---

## Patterns Used

1. **Fixed-point 6-decimal scaling** — Consistent throughout the project: oracle price, minter fee, and this parser all use u64 with 10^6 scaling. No floating point anywhere in the data path.
2. **`split_once` for decimal parsing** — Rust's `str::split_once('.')` cleanly separates integer and fractional parts, handling the no-decimal-point case (returns `None`) and the trailing-dot case (fractional part is `""`).
3. **Truncation by string slicing** — Taking `&frac[..6.min(frac.len())]` or equivalent to truncate fractional digits is simpler and more explicit than numeric rounding approaches. Avoids any floating-point contamination.
4. **Right-padding with format macro** — `format!("{:0<6}", truncated_frac)` right-pads the fractional string with zeros to exactly 6 characters, making it directly parseable as the fractional u64 value.
5. **anyhow error propagation** — The `?` operator on `.parse::<u64>()` automatically converts `ParseIntError` into `anyhow::Error`. No custom error types needed for this utility function.

---

## Implementation Plan

### Change 1: Implement `to_fixed_6()` body

Replace the TODO stub (lines 303-310) in `backend/src/main.rs` with the parsing logic:

```rust
fn to_fixed_6(txt: &str) -> Result<u64> {
    let (int_str, frac_str) = match txt.split_once('.') {
        Some((i, f)) => (i, f),
        None => (txt, ""),
    };

    let int_part: u64 = int_str.parse()
        .map_err(|_| anyhow!("invalid integer part: {}", int_str))?;

    let frac_truncated = if frac_str.len() > 6 {
        &frac_str[..6]
    } else {
        frac_str
    };

    let frac_padded = format!("{:0<6}", frac_truncated);
    let frac_part: u64 = frac_padded.parse()
        .map_err(|_| anyhow!("invalid fractional part: {}", frac_str))?;

    Ok(int_part * 1_000_000 + frac_part)
}
```

**Key decisions:**
- `split_once('.')` handles both "120" (no dot) and "120." (empty fractional part) correctly
- Fractional string truncation at 6 characters ensures no rounding
- `format!("{:0<6}", ...)` right-pads to exactly 6 digits
- Two separate `.parse()` calls provide clear error messages for invalid integer vs fractional parts
- Empty `frac_str` (`""`) gets padded to `"000000"`, parsing to `0`

**Potential concern — empty integer part:** An input like ".5" would have `int_str = ""`, which would fail to parse as u64. This is acceptable behavior since the Binance API always returns a leading zero for sub-1 values (e.g., "0.5"), and the PRD does not require handling this edge case.

**Potential concern — overflow:** `int_part * 1_000_000` can overflow u64 if `int_part > 18_446_744_073_709`. This is not a realistic SOL/USD price. For additional safety, `checked_mul` could be used, but the PRD does not require it and existing tests do not test overflow.

### Change 2: Fix test expected value

At line 343, change:
```rust
assert_eq!(to_fixed_6("1.1234569").unwrap(), 1_123_457);
```
to:
```rust
assert_eq!(to_fixed_6("1.1234569").unwrap(), 1_123_456);
```

### Verification

Run: `cd backend && cargo test`

Expected result: all 7 tests pass.

Test coverage of the `to_fixed_6` function:

| Test Input | Expected Output | Scenario |
|------------|----------------|----------|
| `"120"` | `120_000_000` | Integer only, no decimal point |
| `"120.12"` | `120_120_000` | Short fractional part, zero-padded |
| `"0.000001"` | `1` | Minimal non-zero value |
| `"1.1234569"` | `1_123_456` | 7 digits truncated to 6 (not rounded) |
| `"abc"` | `Err(...)` | Non-numeric input rejected |

---

## Limitations and Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Overflow in `int_part * 1_000_000` | Very Low | High | u64 max is ~1.8 * 10^19; integer part would need to exceed ~1.8 * 10^13, not a realistic SOL/USD price. Could use `checked_mul` for defense but PRD does not require it. |
| Leading/trailing whitespace in API response | Low | Medium | `reqwest`'s JSON deserializer trims whitespace from string values. Not a concern in practice. |
| Input "120." (trailing dot, empty fraction) | Low | None | Handled correctly: `frac_str = ""`, padded to `"000000"`, parses to `0`. Result: `120_000_000`. |
| Multiple decimal points ("1.2.3") | Very Low | Low | `split_once` takes only the first dot. `frac_str = "2.3"` would fail to parse as u64, returning Err. This is correct behavior for invalid input. |
| Negative prices ("-120.5") | Very Low | Low | `int_str = "-120"` would fail `.parse::<u64>()` since u64 cannot be negative. Returns Err, which is correct — negative prices are nonsensical. |

---

## Resolved Questions

The PRD states "None. The requirements are fully specified and self-contained." No open questions required user input. Implementation proceeds with documented requirements only.

---

## New Technical Questions Discovered

None. The implementation is a straightforward string parsing function and a test assertion fix with no ambiguity.

---

## Deviations from Requirements

None found. The existing code structure matches the PRD expectations exactly:

- `to_fixed_6()` has the expected TODO stub at lines 302-311
- The function signature matches: `fn to_fixed_6(txt: &str) -> Result<u64>`
- `anyhow::Result` is already imported at line 4
- The `anyhow!` macro is already imported at line 4 (available for error construction)
- The test assertion at line 343 has the expected wrong value (`1_123_457` instead of `1_123_456`)
- The test TODO comment on line 341 confirms intentional breakage
- No other files need modification per the PRD constraint ("Only one file is modified: `backend/src/main.rs`")
- The Cargo.toml path dependency points to `../program-task/programs/sol_usd_oracle` — this is the correct path for the backend's dependency on the oracle crate

---

## Files to Modify

| File | Change |
|------|--------|
| `backend/src/main.rs` | Implement `to_fixed_6()` body (lines 303-310): split on `.`, parse integer * 10^6, truncate fractional to 6 chars, right-pad, parse, add |
| `backend/src/main.rs` | Fix test assertion (line 343): change `1_123_457` to `1_123_456` |

No other files are in scope per the PRD constraint.
