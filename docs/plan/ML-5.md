# Plan: ML-5 — Local End-to-End Cycle

**Ticket:** ML-5
**Phase:** 5 (Iteration 5)
**Status:** PLAN_APPROVED

---

## Components

ML-5 is an integration/validation iteration. Unlike ML-1 through ML-4, it requires **no source code changes**. The work consists of executing a sequence of operational steps that bring every component online and verifying the system works end-to-end. There is one configuration deviation that must be fixed before execution (`.env.example` has stale program IDs).

### 1. Local Validator with Metaplex (Task 5.1)

**Command:** `make validator-metaplex`

Starts `solana-test-validator` with the Metaplex Token Metadata program (`metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s`) cloned from mainnet. This is required because `token_minter::mint_token` invokes `CreateMetadataAccountV3CpiBuilder` when a non-empty token name is provided. Without the Metaplex program on the validator, minting with metadata will fail.

**Prerequisite:** Internet access for mainnet clone. Ports 8899 (HTTP) and 8900 (WS) must be free.

### 2. Program Deployment and Initialization (Task 5.2)

**Commands:** `make deploy && make init`

- `make deploy` runs `anchor build` + `anchor deploy --provider.cluster localnet`, deploying both `sol_usd_oracle` and `token_minter`.
- `make init` runs `program/scripts/init-local.js`, which:
  1. Derives oracle PDA from seed `"oracle_state"` + oracle program ID
  2. Derives minter PDA from seed `"minter_config"` + minter program ID
  3. Initializes the oracle (admin = payer wallet), then calls `update_price` with $120.00 (`120_000_000`)
  4. Initializes the minter (treasury = payer wallet, fee = $5.00, oracle state PDA, oracle program ID)
  5. Outputs `ORACLE_STATE_PUBKEY=<pda>` twice (before and after init)

Both initialization calls are idempotent -- the script catches "already in use" errors.

**Critical output:** The `ORACLE_STATE_PUBKEY` value printed by the init script must be captured for Task 5.3.

### 3. Backend `.env` Configuration (Task 5.3)

**File:** `backend/.env` (gitignored)

The existing `backend/.env` already has correct program IDs (updated during ML-4). Only `ORACLE_STATE_PUBKEY` needs to be set from the `make init` output. The current value is the placeholder string `<PDA from "oracle_state" seed>`.

Required environment state for the backend to function:

| Variable | Required Value | Current State |
|----------|---------------|---------------|
| `SOLANA_RPC_HTTP` | `http://127.0.0.1:8899` | Correct |
| `SOLANA_RPC_WS` | `ws://127.0.0.1:8900` | Correct |
| `ORACLE_PROGRAM_ID` | `GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3` | Correct |
| `MINTER_PROGRAM_ID` | `3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn` | Correct |
| `ORACLE_STATE_PUBKEY` | PDA from `make init` output | **MUST UPDATE** |
| `BACKEND_KEYPAIR_PATH` | `~/.config/solana/id.json` | Correct |
| `PRICE_POLL_INTERVAL_SEC` | `600` | Correct |

### 4. Backend Service (Task 5.4a)

**Command:** `make backend` (= `cd backend && cargo run`)

Starts two concurrent `tokio` tasks:
1. **Price updater** -- Immediately submits `update_price` tx on startup, then every `PRICE_POLL_INTERVAL_SEC` (600s). Uses Binance live price by default, or `MOCK_PRICE` if set.
2. **Event listener** -- Opens WebSocket subscription to minter program logs. Parses `TokenCreated` events via regex and outputs JSON to stdout.

### 5. Frontend Service (Task 5.4b)

**Command:** `make frontend` (= `make kill-frontend && cd frontend && npm run dev`)

Starts Remix dev server on port 7001. The frontend:
- Connects to `http://127.0.0.1:8899` (localnet default)
- Polls oracle PDA (price at byte offset 40) and minter PDA (fee at byte offset 72) every 5 seconds
- Provides wallet connection (Phantom, Solflare) and token minting form

### 6. Token Minting via UI (Task 5.5)

End-to-end minting flow:

1. Open `http://localhost:7001` in browser
2. Connect a Solana wallet configured for localnet RPC (`http://127.0.0.1:8899`)
3. Ensure wallet has SOL (airdrop via `solana airdrop 2 <wallet-address>` if needed)
4. Fill token form: name (up to 32 chars), symbol (up to 10 chars), metadata URI, decimals, initial supply
5. Enable Metaplex checkbox for on-chain metadata
6. Submit transaction and approve in wallet
7. Frontend shows tx signature with explorer link
8. Backend logs `TokenCreated` event JSON to stdout

**Acceptance criterion:** Backend logs show `TokenCreated` event JSON.

---

## API Contract

No API changes. ML-5 exercises the existing API contracts established in ML-1 through ML-4:

- **Oracle program:** `initialize_oracle` + `update_price` instructions (used by init script and backend)
- **Minter program:** `initialize_minter` + `mint_token` instructions (used by init script and frontend)
- **Cross-program CPI:** `token_minter` reads oracle price via account validation during `mint_token`
- **Backend event listener:** Regex-based parsing of `TokenCreated` event from minter program logs
- **Frontend instruction builder:** Manual Borsh encoding of `mint_token` instruction with discriminator `[172, 137, 183, 14, 207, 110, 234, 56]`

---

## Data Flows

```
[Task 5.1] solana-test-validator + Metaplex clone
    |
    | ports 8899 (HTTP), 8900 (WS)
    v
[Task 5.2] anchor build + anchor deploy --provider.cluster localnet
    |
    | deploys sol_usd_oracle.so + token_minter.so
    v
[Task 5.2] node scripts/init-local.js
    |
    | init oracle (price=$120) + init minter (fee=$5, treasury=wallet)
    | outputs ORACLE_STATE_PUBKEY=<pda>
    v
[Task 5.3] backend/.env -- set ORACLE_STATE_PUBKEY
    |
    v
[Task 5.4] backend (cargo run)                   frontend (npm run dev)
    |   price updater: poll -> update_price tx         |
    |   event listener: WS subscribe minter logs       | polls oracle+minter PDA every 5s
    |                                                  | builds mint_token instruction
    v                                                  v
[Task 5.5] User mints token via browser UI
    |
    | frontend sends mint_token tx -> token_minter program
    | program: compute fee -> transfer SOL -> mint SPL -> create metadata -> emit event
    |
    v
Backend captures TokenCreated event via WS logs -> prints JSON to stdout
```

---

## NFR (Non-Functional Requirements)

| Requirement | How Met |
|-------------|---------|
| All components start without errors | Validator, deploy, init, backend, and frontend each produce expected output with no crashes |
| Backend connects to correct oracle PDA | `ORACLE_STATE_PUBKEY` in `.env` matches the PDA derived from the deployed oracle program ID |
| Backend submits `update_price` transactions | Price updater task logs successful tx signatures to stdout |
| Frontend displays live oracle price | Polls oracle PDA at byte offset 40 every 5s and renders the price |
| Token minting succeeds end-to-end | Mint transaction confirmed on localnet, SPL tokens appear in user's ATA |
| Event pipeline works | Backend WebSocket subscription captures `TokenCreated` event and outputs JSON |
| No code changes required | This iteration validates existing code from ML-1 through ML-4; only `.env` configuration is modified |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `ORACLE_STATE_PUBKEY` not updated in `backend/.env` | HIGH | HIGH | Backend will crash on startup with parse error. Must copy actual PDA value from `make init` output before running `make backend`. |
| `.env.example` has stale program IDs | HIGH | MEDIUM | If student copies `.env.example` to `.env`, backend will use wrong program IDs. See "Deviations to Fix" section -- `.env.example` should be updated. Alternatively, keep existing `.env` and only update `ORACLE_STATE_PUBKEY`. |
| Metaplex program clone requires internet | MEDIUM | HIGH | `make validator-metaplex` clones from mainnet. If offline, use `make validator` and disable Metaplex checkbox in UI. Minting still works but without on-chain metadata. |
| Browser wallet not configured for localnet | MEDIUM | MEDIUM | Phantom/Solflare must use `http://127.0.0.1:8899` as custom RPC. Backpack wallet recommended for localnet per `docs/vision.md`. |
| Admin keypair mismatch | MEDIUM | HIGH | `BACKEND_KEYPAIR_PATH` must point to same keypair used during `make init` (the oracle admin). Default `~/.config/solana/id.json` is consistent between `Anchor.toml` and `.env`. |
| Backend price updater fails to reach Binance | LOW | LOW | Uncomment `MOCK_PRICE=120000000` in `.env` for offline testing. Init script already sets initial price, so event listener works regardless. |
| Port conflicts (7001 or 8899/8900) | LOW | LOW | `make frontend` includes `make kill-frontend` for port 7001. Stop any existing `solana-test-validator` for 8899/8900. |
| Browser wallet has no SOL on localnet | MEDIUM | MEDIUM | Airdrop SOL to browser wallet address: `solana airdrop 2 <browser-wallet-address>`. |

---

## Deviations to Fix

### DEVIATION: `backend/.env.example` contains old program IDs

**Source:** Research document, confirmed by file inspection.

`backend/.env.example` lines 10-11 still reference the original author's program IDs:
```
ORACLE_PROGRAM_ID=4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU
MINTER_PROGRAM_ID=E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE
```

The correct IDs (established in ML-4) are:
```
ORACLE_PROGRAM_ID=GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3
MINTER_PROGRAM_ID=3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn
```

**Impact:** The PRD explicitly documents this risk: "The `.env.example` still contains old program IDs. After copying, the student must update `ORACLE_PROGRAM_ID`, `MINTER_PROGRAM_ID`, and `ORACLE_STATE_PUBKEY`." If a student follows Task 5.3 literally ("copy `.env.example`"), they get wrong program IDs and the backend fails.

**Fix:** Update `backend/.env.example` lines 10-11 to use the correct program IDs. This is a minor configuration template fix (not a code change) that removes a documented UX hazard and makes Task 5.3 instructions safe to follow literally. The ML-4 plan already identified this file as location #10 for update; it was updated in `backend/.env` but the example template was left stale.

---

## Implementation Steps

### Step 0: Fix `.env.example` (Deviation Fix)

Update `backend/.env.example` lines 10-11 to replace old program IDs with the correct ones from ML-4:
- `ORACLE_PROGRAM_ID=4cuvLFFqhaKnTHfeq2FtTUvgudRSe7wq982fA9PBUqBU` -> `ORACLE_PROGRAM_ID=GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3`
- `MINTER_PROGRAM_ID=E5erGzaxgCwHqH7RjLXLGWziXj8CXpyN7zW6BRodfFnE` -> `MINTER_PROGRAM_ID=3eyjeU9ZaU2aVySwSNBQWPtkZnN2NtUNWDmpBfsMj4kn`

Also update the old oracle program ID on line 14 in the PDA derivation comment (`88GBkKvZbhtTcXy2tpwHQSVxHqPqurQbjKW1nLzER84c` -> `GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3`).

### Step 1: Start Local Validator (Task 5.1)

```bash
make validator-metaplex
```

**Runs in:** Dedicated terminal (long-lived process).

**Verification:** Validator starts without errors and logs `genesis hash: <hash>`. Confirm accessibility via:
```bash
solana cluster-version --url http://127.0.0.1:8899
```

**Note:** If a previous `test-ledger/` directory exists, the validator reuses it. To start fresh, delete `test-ledger/` first.

### Step 2: Deploy and Initialize (Task 5.2)

```bash
make deploy && make init
```

**Verification points:**
1. `anchor build` compiles both programs successfully
2. `anchor deploy` deploys both programs to localnet (logs program IDs and deploy signatures)
3. `make init` initializes oracle (price=$120) and minter (fee=$5, treasury=wallet)
4. Init script outputs `ORACLE_STATE_PUBKEY=<pda>` -- **capture this value**

### Step 3: Configure Backend `.env` (Task 5.3)

Set the `ORACLE_STATE_PUBKEY` in `backend/.env` to the value from Step 2 output. Replace line 16:
```
ORACLE_STATE_PUBKEY=<PDA from "oracle_state" seed>
```
with:
```
ORACLE_STATE_PUBKEY=<actual PDA value from make init output>
```

Optionally, uncomment `MOCK_PRICE=120000000` on line 25 for offline-capable local testing.

**Verification:** All required variables in `backend/.env` have valid non-placeholder values.

### Step 4: Start Backend (Task 5.4a)

```bash
make backend
```

**Runs in:** Dedicated terminal (long-lived process).

**Verification:**
1. Backend starts without errors
2. Price updater logs a successful `update_price` tx signature on startup
3. Event listener logs that it has subscribed to minter program logs

### Step 5: Start Frontend (Task 5.4b)

```bash
make frontend
```

**Runs in:** Dedicated terminal (long-lived process).

**Verification:**
1. Remix dev server starts on `http://localhost:7001`
2. Opening the URL in a browser shows the terminal-styled UI
3. Oracle price and mint fee are displayed (polled from chain)

### Step 6: Mint a Token via UI (Task 5.5)

1. Open `http://localhost:7001` in a browser
2. Connect a Solana wallet configured for localnet (`http://127.0.0.1:8899`)
3. Ensure the wallet has SOL: `solana airdrop 2 <wallet-address>` if needed
4. Fill in the token minting form:
   - Token name (e.g., "TestToken")
   - Symbol (e.g., "TEST")
   - Image URL / metadata URI (can be any valid URL or empty)
   - Decimals (e.g., 6)
   - Initial supply (e.g., 1000000)
5. Enable the Metaplex checkbox for on-chain metadata
6. Submit the transaction and approve in the wallet

**Verification (acceptance criteria):**
1. Frontend shows success message with transaction signature and explorer link
2. **Backend terminal logs a `TokenCreated` event in JSON format** -- this is the primary acceptance criterion

Expected backend JSON output format:
```json
{
  "creator": "<wallet-pubkey>",
  "mint": "<new-mint-pubkey>",
  "decimals": 6,
  "initial_supply": 1000000,
  "fee_lamports": <computed>,
  "sol_usd_price": <current-price>,
  "slot": <slot-number>
}
```

### Step 7: Mark Iteration Complete

After successful verification:
1. Update `docs/tasklist.md` -- check all 5 sub-tasks in Iteration 5 and change status from `⬜` to checkmark
2. Update `docs/phase/phase-5.md` -- check all task checkboxes

---

## Open Questions

None. The requirements are fully specified by the phase-5 document and the PRD. ML-5 is a validation/integration iteration that exercises the work from ML-1 through ML-4. The only file change is the `.env.example` deviation fix (stale program IDs from before ML-4).
