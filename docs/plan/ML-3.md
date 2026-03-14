# Plan: ML-3 — Fix Backend

**Ticket:** ML-3
**Phase:** 3 (Iteration 3)
**Status:** PLAN_APPROVED

---

## Components

### 1. Backend — `to_fixed_6()` Implementation

**File:** `backend/src/main.rs` (lines 302-311)

The `to_fixed_6()` function is called by `PriceSource::fetch_price()` (line 110) to convert a decimal price string from the Binance API into a fixed-point u64 with 6 decimal places. The result feeds into `submit_price()` which sends an `update_price` instruction to the oracle PDA.

**Current state:** TODO stub with `todo!()` macro that panics at runtime.

**Target state:** Replace the TODO stub with string-based decimal parsing:
1. Use `split_once('.')` to separate integer and fractional parts
2. Parse integer part as u64, multiply by 1_000_000
3. Truncate fractional part to at most 6 characters (drop extras, do not round)
4. Right-pad fractional part with zeros to exactly 6 characters
5. Parse padded fractional part as u64 and add to scaled integer part
6. Return `Err` for non-numeric input via `?` on `.parse::<u64>()`

**Function signature (unchanged):** `fn to_fixed_6(txt: &str) -> Result<u64>` (uses `anyhow::Result`)

**Dependencies already available:**
- `anyhow::{anyhow, Result}` imported at line 4
- No new imports required

### 2. Backend Test — Assertion Fix

**File:** `backend/src/main.rs` (lines 340-344)

The test `to_fixed_6_truncates_fraction_to_six_digits` asserts that `to_fixed_6("1.1234569")` returns `1_123_457`. This expected value is wrong -- it represents a rounded result. The input "1.1234569" has 7 fractional digits; truncation at 6 digits yields fractional part "123456" (the 7th digit "9" is dropped), so the correct result is `1_123_456`.

**Target state:** Change the expected value from `1_123_457` to `1_123_456`.

---

## API Contract

No API changes. The function `to_fixed_6()` is a private module-level function. Its signature remains `fn to_fixed_6(txt: &str) -> Result<u64>`. No public interfaces, no instruction formats, no account structures are affected.

### Function: `to_fixed_6(txt: &str) -> Result<u64>`

| Aspect | Value |
|--------|-------|
| Visibility | Private (module-level `fn`) |
| Input | `txt: &str` -- decimal string (e.g., "120.12", "0.000001") |
| Output | `Ok(u64)` -- fixed-point value with 6 decimal places |
| Error | `Err(anyhow::Error)` for non-numeric input |
| Truncation | Fractional digits beyond the 6th are dropped, never rounded |
| No-dot input | Treated as integer-only; fractional part is 0 |
| Empty fractional part | "120." treated as "120.000000", result `120_000_000` |

---

## Data Flows

```
Binance API (or MOCK_PRICE)
    |
    | JSON response: { "price": "120.12345678" }
    v
PriceSource::fetch_price()
    |
    | resp.price (String)
    v
to_fixed_6("120.12345678")
    |
    |- split_once('.') -> ("120", "12345678")
    |- integer: 120 * 1_000_000 = 120_000_000
    |- fractional: "123456" (truncated to 6 chars)
    |- padded: "123456" (already 6 chars)
    |- parsed: 123_456
    |- total: 120_000_000 + 123_456 = 120_123_456
    v
submit_price(client, cfg, 120_123_456, admin)
    |
    v
Oracle PDA (price field: u64, 6 decimals)
    |
    v
token_minter reads Oracle PDA -> compute_fee_lamports()
```

**Example calculations from PRD scenarios:**

| Input | Integer Part | Fractional Part | Truncated | Padded | Result |
|-------|-------------|-----------------|-----------|--------|--------|
| "120" | 120 * 10^6 = 120_000_000 | (none) | "" | "000000" -> 0 | 120_000_000 |
| "120.12" | 120 * 10^6 = 120_000_000 | "12" | "12" | "120000" -> 120_000 | 120_120_000 |
| "0.000001" | 0 * 10^6 = 0 | "000001" | "000001" | "000001" -> 1 | 1 |
| "1.1234569" | 1 * 10^6 = 1_000_000 | "1234569" | "123456" | "123456" -> 123_456 | 1_123_456 |
| "99999.999999" | 99999 * 10^6 = 99_999_000_000 | "999999" | "999999" | "999999" -> 999_999 | 99_999_999_999 |
| "abc" | parse fails | -- | -- | -- | Err |

---

## NFR (Non-Functional Requirements)

| Requirement | How Met |
|-------------|---------|
| No floating-point arithmetic | String splitting + integer parsing only; no `f64` anywhere in data path |
| Truncation, not rounding | String slicing at 6 characters drops extra digits before any numeric operation |
| Consistent 6-decimal scaling | Output matches oracle's `PRICE_DECIMALS = 6` and minter's fee scaling |
| Error propagation | `?` operator on `.parse::<u64>()` converts `ParseIntError` to `anyhow::Error` |
| Minimal change scope | Only 1 file modified (`backend/src/main.rs`) per PRD constraint |
| Rust toolchain compatibility | No new language features; compiles with pinned Rust 1.89.0 |
| All 7 tests pass | 3 `to_fixed_6` tests (now correct) + 4 unrelated tests (unchanged) |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Overflow in `int_part * 1_000_000` | Very Low | High | u64 max is ~1.8 * 10^19; integer part would need to exceed ~1.8 * 10^13, not a realistic SOL/USD price |
| Input "120." (trailing dot) | Low | None | `split_once` yields `frac_str = ""`, padded to `"000000"`, parses to `0`. Correct result. |
| Input ".5" (no integer part) | Very Low | Low | `int_str = ""` fails `.parse::<u64>()`, returns Err. Acceptable -- Binance always sends leading zero ("0.5"). |
| Multiple dots ("1.2.3") | Very Low | Low | `split_once` takes first dot only; `frac_str = "2.3"` fails `.parse::<u64>()`, returns Err. Correct for invalid input. |
| Negative prices ("-120.5") | Very Low | Low | `int_str = "-120"` fails `.parse::<u64>()` (u64 cannot be negative), returns Err. Correct behavior. |

---

## Deviations to Fix

None. The research document confirmed zero deviations between the existing code and the PRD requirements:

- `to_fixed_6()` has the expected TODO stub at lines 302-311
- The function signature matches: `fn to_fixed_6(txt: &str) -> Result<u64>`
- `anyhow::{anyhow, Result}` is already imported at line 4
- The test assertion at line 343 has the expected wrong value (`1_123_457` instead of `1_123_456`)
- The TODO comment at line 341 confirms intentional breakage
- No other files need modification per the PRD constraint ("Only one file is modified: `backend/src/main.rs`")

---

## Implementation Steps

### Step 1: Implement `to_fixed_6()` body

**File:** `backend/src/main.rs`

Replace lines 302-311 (the entire function body, preserving the function signature):

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

**Key design choices:**
- `split_once('.')` cleanly handles no-dot, single-dot, and trailing-dot cases
- String truncation (`&frac_str[..6]`) enforces truncation before any numeric conversion -- no possibility of rounding
- `format!("{:0<6}", ...)` right-pads the fractional string with zeros to exactly 6 characters
- Two separate `.parse()` calls with `.map_err()` provide clear error messages distinguishing invalid integer vs fractional parts
- Empty `frac_str` (`""`) gets padded to `"000000"`, parsing to `0` -- correct for integer-only input

### Step 2: Fix test expected value

**File:** `backend/src/main.rs`

At line 343, change:

```rust
assert_eq!(to_fixed_6("1.1234569").unwrap(), 1_123_457);
```

To:

```rust
assert_eq!(to_fixed_6("1.1234569").unwrap(), 1_123_456);
```

### Step 3: Verify

Run: `cd backend && cargo test`

Expected result: all 7 tests pass:

| # | Test Name | Status |
|---|-----------|--------|
| 1 | `to_fixed_6_parses_integer_and_fractional_part` | Passes |
| 2 | `to_fixed_6_truncates_fraction_to_six_digits` | Passes |
| 3 | `to_fixed_6_rejects_invalid_input` | Passes |
| 4 | `parse_token_created_reads_expected_fields` | Passes (unaffected) |
| 5 | `parse_token_created_returns_none_for_unrelated_logs` | Passes (unaffected) |
| 6 | `price_source_prefers_mock_over_url` | Passes (unaffected) |
| 7 | `price_source_uses_default_url_when_no_override` | Passes (unaffected) |

---

## Open Questions

None. Requirements are fully specified per the PRD. No architectural alternatives to evaluate (no ADR needed). The implementation is a straightforward string parsing function and a test assertion fix with no ambiguity.
