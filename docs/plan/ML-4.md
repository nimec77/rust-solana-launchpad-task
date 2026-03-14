# Plan: ML-4 — Generate Own Program IDs

**Ticket:** ML-4
**Phase:** 4 (Iteration 4)
**Status:** PLAN_APPROVED

---

## Components

### 1. Program Keypairs — Generation via `anchor keys sync`

**Directory:** `program/target/deploy/`

Keypair files already exist and contain keys that differ from the hardcoded source IDs:

| Keypair file | Current pubkey (from keypair) | Hardcoded in source |
|---|---|---|
| `program/target/deploy/sol_usd_oracle-keypair.json` | `GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3` | `4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU` |
| `program/target/deploy/token_minter-keypair.json` | `3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn` | `E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE` |

Running `anchor keys sync` from `program/` will read these existing keypair files and synchronize the source code (`declare_id!` macros and `Anchor.toml`) to match. No new keypair generation is needed -- the keypairs already exist.

### 2. Primary Locations (4 locations per phase-4 spec)

These are the locations explicitly required by the phase-4 specification. `anchor keys sync` handles locations 1-3 automatically; location 4 requires manual update.

**Location 1:** `program/programs/sol_usd_oracle/src/lib.rs` (line 8)
- **Current:** `declare_id!("4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU");`
- **Target:** `declare_id!("<new oracle pubkey>");`
- **Method:** `anchor keys sync` (automatic)

**Location 2:** `program/programs/token_minter/src/lib.rs` (line 16)
- **Current:** `declare_id!("E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE");`
- **Target:** `declare_id!("<new minter pubkey>");`
- **Method:** `anchor keys sync` (automatic)

**Location 3:** `program/Anchor.toml` (lines 9-10, 13-14)
- **Current:** Both `[programs.localnet]` and `[programs.devnet]` contain old IDs
- **Target:** All four entries updated to new pubkeys
- **Method:** `anchor keys sync` (automatic)

**Location 4:** `frontend/app/config.ts` (lines 2-3)
- **Current:** `ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID` reference old IDs
- **Target:** Both constants updated to new pubkeys
- **Method:** Manual text replacement

### 3. Additional Locations (beyond phase-4 scope, required for working system)

The research identified 7 additional files containing hardcoded old program IDs. Leaving these stale will cause runtime failures in tests, init scripts, migration, and backend configuration. These must also be updated.

**Critical for functionality:**

| # | File | Line(s) | IDs | Impact if stale |
|---|------|---------|-----|----------------|
| 5 | `program/scripts/init-local.js` | 22-23 | Both | `make init` fails -- PDA derivation and instruction routing use wrong program ID |
| 6 | `program/scripts/get-oracle-pda.js` | 6 | Oracle | Utility outputs wrong PDA address |
| 7 | `program/tests/oracle.litesvm.ts` | 16 | Oracle | LiteSVM test loads `.so` against wrong program ID, test fails |
| 8 | `program/tests/minter.litesvm.ts` | 24-25 | Both | LiteSVM tests load `.so` files against wrong program IDs, tests fail |
| 9 | `program/migrations/deploy.ts` | 10-11 | Both | `anchor migrate` fails |

**Documentation and templates:**

| # | File | Line(s) | IDs | Impact if stale |
|---|------|---------|-----|----------------|
| 10 | `backend/.env.example` | 10-11 | Both | Template misleads; copied `.env` will have wrong program IDs |
| 11 | `CLAUDE.md` | 106 | Both | AI assistant context becomes inaccurate |

**Method for all:** Manual text replacement of old ID strings with new pubkeys.

### 4. Build Verification

After all source updates, run `anchor build` from `program/` to verify:
- `declare_id!` macros compile with valid base58 keys
- `token_minter`'s CPI reference to `sol_usd_oracle` uses the updated oracle `declare_id!`
- Both `.so` binaries are regenerated with the new IDs embedded
- IDL files in `target/idl/` are regenerated automatically

---

## API Contract

No API changes. This ticket modifies only static configuration constants (program IDs). No function signatures, instruction formats, account structures, or public interfaces change.

The only semantic impact is that all PDA addresses derived from the program IDs will change. Specifically, `ORACLE_STATE_PUBKEY` (derived from `oracle_state` seed + oracle program ID) will have a new value. This is expected and correct -- the init script (`make init`) outputs the new PDA value for use in `backend/.env`.

---

## Data Flows

```
anchor keys sync
    |
    |- Reads program/target/deploy/sol_usd_oracle-keypair.json -> extracts pubkey
    |- Reads program/target/deploy/token_minter-keypair.json  -> extracts pubkey
    |
    |- Updates program/programs/sol_usd_oracle/src/lib.rs     (declare_id!)
    |- Updates program/programs/token_minter/src/lib.rs       (declare_id!)
    |- Updates program/Anchor.toml                            ([programs.*] sections)
    v
Manual updates (using the same extracted pubkeys)
    |
    |- frontend/app/config.ts
    |- program/scripts/init-local.js
    |- program/scripts/get-oracle-pda.js
    |- program/tests/oracle.litesvm.ts
    |- program/tests/minter.litesvm.ts
    |- program/migrations/deploy.ts
    |- backend/.env.example
    |- CLAUDE.md
    v
anchor build
    |
    |- Compiles sol_usd_oracle with new declare_id!
    |- Compiles token_minter with new declare_id! (and CPI ref to new oracle ID)
    |- Generates target/deploy/*.so with new IDs embedded
    |- Generates target/idl/*.json with new program metadata
    v
Build succeeds (acceptance criterion met)
```

**Post-implementation flow (not part of this ticket, but for completeness):**
```
make init  ->  outputs new ORACLE_STATE_PUBKEY PDA
    |
    v
Copy PDA to backend/.env  ->  backend connects to correct oracle account
```

---

## NFR (Non-Functional Requirements)

| Requirement | How Met |
|-------------|---------|
| ID consistency | All 11 source files reference the same pair of new program IDs |
| Build success | `anchor build` succeeds as the acceptance criterion |
| Test compatibility | LiteSVM tests load `.so` files against correct program IDs |
| Script functionality | Init and utility scripts use correct program IDs for PDA derivation |
| Cross-program CPI | `token_minter` CPI validation uses oracle's updated `declare_id!` via the Cargo dependency `sol-usd-oracle = { path = "../sol_usd_oracle", features = ["cpi"] }` |
| No new dependencies | No new crates, packages, or tools required |
| Documentation accuracy | `CLAUDE.md` and `.env.example` reflect current program IDs |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `anchor keys sync` fails or is unavailable in Anchor 0.32.1 | Low | Low | Fall back to manual extraction: `solana-keygen pubkey program/target/deploy/<name>-keypair.json`, then manually update all files |
| Missed reference -- old IDs exist in files not identified by research | Low | Medium | After all updates, run a codebase-wide grep for both old ID strings to confirm zero remaining references (excluding historical docs like `docs/idea.md` and `docs/phase/phase-4.md`) |
| PDA addresses change after program ID update | Expected | None | This is correct behavior. The init script outputs the new PDA. Backend `.env` must be regenerated after `make init`. |
| `backend/.env` becomes stale after ID update | Medium | Medium | Document in implementation steps that the student must re-run `make init` and update `backend/.env` with the new `ORACLE_STATE_PUBKEY` |
| Build cache issues after `declare_id!` change | Low | Low | Run `anchor clean` before `anchor build` if build fails unexpectedly |

---

## Deviations to Fix

**DEVIATION (from research):** Existing keypair files already contain different keys from source code. The deploy keypair files at `program/target/deploy/` produce public keys (`GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3` and `3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn`) that differ from the hardcoded IDs in all source files. This is the exact mismatch ML-4 is meant to fix. Running `anchor keys sync` synchronizes source to match existing keypairs. No corrective action needed beyond the planned implementation steps.

**SCOPE EXTENSION:** The phase-4 spec lists only 4 locations for update, but old IDs appear in 11 source files total. Leaving the additional 7 files stale causes runtime failures in tests (`make test`), initialization (`make init`), and migration (`anchor migrate`). The plan updates all 11 locations to deliver a working system, which aligns with the PRD's own risk assessment: "leaving the others stale will cause runtime failures in tests, scripts, and backend."

---

## Implementation Steps

### Step 1: Run `anchor keys sync`

**Working directory:** `program/`

```bash
cd program && anchor keys sync
```

This reads the existing keypair files and updates:
- `program/programs/sol_usd_oracle/src/lib.rs` -- `declare_id!` macro
- `program/programs/token_minter/src/lib.rs` -- `declare_id!` macro
- `program/Anchor.toml` -- `[programs.localnet]` and `[programs.devnet]` sections

**Verification:** After running, extract the new pubkeys for use in subsequent steps:
```bash
solana-keygen pubkey target/deploy/sol_usd_oracle-keypair.json
solana-keygen pubkey target/deploy/token_minter-keypair.json
```

**Fallback** (if `anchor keys sync` fails): Use the extracted pubkeys and manually update the three files above.

### Step 2: Update `frontend/app/config.ts`

Replace old IDs on lines 2-3:
- `ORACLE_PROGRAM_ID`: replace `"4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU"` with the new oracle pubkey
- `MINTER_PROGRAM_ID`: replace `"E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE"` with the new minter pubkey

### Step 3: Update additional source files

For each file, replace old program ID strings with the corresponding new pubkeys:

1. **`program/scripts/init-local.js`** (lines 22-23): Both `ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID` `new PublicKey(...)` constructors
2. **`program/scripts/get-oracle-pda.js`** (line 6): `ORACLE_PROGRAM_ID` `new PublicKey(...)` constructor
3. **`program/tests/oracle.litesvm.ts`** (line 16): `ORACLE_PROGRAM_ID` `new PublicKey(...)` constructor
4. **`program/tests/minter.litesvm.ts`** (lines 24-25): Both `ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID` `new PublicKey(...)` constructors
5. **`program/migrations/deploy.ts`** (lines 10-11): Both `ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID` `new PublicKey(...)` constructors
6. **`backend/.env.example`** (lines 10-11): Both `ORACLE_PROGRAM_ID=` and `MINTER_PROGRAM_ID=` values

### Step 4: Update `CLAUDE.md`

Replace old program IDs on line 106:
- `sol_usd_oracle = 4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU` with the new oracle pubkey
- `token_minter = E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE` with the new minter pubkey

### Step 5: Verify -- `anchor build`

```bash
cd program && anchor build
```

Acceptance criterion: build completes successfully with exit code 0.

### Step 6: Verify -- codebase-wide grep for old IDs

```bash
grep -r "4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU" --include="*.rs" --include="*.ts" --include="*.js" --include="*.toml" --include="*.md" --include="*.env*" .
grep -r "E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE" --include="*.rs" --include="*.ts" --include="*.js" --include="*.toml" --include="*.md" --include="*.env*" .
```

Expected: old IDs appear only in historical documentation files (`docs/idea.md`, `docs/phase/phase-4.md`, `docs/prd/ML-4.prd.md`, `docs/research/ML-4.md`) where they serve as reference for what was replaced. Zero matches in any source code, configuration, or active documentation files.

### Step 7: Verify -- tests pass

```bash
cd program && node --import tsx/esm node_modules/.bin/mocha -t 1000000 "tests/**/*.ts"
```

Both LiteSVM test suites (`oracle.litesvm.ts` and `minter.litesvm.ts`) should pass with the updated program IDs.

---

## Open Questions

None. The requirements are fully specified by the phase-4 document and the PRD. The only scope decision (updating 4 vs 11 locations) has been resolved in favor of updating all 11 source files to deliver a functioning system, consistent with the PRD's own risk assessment.
