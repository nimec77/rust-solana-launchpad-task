# Solana Mini-Launchpad — Task List

**Current Phase:** 1

## Progress

| #  | Iteration                    | Status |
|----|------------------------------|--------|
| 1  | Fix Oracle Program           | ⬜      |
| 2  | Fix Minter Program           | ⬜      |
| 3  | Fix Backend                  | ⬜      |
| 4  | Generate Own Program IDs     | ⬜      |
| 5  | Local End-to-End Cycle       | ⬜      |
| 6  | Devnet Deployment            | ⬜      |

---

## Iteration 1: Fix Oracle Program

- [ ] Implement `apply_price_update()` in `program/programs/sol_usd_oracle/src/lib.rs`
  - Set `oracle.price = new_price` and `oracle.last_updated_slot = current_slot`
- [ ] Fix decimals assertion in `program/tests/oracle.litesvm.ts` (`8` → `6`)

**Test:** `anchor build && anchor test --skip-build` — oracle tests pass

---

## Iteration 2: Fix Minter Program

- [ ] Implement `compute_fee_lamports()` in `program/programs/token_minter/src/lib.rs`
  - Formula: `fee_lamports = mint_fee_usd * LAMPORTS_PER_SOL / price` (use u128 intermediate)
- [ ] Fix fee formula in `program/tests/minter.litesvm.ts` — invert the assertion formula

**Test:** `anchor test --skip-build` — all program tests pass

---

## Iteration 3: Fix Backend

- [ ] Implement `to_fixed_6()` in `backend/src/main.rs`
  - Parse decimal string → `u64` with 6 fixed decimals; truncate, don't round
- [ ] Fix test `to_fixed_6_truncates_fraction_to_six_digits` — change expected `1_123_457` → `1_123_456`

**Test:** `cargo test` — all backend tests pass

---

## Iteration 4: Generate Own Program IDs

- [ ] Generate new keypairs (`solana-keygen grind` or `anchor keys sync`)
- [ ] Update program IDs in 4 locations:
  - `program/programs/sol_usd_oracle/src/lib.rs` — `declare_id!()`
  - `program/programs/token_minter/src/lib.rs` — `declare_id!()`
  - `program/Anchor.toml`
  - `frontend/app/config.ts`

**Test:** `anchor build` succeeds with new IDs

---

## Iteration 5: Local End-to-End Cycle

- [ ] Start validator: `make validator-metaplex`
- [ ] Deploy & init: `make deploy && make init`
- [ ] Configure `backend/.env` (copy `.env.example`, set `ORACLE_STATE_PUBKEY` from init output)
- [ ] Run backend + frontend: `make backend` + `make frontend`
- [ ] Mint a token via UI (connect wallet, fill form, submit)

**Test:** backend logs show `TokenCreated` event JSON

---

## Iteration 6: Devnet Deployment

- [ ] Switch to devnet: `solana config set --url devnet && solana airdrop 2`
- [ ] Deploy & init: `make deploy-devnet && make init-devnet`
- [ ] Configure backend `.env` for devnet RPC endpoints
- [ ] Mint at least one token on devnet via frontend
- [ ] Update `README.md` with deployed program addresses and devnet tx links

**Test:** devnet transaction links are valid and viewable on explorer
