# Iteration 4: Generate Own Program IDs

**Goal:** Generate fresh keypairs for both programs and update all references across the codebase.

## Tasks

- [x] 4.1 Generate new keypairs (`solana-keygen grind` or `anchor keys sync`)
- [x] 4.2 Update program IDs in 4 locations:
  - `program/programs/sol_usd_oracle/src/lib.rs` — `declare_id!()`
  - `program/programs/token_minter/src/lib.rs` — `declare_id!()`
  - `program/Anchor.toml`
  - `frontend/app/config.ts`

## Acceptance Criteria

**Test:** `anchor build` succeeds with new IDs

## Dependencies

- Iteration 3 complete

## Implementation Notes

- Current program IDs: `sol_usd_oracle = 4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU`, `token_minter = E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE`
- Program IDs must match across all 4 locations
- `anchor keys sync` can auto-update `declare_id!` and `Anchor.toml` from deploy keypairs
