# Changelog

All notable changes to this project will be documented in this file.

---

## [ML-2] — 2026-03-14

### Fixed

- **Minter program:** Implemented `compute_fee_lamports()` in `token_minter` — replaced the `todo!()` stub with u128 intermediate arithmetic (`fee_lamports = mint_fee_usd * LAMPORTS_PER_SOL / price`), enabling correct USD-to-lamport fee conversion with overflow protection via `checked_mul`.
- **Minter test:** Corrected the inverted fee formula assertion in `minter.litesvm.ts` from `PRICE * LAMPORTS_PER_SOL / FEE_USD` to `FEE_USD * LAMPORTS_PER_SOL / PRICE`, matching the on-chain calculation.

---

## [ML-1] — 2026-03-14

### Fixed

- **Oracle program:** Implemented `apply_price_update()` in `sol_usd_oracle` — replaced the `todo!()` stub with actual state mutation (`oracle.price = new_price`, `oracle.last_updated_slot = current_slot`), enabling the oracle to persist SOL/USD price updates on-chain.
- **Oracle test:** Corrected the decimals assertion in `oracle.litesvm.ts` from `8` to `6` to match the program's `PRICE_DECIMALS = 6` constant.
