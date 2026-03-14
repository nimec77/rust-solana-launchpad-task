# Research: ML-2 — Fix Minter Program

**Ticket:** ML-2
**Phase:** 2 (Iteration 2)
**Date:** 2026-03-14

---

## Summary

Two targeted fixes in the `token_minter` program: implement the `compute_fee_lamports()` function body (USD-to-lamports conversion using u128 intermediate arithmetic) and correct the inverted fee formula in the minter test assertion. Iteration 1 (oracle program) is confirmed complete — the `apply_price_update()` TODO stub has been replaced with working code.

---

## Existing Code Analysis

### File 1: `program/programs/token_minter/src/lib.rs`

**Location:** Lines 166-175 — `compute_fee_lamports()` function

The function currently contains a TODO stub:

```rust
fn compute_fee_lamports(mint_fee_usd: u64, price: u64) -> Result<u64> {
    require!(price > 0, MinterError::OraclePriceZero);

    // TODO(student): convert the USD-denominated mint fee into lamports.
    // Both `mint_fee_usd` and `price` use 6 decimal places, so the formula is:
    // fee_lamports = mint_fee_usd * LAMPORTS_PER_SOL / price
    // Keep the integer math and overflow protection from the production version.
    let _ = (mint_fee_usd, price);
    todo!("student task: implement fee conversion");
}
```

**Caller context (line 73):** The `mint_token` instruction handler calls this function:
```rust
let fee_lamports = compute_fee_lamports(ctx.accounts.config.mint_fee_usd, oracle_state.price)?;
```

The returned `fee_lamports` value is immediately used for a `system_program::transfer` CPI (lines 76-85), transferring SOL from the user to the treasury.

**Constants available in scope:**
- `LAMPORTS_PER_SOL_U64: u64 = 1_000_000_000` (defined at line 14)
- `USD_DECIMALS: u8 = 6` (defined at line 13, not needed in the formula)

**Required implementation:**

The formula is `fee_lamports = mint_fee_usd * LAMPORTS_PER_SOL / price`. The multiplication `mint_fee_usd * LAMPORTS_PER_SOL` can overflow u64 (max ~1.8 * 10^19) for fees above ~18 SOL equivalent, so u128 intermediate arithmetic is required:

```rust
let fee = (mint_fee_usd as u128)
    .checked_mul(LAMPORTS_PER_SOL_U64 as u128)
    .ok_or(MinterError::MathOverflow)?
    / (price as u128);
Ok(fee as u64)
```

**Why the formula works with 6-decimal scaling:** Both `mint_fee_usd` and `price` are scaled by 10^6. In the formula `mint_fee_usd * LAMPORTS_PER_SOL / price`, the 10^6 factors in numerator and denominator cancel out, leaving the result in raw lamports (no decimal scaling).

**Example calculation:**
- `mint_fee_usd = 5_000_000` ($5.00 * 10^6)
- `price = 120_000_000` ($120.00 * 10^6)
- `fee = 5_000_000 * 1_000_000_000 / 120_000_000 = 41_666_666` lamports (~0.04167 SOL)
- Verification: 0.04167 SOL * $120/SOL = $5.00

**Error handling considerations:**
- `MinterError::MathOverflow` (error code defined at line 291) is available for checked arithmetic failures
- The `price > 0` check on line 167 prevents division by zero
- There is also a redundant `price > 0` check in the caller at line 67 — both are fine to keep

### File 2: `program/tests/minter.litesvm.ts`

**Location:** Line 172 — inverted fee formula in "initialize oracle + minter and mint token with fee" test

Current (broken) assertion:
```typescript
const expectedFee = PRICE.mul(new BN(anchor.web3.LAMPORTS_PER_SOL)).div(FEE_USD);
```

This computes `120_000_000 * 1_000_000_000 / 5_000_000 = 24_000_000_000_000` — a nonsensical value (~24,000 SOL) that grows when SOL price increases.

Required (correct) formula:
```typescript
const expectedFee = FEE_USD.mul(new BN(anchor.web3.LAMPORTS_PER_SOL)).div(PRICE);
```

This computes `5_000_000 * 1_000_000_000 / 120_000_000 = 41_666_666` lamports — matching the on-chain calculation.

**Test constants (lines 32-33):**
- `PRICE = new BN(120_000_000)` — $120 * 10^6
- `FEE_USD = new BN(5_000_000)` — $5 * 10^6

**Test suite structure (3 tests):**

| # | Test | Current Status | After Fix |
|---|------|----------------|-----------|
| 1 | `initialize oracle + minter and mint token with fee` | Fails (todo!() panic in `compute_fee_lamports`) | Passes (correct fee calculated and correct assertion) |
| 2 | `rejects mint when initial supply is zero` | May pass (fails before `compute_fee_lamports`) | Passes |
| 3 | `rejects mint when decimals exceed allowed range` | May pass (fails before `compute_fee_lamports`) | Passes |

Tests run sequentially and share LiteSVM state. Test 1 initializes both oracle and minter, so tests 2 and 3 depend on test 1 succeeding.

---

## Layers and Dependencies

### Build Dependencies
- **Rust toolchain:** 1.89.0 (pinned in `program/rust-toolchain.toml`)
- **Anchor:** 0.32.1 (pinned in `Cargo.toml`)
- **sol-usd-oracle crate:** local path dependency with `cpi` feature (for `OracleState` and `PRICE_DECIMALS` imports)
- **mpl-token-metadata:** 5.1.1 (for Metaplex metadata CPI, not affected by this change)

### Program Dependency Graph
```
sol_usd_oracle (independent, ML-1 complete)
    ^
    |  account read (cross-program deserialization)
    |  imports: OracleState, PRICE_DECIMALS
    |
token_minter (depends on sol_usd_oracle via Cargo path dependency with "cpi" feature)
```

The `token_minter` reads `OracleState` accounts via Anchor's cross-program account validation (not instruction CPI). In the `MintToken` accounts struct (lines 224-230), the oracle state is validated with `seeds::program = oracle_program` and `owner = oracle_program.key()`.

### Test Dependencies
- LiteSVM (`litesvm` ^0.4.0) — in-process SVM, no network required
- Both `.so` binaries must be pre-built: `target/deploy/sol_usd_oracle.so` and `target/deploy/token_minter.so`
- `@coral-xyz/anchor` for Borsh instruction encoding and account decoding
- `tsx` for TypeScript execution (replaced `ts-mocha` per recent commit 827a87b)
- `mocha` ^11.1.0 + `chai` ^4.3.4

### Iteration 1 Dependency (CONFIRMED SATISFIED)
The oracle program's `apply_price_update()` function has been implemented (no `todo!()` found in `sol_usd_oracle/src/lib.rs`). The oracle test assertion has been fixed (`decimals` check is `6`). The minter test relies on a working oracle to initialize price before minting.

---

## Patterns Used

1. **u128 intermediate arithmetic** — Standard Solana pattern for multiplying two u64 values that can exceed u64 max. Cast operands to u128, perform multiplication, divide, cast result back to u64.
2. **`checked_mul` with error propagation** — Using Anchor's `Result` type with a custom `MinterError::MathOverflow` error code for checked arithmetic.
3. **Fixed-point 6-decimal scaling** — Both USD amounts and prices are `u64` scaled by 10^6. The formula `fee_usd * LAMPORTS / price` causes the 10^6 factors to cancel, producing raw lamports.
4. **Separation of calculation and CPI** — `compute_fee_lamports()` is a pure function (no account access). The caller in `mint_token` uses the result for a `system_program::transfer` CPI.
5. **LiteSVM manual transaction construction** — Tests build `TransactionInstruction` manually with explicit account keys and Borsh-encoded data (no Anchor client library).

---

## Implementation Plan

### Change 1: `program/programs/token_minter/src/lib.rs`

Replace the TODO stub body of `compute_fee_lamports()` (lines 169-174) with:

```rust
let fee = (mint_fee_usd as u128)
    .checked_mul(LAMPORTS_PER_SOL_U64 as u128)
    .ok_or(MinterError::MathOverflow)?
    / (price as u128);
Ok(fee as u64)
```

Remove the `let _ = (mint_fee_usd, price);` and `todo!()` lines. The function signature, visibility, and the `require!(price > 0, ...)` guard on line 167 remain unchanged.

**Note on `checked_div`:** Division by zero is already guarded by `require!(price > 0, ...)` on line 167, so regular division (`/`) is safe here. Using `checked_div` would be redundant but not harmful.

### Change 2: `program/tests/minter.litesvm.ts`

At line 172, change:
```typescript
const expectedFee = PRICE.mul(new BN(anchor.web3.LAMPORTS_PER_SOL)).div(FEE_USD);
```
to:
```typescript
const expectedFee = FEE_USD.mul(new BN(anchor.web3.LAMPORTS_PER_SOL)).div(PRICE);
```

### Verification

Run: `anchor build && anchor test --skip-build` (or `make build && make test`)

Expected result: all 7 tests pass (4 oracle + 3 minter).

---

## Limitations and Risks

| Risk | Assessment | Mitigation |
|------|-----------|------------|
| u64 overflow in `mint_fee_usd * LAMPORTS_PER_SOL` | Medium likelihood for large fees | `checked_mul` with `MinterError::MathOverflow` prevents silent overflow; u128 intermediate handles values up to ~3.4 * 10^38 |
| Integer truncation to zero | Low risk — only when fee < 1 lamport | Occurs when `mint_fee_usd * LAMPORTS_PER_SOL < price`, meaning the fee is less than 1 lamport; acceptable for extremely small fees relative to SOL price |
| `anchor build` fails after edit | Low risk | Change is inside a single function body; no signature or account struct changes |
| Oracle tests regress | No risk | No oracle files are modified; oracle tests are independent |
| Test 2/3 fail due to shared state | No risk | Tests 2 and 3 only rely on minter config being initialized (done in test 1); they deliberately trigger validation errors before reaching fee calculation |
| BN.js overflow in test formula | No risk | `BN` handles arbitrary precision; `5_000_000 * 1_000_000_000 / 120_000_000` is well within safe range |

---

## Resolved Questions

The PRD states "None. The requirements are fully specified and self-contained." No open questions required user input. Implementation proceeds with documented requirements only.

---

## New Technical Questions Discovered

None. The implementation is a direct formula implementation and a test fix with no ambiguity.

---

## Deviations from Requirements

None found. The existing code structure matches the PRD expectations exactly:

- `compute_fee_lamports()` has the expected TODO stub at lines 166-175
- The function signature matches: `fn compute_fee_lamports(mint_fee_usd: u64, price: u64) -> Result<u64>`
- `LAMPORTS_PER_SOL_U64` constant is available at line 14
- `MinterError::MathOverflow` error variant is defined at line 291
- The test assertion at line 172 has the expected inverted formula (`PRICE * LAMPORTS / FEE_USD` instead of `FEE_USD * LAMPORTS / PRICE`)
- The test TODO comment on line 170 confirms intentional breakage
- No other files need modification per the PRD constraint

---

## Files to Modify

| File | Change |
|------|--------|
| `program/programs/token_minter/src/lib.rs` | Implement `compute_fee_lamports()` body (lines 169-174): u128 intermediate arithmetic |
| `program/tests/minter.litesvm.ts` | Fix fee formula assertion (line 172): swap `PRICE`/`FEE_USD` operands |

No other files are in scope per the PRD constraint.
