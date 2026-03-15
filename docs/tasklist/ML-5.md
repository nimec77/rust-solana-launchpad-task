# Tasklist: ML-5 — Local End-to-End Cycle

**Ticket:** ML-5
**Phase:** 5 (Iteration 5)
**Status:** IMPLEMENT_STEP_OK

---

## Context

All program bugs are fixed (ML-1 through ML-3) and own program IDs are generated and propagated (ML-4). This iteration brings every component online on a local validator and verifies the complete token minting flow end-to-end: validator, on-chain programs, backend price updater/event listener, and the Remix frontend. There is one configuration deviation to fix (stale oracle program ID in `.env.example` PDA comment) before execution. No source code changes are required.

---

## Tasks

- [x] **Task 1: Fix `.env.example` stale oracle program ID in PDA comment (Deviation Fix)**
  - **File:** `backend/.env.example`
  - Update line 14: replace the old oracle program ID `88GBkKvZbhtTcXy2tpwHQSVxHqPqurQbjKW1nLzER84c` in the PDA derivation comment with the correct ID `GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3`.
  - **Acceptance Criteria:**
    - The PDA derivation comment on line 14 of `backend/.env.example` uses `GSwL85d5Pvvh8HreS3D7d6X3NmCZySbKmZBebF3oqCk3` as the fallback oracle program ID
    - No stale/old program IDs remain anywhere in `backend/.env.example`

- [x] **Task 2: Start local validator with Metaplex support (Task 5.1)**
  - Run `make validator-metaplex` in a dedicated terminal to start `solana-test-validator` with the Metaplex Token Metadata program (`metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s`) cloned from mainnet.
  - **Acceptance Criteria:**
    - Validator starts without errors and logs genesis hash
    - Validator is accessible at `http://127.0.0.1:8899` (confirmed via `solana cluster-version --url http://127.0.0.1:8899`)

- [x] **Task 3: Deploy and initialize both programs (Task 5.2)**
  - Run `make deploy && make init` to build, deploy, and initialize both `sol_usd_oracle` and `token_minter` on localnet.
  - Capture the `ORACLE_STATE_PUBKEY` value printed by the init script output for use in Task 4.
  - **Acceptance Criteria:**
    - `anchor build` completes successfully (exit code 0)
    - `anchor deploy` deploys both programs to localnet without errors
    - `make init` initializes the oracle (price=$120) and the minter (fee=$5, treasury=wallet) without errors
    - `ORACLE_STATE_PUBKEY=<pda>` is printed in the init output

- [x] **Task 4: Configure `backend/.env` with oracle state PDA (Task 5.3)**
  - Set `ORACLE_STATE_PUBKEY` in `backend/.env` to the actual PDA value captured from `make init` output (replacing the placeholder `<PDA from "oracle_state" seed>`).
  - Verify all other required variables have correct values: `SOLANA_RPC_HTTP`, `SOLANA_RPC_WS`, `ORACLE_PROGRAM_ID`, `MINTER_PROGRAM_ID`, `BACKEND_KEYPAIR_PATH`.
  - **Acceptance Criteria:**
    - `ORACLE_STATE_PUBKEY` in `backend/.env` contains a valid base58 Solana public key (not a placeholder)
    - All required environment variables in `backend/.env` have valid, non-placeholder values matching the deployed programs

- [x] **Task 5: Start backend service (Task 5.4a)**
  - Run `make backend` in a dedicated terminal to start the price updater and event listener tasks.
  - **Acceptance Criteria:**
    - Backend starts without errors or panics
    - Price updater logs a successful `update_price` transaction signature on startup
    - Event listener logs that it has subscribed to minter program logs via WebSocket

- [x] **Task 6: Start frontend service (Task 5.4b)**
  - Run `make frontend` in a dedicated terminal to start the Remix dev server.
  - **Acceptance Criteria:**
    - Remix dev server starts on `http://localhost:7001` without errors
    - Opening `http://localhost:7001` in a browser shows the terminal-styled UI
    - Oracle price and mint fee are displayed in the UI (polled from on-chain PDAs every 5 seconds)

- [x] **Task 7: Mint a token via the UI (Task 5.5)**
  - Open `http://localhost:7001` in a browser, connect a Solana wallet configured for localnet (`http://127.0.0.1:8899`), airdrop SOL to the wallet if needed, fill in the token minting form (name, symbol, URI, decimals, supply), enable Metaplex checkbox, submit the transaction, and approve in the wallet.
  - **Acceptance Criteria:**
    - Frontend displays a success message with the transaction signature and explorer link
    - Backend terminal logs a `TokenCreated` event in JSON format (containing creator, mint, decimals, initial_supply, fee_lamports, sol_usd_price, slot fields)

- [x] **Task 8: Mark Iteration 5 complete in project documentation**
  - Update `docs/tasklist.md` to check all 5 sub-tasks in Iteration 5 and change the status indicator to complete.
  - Update `docs/phase/phase-5.md` to check all task checkboxes (`- [x]`).
  - **Acceptance Criteria:**
    - All five sub-tasks (5.1 through 5.5) are marked complete in `docs/phase/phase-5.md`
    - Iteration 5 status is updated in `docs/tasklist.md`
