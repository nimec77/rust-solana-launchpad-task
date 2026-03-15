# Changelog

All notable changes to this project will be documented in this file.

---

## [ML-5] — 2026-03-15

### Fixed

- **Backend `.env.example`:** Corrected stale oracle program ID (`88GBkKvZbhtTcXy2tpwHQSVxHqPqurQbjKW1nLzER84c`) in the PDA derivation comment on line 14 to use the correct ID (`GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3`), eliminating the last remaining stale program ID reference from the ML-4 migration.

### Validated

- **Local end-to-end cycle:** Brought the full local stack online (validator with Metaplex, both programs deployed and initialized, backend price updater + event listener, Remix frontend on port 7001) and successfully minted a token via the browser UI. Backend captured `TokenCreated` event JSON, confirming the complete pipeline works end-to-end.

---

## [ML-4] — 2026-03-14

### Changed

- **Program IDs:** Replaced hardcoded original-author program IDs (`sol_usd_oracle = 4cuvLFF...`, `token_minter = E5erGza...`) with locally generated keypair-derived public keys (`sol_usd_oracle = GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3`, `token_minter = 3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn`) across all 11 source files: `declare_id!` macros in both programs, `Anchor.toml` (localnet + devnet), `frontend/app/config.ts`, init and utility scripts, LiteSVM tests, migration script, `backend/.env.example`, and `CLAUDE.md`.

---

## [ML-3] — 2026-03-14

### Fixed

- **Backend price parser:** Implemented `to_fixed_6()` in `backend/src/main.rs` — replaced the `todo!()` stub with string-based decimal parsing that splits on the decimal point, truncates to 6 fractional digits (no rounding), and produces a fixed-point u64 value. Uses no floating-point arithmetic.
- **Backend test:** Corrected the expected value in `to_fixed_6_truncates_fraction_to_six_digits` from `1_123_457` (rounded) to `1_123_456` (truncated), matching the specified truncation-not-rounding behavior.

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
