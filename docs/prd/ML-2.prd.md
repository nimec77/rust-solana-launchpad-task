# PRD: ML-2 — Fix Minter Program

**Status:** PRD_READY
**Ticket:** ML-2
**Phase:** 2 (Iteration 2)

---

## Context / Idea

**Arguments:** ML-2 docs/phase/phase-2.md

This is the second iteration of the Solana Mini-Launchpad project. The token minter program (`token_minter`) has a TODO stub in its fee calculation function and its test has an intentionally inverted fee formula assertion. Both must be fixed. This iteration depends on Iteration 1 (oracle program) being complete, as the minter reads oracle price via CPI account validation.

### Source: docs/phase/phase-2.md

> **Goal:** Implement the fee calculation logic and fix the broken test assertion.
>
> **Tasks:**
> - Implement `compute_fee_lamports()` in `program/programs/token_minter/src/lib.rs`
>   - Formula: `fee_lamports = mint_fee_usd * LAMPORTS_PER_SOL / price` (use u128 intermediate)
> - Fix fee formula in `program/tests/minter.litesvm.ts` — invert the assertion formula
>
> **Acceptance Criteria:**
> `anchor test --skip-build` — all program tests pass
>
> **Dependencies:**
> - Iteration 1 complete (oracle program must work for CPI price reads)
>
> **Implementation Notes:**
> - Fee formula uses 6 decimal scaling for both `mint_fee_usd` and `price`
> - Use `u128` intermediate to avoid overflow during multiplication
> - The test assertion has an intentionally inverted fee formula that students must fix

---

## Goals

1. **Implement fee calculation logic** — The `compute_fee_lamports()` function in `program/programs/token_minter/src/lib.rs` must convert a USD-denominated mint fee into lamports using the oracle's SOL/USD price.
2. **Fix broken test assertion** — The test in `program/tests/minter.litesvm.ts` uses an inverted fee formula (`PRICE * LAMPORTS_PER_SOL / FEE_USD`) that must be corrected to `FEE_USD * LAMPORTS_PER_SOL / PRICE`.
3. **Establish a passing test baseline for both programs** — All program LiteSVM tests (oracle and minter suites) must pass so that subsequent iterations (backend, deployment) can build on working on-chain programs.

---

## User Stories

1. **As a developer**, I want `compute_fee_lamports()` to correctly convert a USD fee into lamports so that users pay the correct SOL amount when minting a token.
2. **As a developer**, I want the minter test suite to pass with correct assertions so that I can trust the tests as a regression safety net for future changes.
3. **As a token creator (end user)**, I need the fee calculation to produce a sensible lamport amount (fee gets smaller when SOL price is higher) so that I am charged the correct USD-equivalent fee.
4. **As a treasury operator**, I need the full fee amount transferred to the treasury account on each mint so that revenue is collected correctly.

---

## Scenarios

### Scenario 1: Successful token mint with correct fee

- **Given** the oracle is initialized with price = 120_000_000 ($120.00) and the minter is initialized with mint_fee_usd = 5_000_000 ($5.00)
- **When** a user calls `mint_token` with valid parameters (decimals=6, initial_supply=1_000_000)
- **Then** the fee transferred to treasury equals `5_000_000 * 1_000_000_000 / 120_000_000 = 41_666_666` lamports (~0.04167 SOL), a new mint account is created, and tokens are minted to the user's ATA

### Scenario 2: Fee decreases when SOL price increases

- **Given** mint_fee_usd = 5_000_000 ($5.00)
- **When** the oracle price increases from 120_000_000 ($120) to 240_000_000 ($240)
- **Then** the fee_lamports decreases from 41_666_666 to 20_833_333 (halved, since the same $5 costs fewer lamports at a higher SOL price)

### Scenario 3: Overflow protection with u128 intermediate

- **Given** mint_fee_usd = 1_000_000_000 ($1000.00) and LAMPORTS_PER_SOL = 1_000_000_000
- **When** `compute_fee_lamports()` multiplies them
- **Then** the u128 intermediate handles the product (10^18) without overflow, and the final result fits in u64 after division by price

### Scenario 4: Zero oracle price rejected

- **Given** the oracle price is 0
- **When** `compute_fee_lamports()` is called
- **Then** it returns `MinterError::OraclePriceZero` (division by zero is prevented)

### Scenario 5: Mint with zero supply rejected

- **Given** the minter is initialized
- **When** a user calls `mint_token` with `initial_supply = 0`
- **Then** the transaction fails and no fee is transferred to treasury

### Scenario 6: Mint with decimals exceeding allowed range rejected

- **Given** the minter is initialized
- **When** a user calls `mint_token` with `decimals = 10` (exceeds max of 9)
- **Then** the transaction fails and no fee is transferred to treasury

---

## Metrics

| Metric | Target |
|--------|--------|
| Minter LiteSVM tests passing | 3/3 (all tests green) |
| Oracle LiteSVM tests passing | 4/4 (remain green from Iteration 1) |
| `anchor build` succeeds | Yes |
| `anchor test --skip-build` succeeds | Yes (both oracle and minter suites) |

---

## Constraints

1. **Minimal change scope** — Only two files are modified: `program/programs/token_minter/src/lib.rs` and `program/tests/minter.litesvm.ts`. No other files should be touched.
2. **No architectural changes** — The `compute_fee_lamports()` function signature (`fn compute_fee_lamports(mint_fee_usd: u64, price: u64) -> Result<u64>`) and the program's account structures must remain unchanged.
3. **6 decimal places** — Both `mint_fee_usd` and `price` use 6 decimal scaling. The formula `fee_lamports = mint_fee_usd * LAMPORTS_PER_SOL / price` relies on the 10^6 factors canceling out.
4. **u128 intermediate arithmetic** — The multiplication `mint_fee_usd * LAMPORTS_PER_SOL` can overflow u64 for large fee values. Use u128 for the intermediate calculation to prevent overflow.
5. **Anchor version 0.32.1** — Must build with the project's pinned Anchor version.
6. **Rust toolchain 1.89.0** — Must compile with the project's pinned Rust version (defined in `program/rust-toolchain.toml`).
7. **Iteration 1 dependency** — Oracle program must be working (Iteration 1 complete) since the minter reads the oracle price via CPI account validation.

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| u64 overflow in fee calculation | Medium | High | Use u128 intermediate as specified; `mint_fee_usd * LAMPORTS_PER_SOL` can exceed u64 max (~1.8 * 10^19) for fees above ~18 SOL equivalent |
| Integer truncation produces zero fee | Low | High | Only occurs when `mint_fee_usd * LAMPORTS_PER_SOL < price`, which means fee < 1 lamport; acceptable for extremely small fees relative to SOL price |
| Test fix causes mismatch with implementation | Low | Medium | The correct formula in both implementation and test is `fee_lamports = mint_fee_usd * LAMPORTS_PER_SOL / price`; verify both use the same formula |
| Oracle tests regress | Low | Low | Oracle tests are independent of minter changes; run full test suite to confirm |

---

## Open Questions

None. The requirements are fully specified and self-contained. Implementation can proceed.

---

## Implementation Reference

### File 1: `program/programs/token_minter/src/lib.rs`

**Location:** `compute_fee_lamports()` function (line 166-175)

**Current code (TODO stub):**
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

**Required:** Implement the formula `fee_lamports = mint_fee_usd * LAMPORTS_PER_SOL / price` using u128 intermediate to avoid overflow. Cast `mint_fee_usd` and `LAMPORTS_PER_SOL_U64` to u128, multiply, divide by `price` as u128, then cast the result back to u64.

### File 2: `program/tests/minter.litesvm.ts`

**Location:** Fee assertion in `initialize oracle + minter and mint token with fee` test (line 172)

**Current code (inverted formula):**
```typescript
const expectedFee = PRICE.mul(new BN(anchor.web3.LAMPORTS_PER_SOL)).div(FEE_USD);
```

**Required:** Invert to `FEE_USD.mul(new BN(anchor.web3.LAMPORTS_PER_SOL)).div(PRICE)` so that the expected fee matches the correct formula where fee gets smaller when SOL price gets larger.
