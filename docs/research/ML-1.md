# Research: ML-1 — Fix Oracle Program

**Ticket:** ML-1
**Phase:** 1 (Iteration 1)
**Date:** 2026-03-14

---

## Summary

Two targeted fixes in the `sol_usd_oracle` program: implement the `apply_price_update()` function body and correct a wrong test assertion. This is the first iteration and has no dependencies.

---

## Existing Code Analysis

### File 1: `program/programs/sol_usd_oracle/src/lib.rs`

**Location:** Lines 10-16 — `apply_price_update()` function

The function currently contains a TODO stub:

```rust
fn apply_price_update(oracle: &mut OracleState, new_price: u64, current_slot: u64) -> Result<()> {
    // TODO(student): finish the happy-path state update.
    let _ = (oracle, new_price, current_slot);
    todo!("student task: persist the new price and slot");
}
```

**Caller context (lines 32-40):** The `update_price` instruction handler validates:
1. `new_price > 0` (returns `OracleError::InvalidPrice` if zero)
2. `ctx.accounts.admin.key() == oracle.admin` (returns `OracleError::Unauthorized` if mismatch)
3. Gets `current_slot` from `Clock::get()?.slot`
4. Calls `apply_price_update(oracle, new_price, current_slot)`

The function signature is `-> Result<()>`, so it must return `Ok(())` on success. The `has_one = admin` constraint on the `UpdatePrice` accounts struct provides an additional admin check at the Anchor constraint level.

**Required implementation:**
- Set `oracle.price = new_price`
- Set `oracle.last_updated_slot = current_slot`
- Return `Ok(())`

No arithmetic is needed — both values are direct `u64` assignments. No overflow risk.

### File 2: `program/programs/sol_usd_oracle/src/state.rs`

The `OracleState` account struct:

```rust
#[account]
pub struct OracleState {
    pub admin: Pubkey,        // 32 bytes
    pub price: u64,           // 8 bytes
    pub decimals: u8,         // 1 byte
    pub last_updated_slot: u64, // 8 bytes
    pub bump: u8,             // 1 byte
}
```

- `SEED = b"oracle_state"` — singleton PDA
- `SIZE = 50` bytes (32 + 8 + 1 + 8 + 1)
- Both `price` and `last_updated_slot` are public `u64` fields — directly writable via `&mut` reference

### File 3: `program/tests/oracle.litesvm.ts`

**Location:** Line 89 — wrong assertion in "initialize_oracle sets admin and defaults" test

```typescript
expect(decoded.decimals).to.eq(8);
```

The oracle program defines `PRICE_DECIMALS: u8 = 6` (lib.rs line 3) and sets `oracle.decimals = PRICE_DECIMALS` during initialization (lib.rs line 26). The assertion must be `6`, not `8`.

**Test suite structure (4 tests):**

| # | Test | Status | Notes |
|---|------|--------|-------|
| 1 | `initialize_oracle sets admin and defaults` | Fails | Wrong decimals assertion (`8` should be `6`) |
| 2 | `update_price updates price only for admin` | Fails | Calls `apply_price_update()` which hits `todo!()` panic |
| 3 | `rejects update_price from non-admin signer` | Fails | Same — `apply_price_update()` may be reached depending on constraint order |
| 4 | `rejects zero price update` | May pass | Zero price is rejected before `apply_price_update()` is called |

Tests run sequentially and share state. Test 1 initializes the oracle. Tests 2-4 depend on the initialized state from test 1.

---

## Layers and Dependencies

### Build Dependencies
- **Rust toolchain:** 1.89.0 (pinned in `program/rust-toolchain.toml`)
- **Anchor:** 0.32.1 (pinned in `Cargo.toml` dependency)
- **anchor-lang:** 0.32.1 (sole dependency of `sol-usd-oracle` crate)

### Program Dependency Graph
```
sol_usd_oracle (independent, no external program deps)
    ^
    |  account read (CPI validation)
    |
token_minter (depends on sol_usd_oracle via `use sol_usd_oracle::{state::OracleState, PRICE_DECIMALS}`)
```

The `token_minter` imports `PRICE_DECIMALS` from the oracle crate and reads `OracleState` accounts. It validates `oracle_state.decimals == PRICE_DECIMALS` (line 69 of token_minter/lib.rs). The oracle's `PRICE_DECIMALS = 6` constant is authoritative for both programs.

### Test Dependencies
- LiteSVM (in-process SVM, no validator)
- `@coral-xyz/anchor` for Borsh encoding/decoding
- `ts-mocha` + `chai` for test runner and assertions
- Pre-built `.so` file at `target/deploy/sol_usd_oracle.so` (tests use `--skip-build`)

---

## Patterns Used

1. **Anchor PDA accounts** — `OracleState` is a singleton PDA derived from `[b"oracle_state"]` with stored bump
2. **Admin-gated mutations** — `has_one = admin` Anchor constraint + explicit `require_keys_eq!` check in handler
3. **Separation of validation and mutation** — `update_price` handler validates, then delegates to `apply_price_update()` for state changes
4. **Fixed-point arithmetic** — All prices use `u64` with 6 decimal places (e.g., `123_000_000` = $123.00)
5. **LiteSVM testing** — Tests build transactions manually (no Anchor client), load `.so` binary directly, run in-process without a validator

---

## Implementation Plan

### Change 1: `program/programs/sol_usd_oracle/src/lib.rs`

Replace the TODO stub body of `apply_price_update()` (lines 11-15) with:

```rust
oracle.price = new_price;
oracle.last_updated_slot = current_slot;
Ok(())
```

Remove the `let _ = ...` and `todo!()` lines. The function signature and visibility remain unchanged.

### Change 2: `program/tests/oracle.litesvm.ts`

At line 89, change:
```typescript
expect(decoded.decimals).to.eq(8);
```
to:
```typescript
expect(decoded.decimals).to.eq(6);
```

### Verification

Run: `anchor build && anchor test --skip-build` (or `make test`)

Expected result: all 4 oracle tests pass.

---

## Limitations and Risks

| Risk | Assessment | Mitigation |
|------|-----------|------------|
| Implementation too trivial to get wrong | Very low risk | Two field assignments and `Ok(())` — no logic to misconstruct |
| Test fix breaks other tests | No risk | The decimals assertion is isolated to test 1; tests 2-4 do not check `decimals` |
| `anchor build` fails due to toolchain | Low risk | Toolchain is pinned at 1.89.0, Anchor at 0.32.1; should build if toolchain is installed |
| Downstream minter tests affected | No risk | Minter has its own test file; oracle changes don't affect minter test behavior (minter has its own TODO) |

---

## Resolved Questions

The PRD states "No Open Questions" — requirements are fully specified. The implementation is a direct two-line function body and a single constant change in a test assertion.

---

## New Technical Questions Discovered

None. The implementation is straightforward with no ambiguity.

---

## Deviations from Requirements

None found. The existing code structure matches the PRD expectations exactly:
- `apply_price_update()` has the expected TODO stub at the expected location
- The test assertion at line 89 has the expected wrong value (`8` instead of `6`)
- `PRICE_DECIMALS = 6` is correctly defined
- The function signature matches what the PRD documents
- No other files need modification

---

## Files to Modify

| File | Change |
|------|--------|
| `program/programs/sol_usd_oracle/src/lib.rs` | Implement `apply_price_update()` body (lines 10-16) |
| `program/tests/oracle.litesvm.ts` | Fix assertion `8` -> `6` (line 89) |

No other files are in scope per the PRD constraint.
