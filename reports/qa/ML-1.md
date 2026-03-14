# QA Report: ML-1 — Fix Oracle Program

**Ticket:** ML-1
**Phase:** 1 (Iteration 1)
**Date:** 2026-03-14
**Tasklist Status:** IMPLEMENT_STEP_OK

---

## Scope

Two changes were required:

1. **Implement `apply_price_update()`** in `program/programs/sol_usd_oracle/src/lib.rs` — replace the `todo!()` stub with actual state mutation (`oracle.price = new_price`, `oracle.last_updated_slot = current_slot`).
2. **Fix decimals assertion** in `program/tests/oracle.litesvm.ts` — change `expect(decoded.decimals).to.eq(8)` to `.to.eq(6)` to match `PRICE_DECIMALS = 6`.

---

## Implementation Verification

### Task 1: `apply_price_update()` — PASS

- The `todo!()` macro and `let _ = (oracle, new_price, current_slot)` suppression have been removed.
- The function body now contains exactly two assignments and a return:
  - `oracle.price = new_price`
  - `oracle.last_updated_slot = current_slot`
  - `Ok(())`
- The function signature is unchanged: `fn apply_price_update(oracle: &mut OracleState, new_price: u64, current_slot: u64) -> Result<()>`.
- No arithmetic operations are performed; both assignments are direct `u64` field writes, eliminating any overflow risk.

### Task 2: Decimals Assertion Fix — PASS

- Line 89 of `program/tests/oracle.litesvm.ts` now reads `expect(decoded.decimals).to.eq(6)`.
- The TODO comment on line 87-88 has been preserved for student reference.
- No other test assertions were modified.

### Task 3: Build & Test Verification — PENDING (manual step)

- `anchor build` and `anchor test --skip-build` (or `make test`) must be run to confirm all 4 oracle tests pass.

---

## Positive Scenarios

| # | Scenario | Expected Result | Verification Method |
|---|----------|----------------|---------------------|
| P1 | Admin calls `update_price` with `new_price = 123_000_000` | `oracle.price == 123_000_000`, `oracle.last_updated_slot == current_slot` | Automated: test "update_price updates price only for admin" |
| P2 | Oracle initialized with `PRICE_DECIMALS = 6` | `oracle.decimals == 6` | Automated: test "initialize_oracle sets admin and defaults" |
| P3 | Oracle initialized with `price = 0` | `oracle.price == 0` | Automated: test "initialize_oracle sets admin and defaults" |
| P4 | Admin key matches `oracle.admin` after initialization | `oracle.admin == payer.publicKey` | Automated: test "initialize_oracle sets admin and defaults" |
| P5 | Sequential price updates overwrite previous price | After second `update_price`, oracle reflects the latest value | Automated: can be inferred from test sequence (init -> update -> attack -> zero) |

---

## Negative and Edge Cases

| # | Scenario | Expected Result | Verification Method |
|---|----------|----------------|---------------------|
| N1 | Non-admin signer calls `update_price` | Transaction fails; price unchanged (remains `123_000_000`) | Automated: test "rejects update_price from non-admin signer" |
| N2 | Admin calls `update_price` with `new_price = 0` | Transaction fails with `OracleError::InvalidPrice`; price unchanged | Automated: test "rejects zero price update" |
| N3 | `update_price` with `u64::MAX` (maximum possible price) | Should succeed — direct assignment, no arithmetic | Not covered by tests (edge case) |
| N4 | Multiple rapid `update_price` calls | Each call overwrites previous; last write wins | Not explicitly tested but implied by sequential test execution |
| N5 | Calling `initialize_oracle` a second time | Should fail — PDA already initialized (Anchor `init` constraint) | Not explicitly tested |

---

## Division: Automated Tests vs. Manual Checks

### Automated Tests (LiteSVM — 4 tests)

1. **`initialize_oracle sets admin and defaults`** — Verifies admin, price=0, decimals=6 after initialization.
2. **`update_price updates price only for admin`** — Verifies price persistence after admin call with `new_price = 123_000_000`.
3. **`rejects update_price from non-admin signer`** — Verifies unauthorized access is blocked and price is unchanged.
4. **`rejects zero price update`** — Verifies zero price is rejected and price is unchanged.

**Run command:** `anchor build && anchor test --skip-build` or `make test`

### Manual Checks

| # | Check | Status |
|---|-------|--------|
| M1 | `apply_price_update()` contains no `todo!()` macro | PASS — verified by code review |
| M2 | `apply_price_update()` contains no `let _ = ...` suppression | PASS — verified by code review |
| M3 | Function signature unchanged | PASS — verified by code review |
| M4 | Only two files modified per PRD constraint | PASS — tasklist confirms only `lib.rs` and `oracle.litesvm.ts` were changed |
| M5 | `PRICE_DECIMALS` constant remains `6` | PASS — line 3 of `lib.rs` reads `pub const PRICE_DECIMALS: u8 = 6` |
| M6 | No changes to `OracleState` struct | PASS — `state.rs` unmodified (admin, price, decimals, last_updated_slot, bump) |
| M7 | No changes to error codes (`OracleError`) | PASS — `Unauthorized` and `InvalidPrice` remain as defined |
| M8 | No changes to account constraints (`InitializeOracle`, `UpdatePrice`) | PASS — verified by code review |

---

## Risk Zones

| Risk | Severity | Assessment |
|------|----------|------------|
| **Overflow in `apply_price_update()`** | N/A | No arithmetic performed. Both `new_price` and `current_slot` are direct `u64` assignments. No risk. |
| **Test assertion fix cascading to other tests** | Low | The decimals assertion is isolated to test 1. Tests 2-4 do not check `decimals`. Verified: no other assertions were changed. |
| **`last_updated_slot` correctness** | Low | The slot is obtained via `Clock::get()?.slot` in the `update_price` handler and passed to `apply_price_update()`. This is the standard Anchor pattern. The test does not assert on `last_updated_slot` value directly, but a non-crash result confirms the field is writable. |
| **Downstream impact on `token_minter`** | None for this ticket | The minter reads `oracle.price` via CPI account validation. A working oracle is a prerequisite for minter (ML-2), not a breaking change. |
| **Build toolchain mismatch** | Low | Rust 1.89.0 and Anchor 0.32.1 are pinned in `program/rust-toolchain.toml`. Build verification (`make test`) will catch any mismatch. |

---

## Test Coverage Gaps

| Gap | Severity | Recommendation |
|-----|----------|----------------|
| No test asserts `last_updated_slot` after `update_price` | Low | The slot value is difficult to predict in LiteSVM but could be asserted as `> 0`. Consider adding in a future iteration. |
| No test for `u64::MAX` price value | Low | Edge case. Direct assignment makes this safe, but an explicit test would improve confidence. |
| No test for double initialization | Low | Anchor's `init` constraint should prevent this, but an explicit test would document the behavior. |
| No test verifies price unchanged after failed non-admin update (value assertion) | Covered | Test 3 does check `decoded.price.toString() == "123000000"` after the rejected attack. |

---

## Final Verdict

### RELEASE (with minor reservations)

**Rationale:**

- Both code changes match the PRD and plan specifications exactly.
- The implementation is minimal and correct: two direct field assignments with no arithmetic, no new dependencies, no signature changes.
- All 4 existing automated tests cover the core scenarios (initialization, admin update, unauthorized rejection, zero price rejection).
- Manual code review confirms no regressions in account structures, error codes, or constraints.
- The only gap is the absence of `last_updated_slot` value assertion in tests, which is a minor observation rather than a blocker.

**Reservations:**

1. Build and test execution (`anchor build && anchor test --skip-build`) must be confirmed to pass before final release. The tasklist status is IMPLEMENT_STEP_OK, meaning implementation is complete but runtime verification should be confirmed.
2. Test coverage for `last_updated_slot` assertion is absent but non-critical for this iteration.

**Recommendation:** Proceed with release after confirming `make test` passes successfully.
