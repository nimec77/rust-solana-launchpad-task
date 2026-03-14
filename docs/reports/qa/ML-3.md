# QA Report: ML-3 — Fix Backend

**Ticket:** ML-3
**Phase:** 3 (Iteration 3)
**Date:** 2026-03-14
**Status:** RELEASE

---

## Scope

Two changes in the backend iteration, both in a single file (`backend/src/main.rs`):
1. Implement `to_fixed_6()` — parse a decimal price string (e.g., "120.12") into a fixed-point u64 with 6 decimal places (e.g., 120_120_000), truncating (not rounding) any digits beyond the 6th decimal place.
2. Fix test `to_fixed_6_truncates_fraction_to_six_digits` — change expected value from `1_123_457` (rounded) to `1_123_456` (truncated).

Only one file modified. No API/signature changes, no new dependencies, no configuration changes.

---

## Positive Scenarios

| # | Scenario | Type | Status |
|---|----------|------|--------|
| P1 | Integer-only input: `to_fixed_6("120")` returns `120_000_000` (120 * 10^6, no fractional part). | Automated (unit test 1) | PASS — `to_fixed_6_parses_integer_and_fractional_part` asserts `120_000_000` |
| P2 | Short fractional part: `to_fixed_6("120.12")` returns `120_120_000` (fractional part "12" zero-padded to "120000"). | Automated (unit test 1) | PASS — same test asserts `120_120_000` |
| P3 | Minimum non-zero value: `to_fixed_6("0.000001")` returns `1` (6th decimal place is the smallest representable unit). | Automated (unit test 1) | PASS — same test asserts `1` |
| P4 | Truncation of excess digits: `to_fixed_6("1.1234569")` returns `1_123_456` (7th digit "9" is dropped, not rounded up to `1_123_457`). | Automated (unit test 2) | PASS — `to_fixed_6_truncates_fraction_to_six_digits` asserts `1_123_456` |
| P5 | Invalid input rejection: `to_fixed_6("abc")` returns `Err`. | Automated (unit test 3) | PASS — `to_fixed_6_rejects_invalid_input` asserts `is_err()` |
| P6 | High value price: `to_fixed_6("99999.999999")` returns `99_999_999_999` — fits within u64. | Manual (code review + formula verification) | VERIFIED — `99999 * 1_000_000 + 999_999 = 99_999_999_999`, well within u64 range |
| P7 | Data flow integration: `PriceSource::fetch_price()` calls `to_fixed_6(&resp.price)` at line 111, feeding the result into `submit_price()` which sends an `update_price` instruction to the oracle PDA. | Manual (code review) | VERIFIED — call chain is intact and unchanged |
| P8 | Unrelated tests remain green: 4 non-`to_fixed_6` tests pass without modification. | Automated (unit tests 4-7) | PASS — `parse_token_created_reads_expected_fields`, `parse_token_created_returns_none_for_unrelated_logs`, `price_source_prefers_mock_over_url`, `price_source_uses_default_url_when_no_override` all pass |

---

## Negative and Edge Case Scenarios

| # | Scenario | Type | Status |
|---|----------|------|--------|
| N1 | Non-numeric input: `to_fixed_6("abc")` returns `Err` — `"abc".parse::<u64>()` fails, error propagated via `.map_err()`. | Automated (unit test 3) | PASS |
| N2 | Trailing dot input: `to_fixed_6("120.")` — `split_once('.')` yields `int_str="120"`, `frac_str=""`. Empty fractional part padded to `"000000"`, parses to `0`. Result: `120_000_000`. | Manual (code review) | VERIFIED — `format!("{:0<6}", "")` produces `"000000"`, which parses to `0`. Correct behavior. |
| N3 | Leading-dot input: `to_fixed_6(".5")` — `int_str=""`, which fails `"".parse::<u64>()`, returns `Err`. | Manual (code review) | VERIFIED — acceptable behavior. Binance API always sends a leading zero (e.g., "0.5"). |
| N4 | Multiple dots: `to_fixed_6("1.2.3")` — `split_once('.')` yields `int_str="1"`, `frac_str="2.3"`. `"2.3"` truncated to `"2.3"` (3 chars < 6), padded to `"2.3000"`. `"2.3000".parse::<u64>()` fails (contains "."), returns `Err`. | Manual (code review) | VERIFIED — correctly rejects invalid input via fractional parse failure. |
| N5 | Negative price: `to_fixed_6("-120.5")` — `int_str="-120"`, `"-120".parse::<u64>()` fails (u64 cannot be negative), returns `Err`. | Manual (code review) | VERIFIED — correctly rejects negative values. |
| N6 | Empty input: `to_fixed_6("")` — no dot found, so `int_str=""`. `"".parse::<u64>()` fails, returns `Err`. | Manual (code review) | VERIFIED — correctly rejects empty string. |
| N7 | Overflow in `int_part * 1_000_000`: would require `int_part > 18_446_744_073_709` (>$18 trillion). | Manual (code review) | ACCEPTED — not a realistic SOL/USD price. Unchecked multiplication is adequate. PRD does not require `checked_mul` and no test covers this. |
| N8 | Exactly 6 fractional digits: `to_fixed_6("1.123456")` — `frac_str.len() == 6`, no truncation. `frac_truncated = "123456"`, no padding needed. Result: `1_123_456`. | Manual (code review) | VERIFIED — truncation branch is skipped correctly when `len <= 6`. |
| N9 | Very long fractional part: `to_fixed_6("1.123456789012345")` — truncated to `"123456"` (first 6 chars). Extra 9 digits dropped. Result: `1_123_456`. | Manual (code review) | VERIFIED — `&frac_str[..6]` takes only first 6 bytes. Safe for ASCII digit strings. |

---

## Automated Test Coverage

| Suite | Test | Covers |
|-------|------|--------|
| to_fixed_6 (3 tests) | `to_fixed_6_parses_integer_and_fractional_part` | Integer-only ("120"), short fraction ("120.12"), minimum value ("0.000001") |
| | `to_fixed_6_truncates_fraction_to_six_digits` | 7-digit fractional part truncated to 6 (not rounded): "1.1234569" -> `1_123_456` |
| | `to_fixed_6_rejects_invalid_input` | Non-numeric input ("abc") returns `Err` |
| parse_token_created (2 tests) | `parse_token_created_reads_expected_fields` | Regex parsing of TokenCreated event log (unaffected) |
| | `parse_token_created_returns_none_for_unrelated_logs` | Non-matching log lines return None (unaffected) |
| PriceSource (2 tests) | `price_source_prefers_mock_over_url` | Mock price takes precedence over API URL (unaffected) |
| | `price_source_uses_default_url_when_no_override` | Default Binance URL used when no overrides set (unaffected) |

**Total: 7 automated tests (3 to_fixed_6 + 2 parse_token_created + 2 PriceSource) -- all passing**

---

## Manual Checks

| # | Check | Result |
|---|-------|--------|
| M1 | `to_fixed_6()` function signature unchanged: `fn to_fixed_6(txt: &str) -> Result<u64>` | CONFIRMED — line 306 |
| M2 | No `todo!()` macro or `let _ = ...` suppression remains in function body | CONFIRMED — replaced with `split_once` + parse logic |
| M3 | Only one file modified: `backend/src/main.rs` | CONFIRMED — per PRD constraint |
| M4 | Test expected value corrected from `1_123_457` to `1_123_456` | CONFIRMED — line 358 |
| M5 | No other test assertions modified | CONFIRMED — only the expected value on line 358 was changed |
| M6 | No floating-point arithmetic used anywhere in `to_fixed_6()` | CONFIRMED — string splitting + integer parsing + `format!` padding only |
| M7 | Truncation implemented via string slicing (`&frac_str[..6]`), not numeric rounding | CONFIRMED — lines 316-320 |
| M8 | `anyhow::{anyhow, Result}` import already present at line 4 — no new imports needed | CONFIRMED |
| M9 | `cargo test` completes with all 7 tests passing | CONFIRMED — verified by running `cargo test` |

---

## Risk Zones

| Risk | Assessment | Mitigation |
|------|-----------|------------|
| **u64 overflow in `int_part * 1_000_000`** | VERY LOW — requires integer part > 1.8 * 10^13, not a realistic SOL/USD price. | Accepted by design per PRD. No `checked_mul` needed for this range. |
| **Non-ASCII input causing slice panic** | VERY LOW — `&frac_str[..6]` could panic on non-ASCII multi-byte characters. However, price strings from Binance API contain only ASCII digits and dots. | Acceptable. The function is called only with Binance API output or mock values. Non-numeric input is caught by `.parse::<u64>()` before any slicing in the integer path, and the fractional path would fail at `parse` after slicing. |
| **Test coverage gaps** | LOW — no automated test for trailing dot ("120."), empty string (""), or high values ("99999.999999"). | Mitigated by manual code review above (N2, N6, P6). Existing 3 tests cover the primary scenarios from the PRD. |
| **Regression in unrelated tests** | NONE — 4 non-`to_fixed_6` tests are unaffected. No shared state between test functions. | No risk. |
| **Integration with price updater** | LOW — `to_fixed_6` is called only by `PriceSource::fetch_price()`. The call site and data flow are unchanged. The function now returns correct values instead of panicking via `todo!()`. | No risk introduced; this is strictly an improvement from "always panics" to "correctly parses." |
| **Decimal format from Binance API** | VERY LOW — Binance API always uses `.` as decimal separator and never sends locale-specific formats (e.g., comma). | No mitigation needed. |

---

## Verdict

**RELEASE**

All acceptance criteria from the PRD, plan, and tasklist are met:
- `to_fixed_6()` correctly parses decimal strings into fixed-point u64 with 6 decimal places using string splitting and integer arithmetic only (no floating-point)
- Extra fractional digits beyond the 6th are truncated (dropped), never rounded
- Test assertion corrected from `1_123_457` to `1_123_456`
- All 7 backend unit tests pass (`cargo test` confirmed)
- Only one file modified (`backend/src/main.rs`) per PRD constraint
- Function signature unchanged: `fn to_fixed_6(txt: &str) -> Result<u64>`
- No regressions in the 4 unrelated tests (parse_token_created, PriceSource)
- Edge cases (trailing dot, empty input, negative input, multiple dots) handled correctly via code review

No reservations. The implementation is minimal, correct, and well-tested.
