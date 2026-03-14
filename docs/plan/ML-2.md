# Plan: ML-2 — Fix Minter Program

**Ticket:** ML-2
**Phase:** 2 (Iteration 2)
**Status:** PLAN_APPROVED

---

## Components

### 1. Minter Program — `compute_fee_lamports()` Implementation

**File:** `program/programs/token_minter/src/lib.rs` (lines 166-175)

The `compute_fee_lamports()` function is a private helper called by the `mint_token` instruction handler at line 73. It receives `mint_fee_usd` (USD fee scaled by 10^6) and `price` (SOL/USD price scaled by 10^6), and must return the equivalent fee in lamports.

**Current state:** TODO stub with `todo!()` macro that panics at runtime. The zero-price guard (`require!(price > 0, MinterError::OraclePriceZero)`) is already in place at line 167.

**Target state:** Replace the TODO stub with u128 intermediate arithmetic:
- Cast `mint_fee_usd` and `LAMPORTS_PER_SOL_U64` to u128
- Multiply with `checked_mul`, propagating `MinterError::MathOverflow` on failure
- Divide by `price` (as u128) — safe because the zero-price guard precedes it
- Cast the result back to u64 and return

**Constants used:**
- `LAMPORTS_PER_SOL_U64: u64 = 1_000_000_000` (line 14)
- `MinterError::MathOverflow` (line 291)

### 2. Minter Test — Fee Formula Assertion Fix

**File:** `program/tests/minter.litesvm.ts` (line 172)

The "initialize oracle + minter and mint token with fee" test computes `expectedFee` with an inverted formula: `PRICE * LAMPORTS_PER_SOL / FEE_USD`. This produces a nonsensical value (~24,000 SOL) that grows when SOL price increases — the opposite of correct economic behavior.

**Target state:** Swap operands to `FEE_USD * LAMPORTS_PER_SOL / PRICE`, producing `41_666_666` lamports (~0.04167 SOL, i.e. $5.00 at $120/SOL).

---

## API Contract

No API changes. The `mint_token` instruction signature, `MintToken` accounts struct, `MinterConfig` account structure, and all error codes remain identical. The only change is that `compute_fee_lamports()` now performs the fee conversion it was always intended to perform.

### Function: `compute_fee_lamports(mint_fee_usd: u64, price: u64) -> Result<u64>`

| Aspect | Value |
|--------|-------|
| Visibility | Private (module-level `fn`) |
| Inputs | `mint_fee_usd` (USD * 10^6), `price` (SOL/USD * 10^6) |
| Validation | `price > 0` (already present, unchanged) |
| Computation | `(mint_fee_usd as u128) * (LAMPORTS_PER_SOL_U64 as u128) / (price as u128)` |
| Overflow protection | `checked_mul` with `MinterError::MathOverflow` |
| Return | `Ok(fee as u64)` on success |

---

## Data Flows

```
User (Frontend / Test)
    |
    | signs mint_token(decimals, supply, name, symbol, uri)
    v
mint_token handler (token_minter)
    |
    |- require!(initial_supply > 0)
    |- require!(decimals <= 9)
    |- require!(oracle_state.price > 0)
    |- require!(oracle_state.decimals == 6)
    |
    v
compute_fee_lamports(config.mint_fee_usd, oracle_state.price)
    |
    |- require!(price > 0)                         [redundant guard, kept]
    |- fee = (mint_fee_usd as u128)
    |         .checked_mul(LAMPORTS_PER_SOL as u128)
    |         .ok_or(MathOverflow)?
    |         / (price as u128)
    |- Ok(fee as u64)
    v
system_program::transfer(user -> treasury, fee_lamports)
    |
    v
token::mint_to(mint -> user_ata, initial_supply)
    |
    v
emit!(TokenCreated { ..., fee_lamports, ... })
```

**Example calculation (from PRD Scenario 1):**
- `mint_fee_usd = 5_000_000` ($5.00 * 10^6)
- `price = 120_000_000` ($120.00 * 10^6)
- `fee = 5_000_000 * 1_000_000_000 / 120_000_000 = 41_666_666` lamports
- Verification: 41_666_666 lamports = ~0.04167 SOL; 0.04167 * $120 = $5.00

**Decimal cancellation:** Both `mint_fee_usd` and `price` carry 10^6 scaling. In the fraction `(mint_fee_usd * LAMPORTS_PER_SOL) / price`, the 10^6 factors cancel, yielding raw lamports with no residual scaling.

---

## NFR (Non-Functional Requirements)

| Requirement | How Met |
|-------------|---------|
| No u64 overflow | u128 intermediate arithmetic with `checked_mul` |
| Division-by-zero safety | `require!(price > 0)` guard at line 167 (unchanged) |
| Minimal change scope | Only 2 files modified per PRD constraint |
| No signature changes | Function signature, account structs, error codes unchanged |
| Build compatibility | Uses pinned Rust 1.89.0 + Anchor 0.32.1 |
| All 7 tests pass | 4 oracle tests (unchanged from ML-1) + 3 minter tests |
| Integer truncation | Acceptable — only occurs when fee < 1 lamport (economically negligible) |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| u64 overflow in `mint_fee_usd * LAMPORTS_PER_SOL` | Medium (for fees > ~18 SOL) | High (silent wrong result) | u128 intermediate + `checked_mul` with `MinterError::MathOverflow` |
| Integer truncation to zero fee | Low | Low | Only when `mint_fee_usd * LAMPORTS_PER_SOL < price` (fee < 1 lamport); economically acceptable |
| Test formula fix mismatches implementation | Low | Medium | Both use identical formula: `fee_usd * LAMPORTS_PER_SOL / price` |
| Oracle tests regress | None | N/A | No oracle files modified; oracle tests are independent |
| Tests 2-3 fail from shared state dependency | None | N/A | Tests 2-3 depend only on minter config being initialized (done in test 1); they trigger validation errors before reaching fee calculation |
| Build failure | Low | Medium | Change is inside a single function body; no new imports, no structural changes |

---

## Deviations to Fix

None. The research document confirmed zero deviations between the existing code and the PRD requirements:
- `compute_fee_lamports()` has the expected TODO stub at lines 166-175
- The function signature matches: `fn compute_fee_lamports(mint_fee_usd: u64, price: u64) -> Result<u64>`
- `LAMPORTS_PER_SOL_U64: u64 = 1_000_000_000` is defined at line 14
- `MinterError::MathOverflow` is defined at line 291
- The test assertion at line 172 has the expected inverted formula (`PRICE * LAMPORTS / FEE_USD`)
- The TODO comment at line 170 confirms intentional breakage
- No other files need modification per the PRD constraint

---

## Implementation Steps

### Step 1: Implement `compute_fee_lamports()`

**File:** `program/programs/token_minter/src/lib.rs`

Replace lines 169-174 (the TODO stub body, preserving the function signature and `require!` guard):

```rust
    // TODO(student): convert the USD-denominated mint fee into lamports.
    // Both `mint_fee_usd` and `price` use 6 decimal places, so the formula is:
    // fee_lamports = mint_fee_usd * LAMPORTS_PER_SOL / price
    // Keep the integer math and overflow protection from the production version.
    let _ = (mint_fee_usd, price);
    todo!("student task: implement fee conversion");
```

With:

```rust
    let fee = (mint_fee_usd as u128)
        .checked_mul(LAMPORTS_PER_SOL_U64 as u128)
        .ok_or(MinterError::MathOverflow)?
        / (price as u128);
    Ok(fee as u64)
```

The function after the change:

```rust
fn compute_fee_lamports(mint_fee_usd: u64, price: u64) -> Result<u64> {
    require!(price > 0, MinterError::OraclePriceZero);

    let fee = (mint_fee_usd as u128)
        .checked_mul(LAMPORTS_PER_SOL_U64 as u128)
        .ok_or(MinterError::MathOverflow)?
        / (price as u128);
    Ok(fee as u64)
}
```

**Rationale for `checked_mul` + plain division:**
- `checked_mul`: prevents silent overflow if `mint_fee_usd * LAMPORTS_PER_SOL` exceeds u128 max (practically impossible with u64 inputs, but defensive)
- Plain division (`/`): safe because `price > 0` is enforced by the `require!` guard at line 167. Using `checked_div` would be redundant.

### Step 2: Fix test fee assertion

**File:** `program/tests/minter.litesvm.ts`

At line 172, change:

```typescript
const expectedFee = PRICE.mul(new BN(anchor.web3.LAMPORTS_PER_SOL)).div(FEE_USD);
```

To:

```typescript
const expectedFee = FEE_USD.mul(new BN(anchor.web3.LAMPORTS_PER_SOL)).div(PRICE);
```

This produces `5_000_000 * 1_000_000_000 / 120_000_000 = 41_666_666` lamports, matching the on-chain calculation and the correct economic behavior (fee decreases when SOL price increases).

### Step 3: Build and verify

Run: `anchor build && anchor test --skip-build` (or `make build && make test`)

Expected: all 7 tests pass (4 oracle + 3 minter):

**Oracle tests (unchanged from ML-1):**
1. `initialize_oracle sets admin and defaults` -- passes
2. `update_price updates price only for admin` -- passes
3. `rejects update_price from non-admin signer` -- passes
4. `rejects zero price update` -- passes

**Minter tests:**
1. `initialize oracle + minter and mint token with fee` -- passes (fee correctly computed as 41_666_666 lamports; assertion matches)
2. `rejects mint when initial supply is zero` -- passes (rejected by `InvalidSupply` before fee calculation)
3. `rejects mint when decimals exceed allowed range` -- passes (rejected by `InvalidDecimals` before fee calculation)

---

## Open Questions

None. Requirements are fully specified per the PRD. No architectural alternatives to evaluate (no ADR needed).
