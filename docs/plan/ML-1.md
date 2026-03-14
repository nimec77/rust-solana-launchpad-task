# Plan: ML-1 — Fix Oracle Program

**Ticket:** ML-1
**Phase:** 1 (Iteration 1)
**Status:** PLAN_APPROVED

---

## Components

### 1. Oracle Program — `apply_price_update()` Implementation

**File:** `program/programs/sol_usd_oracle/src/lib.rs` (lines 10-16)

The `apply_price_update()` function is a private helper called by the `update_price` instruction handler. It receives a mutable reference to `OracleState`, the new price, and the current slot. All validation (non-zero price, admin authorization) is performed by the caller before this function is invoked.

**Current state:** TODO stub with `todo!()` macro that panics at runtime.

**Target state:** Two field assignments and `Ok(())` return:
- `oracle.price = new_price`
- `oracle.last_updated_slot = current_slot`

### 2. Oracle Test — Decimals Assertion Fix

**File:** `program/tests/oracle.litesvm.ts` (line 89)

The "initialize_oracle sets admin and defaults" test asserts `decoded.decimals == 8`, but the program sets `oracle.decimals = PRICE_DECIMALS` where `PRICE_DECIMALS = 6`.

**Target state:** Assertion changed to `expect(decoded.decimals).to.eq(6)`.

---

## API Contract

No API changes. The `update_price` instruction signature and `OracleState` account structure remain identical. The only change is that `apply_price_update()` now performs the state mutation it was always intended to perform.

### Instruction: `update_price(new_price: u64)`

| Aspect | Value |
|--------|-------|
| Accounts | `oracle` (mut, PDA), `admin` (signer) |
| Validation | `new_price > 0`, `admin == oracle.admin` |
| Mutation | `oracle.price = new_price`, `oracle.last_updated_slot = current_slot` |
| Return | `Ok(())` on success |

---

## Data Flows

```
Admin wallet
    |
    | signs update_price(new_price)
    v
update_price handler
    |
    |- require!(new_price > 0)
    |- require_keys_eq!(admin, oracle.admin)
    |- current_slot = Clock::get()?.slot
    |
    v
apply_price_update(oracle, new_price, current_slot)
    |
    |- oracle.price = new_price
    |- oracle.last_updated_slot = current_slot
    |- Ok(())
    v
OracleState PDA (persisted on-chain)
    |
    | read by token_minter (in later iterations)
    v
compute_fee_lamports() [ML-2 scope]
```

---

## NFR (Non-Functional Requirements)

| Requirement | How Met |
|-------------|---------|
| No arithmetic overflow | Direct u64 assignment, no arithmetic |
| Minimal change scope | Only 2 files modified, per PRD constraint |
| No signature changes | Function signature, account structs, error codes unchanged |
| Build compatibility | Uses pinned Rust 1.89.0 + Anchor 0.32.1 |
| All 4 oracle tests pass | Verified by `anchor build && anchor test --skip-build` |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Overflow in price assignment | None | N/A | Direct u64-to-u64 assignment, no arithmetic |
| Test fix breaks other tests | None | N/A | Decimals assertion is isolated to test 1; tests 2-4 do not check decimals |
| Build failure due to toolchain | Low | Medium | Toolchain pinned in `program/rust-toolchain.toml`; verify with `anchor build` |
| Downstream minter affected | None | N/A | Minter has its own test file and TODO; oracle changes are prerequisite, not breaking |

---

## Deviations to Fix

None. The research document confirmed zero deviations between the existing code structure and the PRD requirements:
- `apply_price_update()` has the expected TODO stub at lines 10-16
- The test assertion at line 89 has the expected wrong value (`8` instead of `6`)
- `PRICE_DECIMALS = 6` is correctly defined at line 3
- The function signature matches the PRD documentation

---

## Implementation Steps

### Step 1: Implement `apply_price_update()`

**File:** `program/programs/sol_usd_oracle/src/lib.rs`

Replace lines 10-16:

```rust
fn apply_price_update(oracle: &mut OracleState, new_price: u64, current_slot: u64) -> Result<()> {
    // TODO(student): finish the happy-path state update.
    // Hint: once validation passes, the oracle should remember both the latest
    // price and the slot at which it was refreshed.
    let _ = (oracle, new_price, current_slot);
    todo!("student task: persist the new price and slot");
}
```

With:

```rust
fn apply_price_update(oracle: &mut OracleState, new_price: u64, current_slot: u64) -> Result<()> {
    oracle.price = new_price;
    oracle.last_updated_slot = current_slot;
    Ok(())
}
```

### Step 2: Fix test assertion

**File:** `program/tests/oracle.litesvm.ts`

At line 89, change:

```typescript
expect(decoded.decimals).to.eq(8);
```

To:

```typescript
expect(decoded.decimals).to.eq(6);
```

### Step 3: Verify

Run: `anchor build && anchor test --skip-build` (or `make test`)

Expected: all 4 oracle tests pass:
1. `initialize_oracle sets admin and defaults` -- passes (decimals assertion now correct)
2. `update_price updates price only for admin` -- passes (apply_price_update now works)
3. `rejects update_price from non-admin signer` -- passes (rejected by admin constraint before apply_price_update)
4. `rejects zero price update` -- passes (rejected by `require!(new_price > 0)` before apply_price_update)

---

## Open Questions

None. Requirements are fully specified per the PRD. No architectural alternatives to evaluate (no ADR needed).
