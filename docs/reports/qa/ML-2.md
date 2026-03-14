# QA Report: ML-2 — Fix Minter Program

**Ticket:** ML-2
**Phase:** 2 (Iteration 2)
**Date:** 2026-03-14
**Status:** RELEASE

---

## Scope

Two changes in the token minter iteration:
1. Implement `compute_fee_lamports()` in `program/programs/token_minter/src/lib.rs` — convert USD-denominated mint fee to lamports using oracle SOL/USD price.
2. Fix inverted fee formula assertion in `program/tests/minter.litesvm.ts` — swap operands so `expectedFee = FEE_USD * LAMPORTS_PER_SOL / PRICE`.

Only two files modified. No API/signature changes, no new accounts, no new error codes.

---

## Positive Scenarios

| # | Scenario | Type | Status |
|---|----------|------|--------|
| P1 | Successful token mint with correct fee: inputs `mint_fee_usd=5_000_000`, `price=120_000_000` produce `fee_lamports=41_666_666`. Treasury balance increases by that amount. | Automated (LiteSVM test 1) | PASS — implementation uses u128 intermediate with `checked_mul`, test assertion uses matching formula `FEE_USD * LAMPORTS_PER_SOL / PRICE` |
| P2 | Mint account is created and tokens are minted to user's ATA after successful `mint_token` call. | Automated (LiteSVM test 1) | PASS — test verifies `mintAcct` and `ataAcct` are non-null |
| P3 | MinterConfig state is persisted correctly after initialization (fee, treasury). | Automated (LiteSVM test 1) | PASS — test decodes config PDA and asserts `mint_fee_usd` and `treasury` |
| P4 | Oracle price is read correctly by minter via CPI account validation. | Automated (LiteSVM test 1) | PASS — oracle state decoded and verified `price == 120_000_000` |
| P5 | Fee decreases when SOL price increases ($5 at $120 = ~0.0417 SOL; $5 at $240 = ~0.0208 SOL). | Manual | VERIFIED by formula inspection: `fee = fee_usd * LAMPORTS / price`, so higher `price` yields lower `fee` |
| P6 | Oracle tests remain green (4/4) — no regression from minter changes. | Automated (LiteSVM oracle suite) | PASS — oracle files untouched |

---

## Negative and Edge Case Scenarios

| # | Scenario | Type | Status |
|---|----------|------|--------|
| N1 | Reject mint when `initial_supply = 0` — transaction fails with `InvalidSupply`, no fee transferred to treasury. | Automated (LiteSVM test 2) | PASS — `assertFailure` confirms tx rejection, treasury balance unchanged |
| N2 | Reject mint when `decimals = 10` (exceeds max 9) — transaction fails with `InvalidDecimals`, no fee transferred. | Automated (LiteSVM test 3) | PASS — `assertFailure` confirms tx rejection, treasury balance unchanged |
| N3 | Zero oracle price rejected — `require!(price > 0, MinterError::OraclePriceZero)` guard at line 167 prevents division by zero. | Manual (code review) | VERIFIED — guard is present and unchanged. Additionally, the `mint_token` handler has a redundant guard at line 67 (`require!(oracle_state.price > 0)`). Both reject zero price before division is reached. |
| N4 | Overflow protection — `checked_mul` on u128 prevents silent overflow for `mint_fee_usd * LAMPORTS_PER_SOL`. Returns `MinterError::MathOverflow` on failure. | Manual (code review) | VERIFIED — u128 `checked_mul` is used. For u64 inputs max product is ~3.4 * 10^38, well within u128 max (~3.4 * 10^38). Practically impossible to overflow, but the guard is defensive and correct. |
| N5 | Integer truncation to zero fee — occurs only when `mint_fee_usd * LAMPORTS_PER_SOL < price`, meaning fee < 1 lamport. Economically negligible. | Manual (code review) | ACCEPTED — by design. Would require `mint_fee_usd < price / LAMPORTS_PER_SOL`, i.e., fee < `price / 10^9` micro-dollars. For $120 SOL price this means fee < $0.00012, which is outside any realistic configuration. |
| N6 | Oracle decimals mismatch — `require!(oracle_state.decimals == PRICE_DECIMALS)` at line 69 rejects oracle states that don't use 6 decimals. | Manual (code review) | VERIFIED — guard present and correct |
| N7 | Non-admin cannot call `update_price` on oracle — attacker's tx is rejected, price unchanged. | Automated (LiteSVM oracle test 3) | PASS |
| N8 | Zero price update rejected by oracle — `require!(new_price > 0, OracleError::InvalidPrice)`. | Automated (LiteSVM oracle test 4) | PASS |

---

## Automated Test Coverage

| Suite | Test | Covers |
|-------|------|--------|
| Oracle (4 tests) | `initialize_oracle sets admin and defaults` | Init, admin assignment, decimals=6, price=0 |
| | `update_price updates price only for admin` | Happy path price update |
| | `rejects update_price from non-admin signer` | Authorization check |
| | `rejects zero price update` | Zero price validation |
| Minter (3 tests) | `initialize oracle + minter and mint token with fee` | Full flow: init oracle, set price, init minter, mint token, verify fee=41_666_666, verify mint+ATA created, verify config state |
| | `rejects mint when initial supply is zero` | Supply validation, no fee leakage |
| | `rejects mint when decimals exceed allowed range` | Decimals validation, no fee leakage |

**Total: 7 automated tests (4 oracle + 3 minter)**

---

## Manual Checks

| # | Check | Result |
|---|-------|--------|
| M1 | `compute_fee_lamports()` function signature unchanged: `fn compute_fee_lamports(mint_fee_usd: u64, price: u64) -> Result<u64>` | CONFIRMED — line 166 |
| M2 | No `todo!()` macro or `let _ = ...` suppression remains in function body | CONFIRMED — replaced with u128 arithmetic |
| M3 | Only two files modified: `token_minter/src/lib.rs` and `minter.litesvm.ts` | CONFIRMED — git status clean, tasklist shows only these two files |
| M4 | Test formula corrected from `PRICE * LAMPORTS / FEE_USD` to `FEE_USD * LAMPORTS / PRICE` | CONFIRMED — line 172 of `minter.litesvm.ts` |
| M5 | No other test assertions modified | CONFIRMED — only line 172 changed |
| M6 | TODO comment on line 170 of test retained as documentation | CONFIRMED — comment still present |
| M7 | Constants `LAMPORTS_PER_SOL_U64` (line 14) and `MinterError::MathOverflow` (line 289) exist and are used correctly | CONFIRMED |
| M8 | `PRICE_DECIMALS` imported from `sol_usd_oracle` matches expected value of 6 | CONFIRMED — `sol_usd_oracle/src/lib.rs` line 3: `pub const PRICE_DECIMALS: u8 = 6` |

---

## Risk Zones

| Risk | Assessment | Mitigation |
|------|-----------|------------|
| **u64 overflow in fee multiplication** | LOW — u128 intermediate handles all u64 input combinations. `checked_mul` provides additional safety. | Properly mitigated by implementation. |
| **Division by zero** | NONE — double-guarded: `mint_token` handler checks `oracle_state.price > 0` at line 67, and `compute_fee_lamports` checks again at line 167. | Fully mitigated. |
| **Test formula mismatch with implementation** | NONE — both use `fee_usd * LAMPORTS_PER_SOL / price`. Verified by code review and passing test. | Formula identity confirmed in both locations. |
| **Oracle regression** | NONE — no oracle files modified. Oracle test suite (4 tests) remains independent and passing. | No risk. |
| **Integer truncation** | NEGLIGIBLE — only occurs for fees below 1 lamport. No realistic configuration triggers this. | Accepted by design per PRD. |
| **Shared LiteSVM state between minter tests** | LOW — tests 2 and 3 depend on minter config being initialized in test 1. Test order is deterministic in Mocha `describe` blocks. | Acceptable; test isolation is adequate for the test suite's scope. |

---

## Verdict

**RELEASE**

All acceptance criteria from the PRD, plan, and tasklist are met:
- `compute_fee_lamports()` correctly implements `fee_lamports = mint_fee_usd * LAMPORTS_PER_SOL / price` with u128 overflow protection
- Test assertion uses the correct (non-inverted) fee formula
- All 7 LiteSVM tests pass (4 oracle + 3 minter)
- Only the two specified files were modified
- No function signatures, account structures, or error codes were changed
- Overflow and division-by-zero protections are in place
- No regressions in the oracle test suite

No reservations. The implementation is minimal, correct, and well-tested.
