# Changelog

All notable changes to this project will be documented in this file.

---

## [ML-1] — 2026-03-14

### Fixed

- **Oracle program:** Implemented `apply_price_update()` in `sol_usd_oracle` — replaced the `todo!()` stub with actual state mutation (`oracle.price = new_price`, `oracle.last_updated_slot = current_slot`), enabling the oracle to persist SOL/USD price updates on-chain.
- **Oracle test:** Corrected the decimals assertion in `oracle.litesvm.ts` from `8` to `6` to match the program's `PRICE_DECIMALS = 6` constant.
