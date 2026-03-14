# PRD: ML-1 — Fix Oracle Program

**Status:** PRD_READY
**Ticket:** ML-1
**Phase:** 1 (Iteration 1)

---

## Context / Idea

**Arguments:** ML-1 docs/phase/phase-1.md

This is the first iteration of the Solana Mini-Launchpad project. The oracle program (`sol_usd_oracle`) has a TODO stub in its core logic and one of its tests has an intentionally wrong assertion. Both must be fixed before any downstream work (minter, backend, deployment) can proceed.

### Source: docs/phase/phase-1.md

> **Goal:** Implement the oracle price update logic and fix the broken test assertion.
>
> **Tasks:**
> - Implement `apply_price_update()` in `program/programs/sol_usd_oracle/src/lib.rs`
>   - Set `oracle.price = new_price` and `oracle.last_updated_slot = current_slot`
> - Fix decimals assertion in `program/tests/oracle.litesvm.ts` (`8` -> `6`)
>
> **Acceptance Criteria:**
> `anchor build && anchor test --skip-build` — oracle tests pass
>
> **Dependencies:** None (first iteration)
>
> **Implementation Notes:**
> - Oracle price uses 6 decimal places (not 8)
> - `apply_price_update()` is a TODO stub that students must fill in
> - The test file has an intentional wrong assertion (`decimals == 8` should be `decimals == 6`)

---

## Goals

1. **Implement oracle price update logic** — The `apply_price_update()` function in `program/programs/sol_usd_oracle/src/lib.rs` must persist the new price and the current slot to the oracle state account.
2. **Fix broken test assertion** — The test in `program/tests/oracle.litesvm.ts` asserts `decimals == 8` but the oracle uses 6 decimal places (`PRICE_DECIMALS = 6`). The assertion must be corrected to `6`.
3. **Establish a passing test baseline** — All oracle LiteSVM tests must pass so that subsequent iterations (minter, backend) can build on a working oracle.

---

## User Stories

1. **As a developer**, I want `apply_price_update()` to correctly store the new price and slot so that the oracle state reflects the latest SOL/USD price after an admin calls `update_price`.
2. **As a developer**, I want the oracle test suite to pass with correct assertions so that I can trust the tests as a regression safety net for future changes.
3. **As a downstream consumer (token_minter)**, I need the oracle to return a valid, non-zero price so that the fee calculation in `compute_fee_lamports()` works correctly in later iterations.

---

## Scenarios

### Scenario 1: Admin updates oracle price
- **Given** the oracle is initialized with `price = 0` and an admin keypair
- **When** the admin calls `update_price` with `new_price = 123_000_000` (i.e., $123.000000)
- **Then** the oracle account's `price` field equals `123_000_000` and `last_updated_slot` equals the current slot

### Scenario 2: Non-admin attempts price update
- **Given** the oracle is initialized with a specific admin
- **When** a non-admin signer calls `update_price`
- **Then** the transaction fails and the price remains unchanged

### Scenario 3: Zero price rejected
- **Given** the oracle is initialized
- **When** the admin calls `update_price` with `new_price = 0`
- **Then** the transaction fails with `OracleError::InvalidPrice`

### Scenario 4: Initialization sets correct decimals
- **Given** the oracle program defines `PRICE_DECIMALS = 6`
- **When** `initialize_oracle` is called
- **Then** the oracle account's `decimals` field equals `6` (not `8`)

---

## Metrics

| Metric | Target |
|--------|--------|
| Oracle LiteSVM tests passing | 4/4 (all tests green) |
| `anchor build` succeeds | Yes |
| `anchor test --skip-build` succeeds | Yes (oracle suite) |

---

## Constraints

1. **Minimal change scope** — Only two files are modified: `program/programs/sol_usd_oracle/src/lib.rs` and `program/tests/oracle.litesvm.ts`. No other files should be touched.
2. **No architectural changes** — The `apply_price_update()` function signature and the program's account structures must remain unchanged.
3. **6 decimal places** — The oracle price uses 6 decimal places, defined by `PRICE_DECIMALS: u8 = 6`. This is a fixed constant and must not be changed.
4. **Anchor version 0.32.1** — Must build with the project's pinned Anchor version.
5. **Rust toolchain 1.89.0** — Must compile with the project's pinned Rust version (defined in `program/rust-toolchain.toml`).

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `apply_price_update()` implementation introduces overflow | Low | High | Both `new_price` and `current_slot` are `u64` values assigned directly — no arithmetic needed, so overflow is not a concern for this task |
| Test fix breaks other tests | Low | Medium | The decimals assertion is isolated to the `initialize_oracle` test; changing `8` to `6` does not affect other test cases |
| Build failure due to toolchain mismatch | Low | Medium | Ensure `program/rust-toolchain.toml` specifies Rust 1.89.0 and Anchor 0.32.1 is installed |

---

## Open Questions

None. The requirements are fully specified and self-contained. Implementation can proceed.

---

## Implementation Reference

### File 1: `program/programs/sol_usd_oracle/src/lib.rs`

**Location:** `apply_price_update()` function (line 10-16)

**Current code (TODO stub):**
```rust
fn apply_price_update(oracle: &mut OracleState, new_price: u64, current_slot: u64) -> Result<()> {
    // TODO(student): finish the happy-path state update.
    let _ = (oracle, new_price, current_slot);
    todo!("student task: persist the new price and slot");
}
```

**Required:** Set `oracle.price = new_price` and `oracle.last_updated_slot = current_slot`, then return `Ok(())`.

### File 2: `program/tests/oracle.litesvm.ts`

**Location:** `initialize_oracle sets admin and defaults` test (line 89)

**Current code (wrong assertion):**
```typescript
expect(decoded.decimals).to.eq(8);
```

**Required:** Change `8` to `6` to match `PRICE_DECIMALS = 6`.
