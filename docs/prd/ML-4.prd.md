# PRD: ML-4 — Generate Own Program IDs

**Status:** PRD_READY
**Ticket:** ML-4
**Phase:** 4 (Iteration 4)

---

## Context / Idea

**Arguments:** ML-4 docs/phase/phase-4.md

The codebase currently ships with hardcoded program IDs (`sol_usd_oracle = 4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU`, `token_minter = E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE`) that belong to the original author's deployment. Students must generate their own keypairs for both programs and update all references across the codebase so that they own and control the deployed programs on their local validator (and eventually devnet).

### Source: docs/phase/phase-4.md

> **Goal:** Generate fresh keypairs for both programs and update all references across the codebase.
>
> **Tasks:**
> - 4.1 Generate new keypairs (`solana-keygen grind` or `anchor keys sync`)
> - 4.2 Update program IDs in 4 locations:
>   - `program/programs/sol_usd_oracle/src/lib.rs` — `declare_id!()`
>   - `program/programs/token_minter/src/lib.rs` — `declare_id!()`
>   - `program/Anchor.toml`
>   - `frontend/app/config.ts`
>
> **Acceptance Criteria:**
> `anchor build` succeeds with new IDs
>
> **Dependencies:**
> - Iteration 3 complete
>
> **Implementation Notes:**
> - Current program IDs: `sol_usd_oracle = 4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU`, `token_minter = E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE`
> - Program IDs must match across all 4 locations
> - `anchor keys sync` can auto-update `declare_id!` and `Anchor.toml` from deploy keypairs

---

## Goals

1. **Generate fresh program keypairs** — Create new keypairs for both `sol_usd_oracle` and `token_minter` so the student owns the program deployment keys rather than using the original author's IDs.
2. **Update all program ID references** — Replace the old program IDs in all locations across the codebase so that builds, deployments, tests, and the frontend all reference the new IDs consistently.
3. **Verify build integrity** — Confirm that `anchor build` succeeds with the new program IDs, ensuring the programs compile and link correctly.

---

## User Stories

1. **As a student**, I want to generate my own program keypairs so that I own the deployment authority and can deploy both programs to my local validator and devnet under my control.
2. **As a developer**, I want all program ID references across the codebase to be consistent so that the oracle, minter, backend, frontend, tests, and scripts all point to the same programs.
3. **As a deployer**, I want `anchor build` to succeed with the new IDs so that I can proceed with deployment and local end-to-end testing in subsequent iterations.

---

## Scenarios

### Scenario 1: Generate new keypairs via `anchor keys sync`

- **Given** the Anchor project is set up in `program/`
- **When** the student runs `anchor keys sync` (or generates new keypairs manually)
- **Then** new keypair JSON files are created under `program/target/deploy/` for both `sol_usd_oracle-keypair.json` and `token_minter-keypair.json`
- **And** the `declare_id!()` macros and `Anchor.toml` entries are updated to match the new public keys

### Scenario 2: Update frontend config

- **Given** new program IDs have been generated
- **When** the student updates `frontend/app/config.ts`
- **Then** `ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID` reflect the new public keys

### Scenario 3: Anchor build succeeds

- **Given** all 4 locations have been updated with the new, matching program IDs
- **When** `anchor build` is run from the `program/` directory
- **Then** the build completes successfully without errors

### Scenario 4: Program IDs are consistent across all locations

- **Given** the new IDs have been applied
- **When** the IDs in `declare_id!()` (both programs), `Anchor.toml` (localnet and devnet sections), and `frontend/app/config.ts` are compared
- **Then** all references for `sol_usd_oracle` match each other and all references for `token_minter` match each other

---

## Metrics

| Metric | Target |
|--------|--------|
| `anchor build` succeeds | Yes |
| Program IDs consistent across all 4 specified locations | Yes |
| New IDs differ from original IDs | Yes |

---

## Constraints

1. **Exactly 4 locations specified for update** — The phase document specifies updating program IDs in 4 locations: `program/programs/sol_usd_oracle/src/lib.rs` (`declare_id!()`), `program/programs/token_minter/src/lib.rs` (`declare_id!()`), `program/Anchor.toml`, and `frontend/app/config.ts`.
2. **IDs must be consistent** — The same oracle program ID must appear in all oracle references, and the same minter program ID must appear in all minter references.
3. **`anchor keys sync` is the recommended approach** — It auto-generates keypairs and updates `declare_id!` and `Anchor.toml` from the deploy keypair files, reducing manual errors.
4. **Anchor version 0.32.1** — Must use the project's pinned Anchor version.
5. **Iteration 3 dependency** — Backend fixes (Iteration 3) must be complete before starting this iteration.

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Missed reference — program IDs exist in more than the 4 specified locations | High | Medium | A codebase-wide search reveals the old IDs also appear in `program/scripts/init-local.js`, `program/scripts/get-oracle-pda.js`, `program/tests/oracle.litesvm.ts`, `program/tests/minter.litesvm.ts`, `program/migrations/deploy.ts`, `backend/.env.example`, and `CLAUDE.md`. While the phase spec lists only 4 locations, leaving the others stale will cause runtime failures in tests, scripts, and backend. These additional locations should also be updated. |
| `anchor keys sync` not available or behaves differently | Low | Low | Fall back to manual `solana-keygen grind` or `solana-keygen new` for each keypair, then manually update the 4 locations |
| Old keypair files in `target/deploy/` conflict | Low | Low | Delete old keypair files before generating new ones, or let `anchor keys sync` overwrite them |
| Forgetting to update both `[programs.localnet]` and `[programs.devnet]` in Anchor.toml | Medium | Medium | Anchor.toml has program IDs under both `[programs.localnet]` and `[programs.devnet]` — both sections must be updated |

---

## Open Questions

None. The requirements are fully specified. The only notable observation is that program IDs exist in more locations than the 4 listed in the phase document (see Risks above). The implementer should decide whether to update all locations for consistency or strictly follow the 4-location scope and address the others in a follow-up.

---

## Implementation Reference

### Task 4.1: Generate new keypairs

**Method:** Run `anchor keys sync` from the `program/` directory, or manually:
```bash
solana-keygen new -o program/target/deploy/sol_usd_oracle-keypair.json --no-bip39-passphrase --force
solana-keygen new -o program/target/deploy/token_minter-keypair.json --no-bip39-passphrase --force
```
Then extract public keys with `solana-keygen pubkey program/target/deploy/<name>-keypair.json`.

### Task 4.2: Update program IDs in 4 locations

**Location 1:** `program/programs/sol_usd_oracle/src/lib.rs` (line 8)
```rust
declare_id!("4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU");
```
Replace with the new oracle program public key.

**Location 2:** `program/programs/token_minter/src/lib.rs` (line 16)
```rust
declare_id!("E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE");
```
Replace with the new minter program public key.

**Location 3:** `program/Anchor.toml` (lines 9-10 and 13-14)
```toml
[programs.localnet]
sol_usd_oracle = "4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU"
token_minter = "E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE"

[programs.devnet]
sol_usd_oracle = "4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU"
token_minter = "E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE"
```
Replace all four entries with the new public keys.

**Location 4:** `frontend/app/config.ts` (lines 2-3)
```ts
export const ORACLE_PROGRAM_ID = "4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU";
export const MINTER_PROGRAM_ID = "E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE";
```
Replace with the new public keys.

### Additional locations (beyond phase-4 scope, but contain old IDs)

- `program/scripts/init-local.js` (lines 22-23)
- `program/scripts/get-oracle-pda.js` (line 6)
- `program/tests/oracle.litesvm.ts` (line 16)
- `program/tests/minter.litesvm.ts` (lines 24-25)
- `program/migrations/deploy.ts` (lines 10-11)
- `backend/.env.example` (lines 10-11)
