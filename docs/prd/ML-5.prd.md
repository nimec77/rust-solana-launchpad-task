# PRD: ML-5 — Local End-to-End Cycle

**Status:** PRD_READY
**Ticket:** ML-5
**Phase:** 5 (Iteration 5)

---

## Context / Idea

**Arguments:** ML-5 docs/phase/phase-5.md

With all program bugs fixed (Iterations 1-3) and fresh program IDs generated (Iteration 4), the project is ready for a full local integration test. This iteration brings all components together — local Solana validator, both on-chain programs, the Rust backend, and the Remix frontend — to execute the complete token minting flow end-to-end on a local validator.

### Source: docs/phase/phase-5.md

> **Goal:** Run the full stack locally — validator, programs, backend, and frontend — and mint a token via the UI.
>
> **Tasks:**
> - 5.1 Start validator: `make validator-metaplex`
> - 5.2 Deploy & init: `make deploy && make init`
> - 5.3 Configure `backend/.env` (copy `.env.example`, set `ORACLE_STATE_PUBKEY` from init output)
> - 5.4 Run backend + frontend: `make backend` + `make frontend`
> - 5.5 Mint a token via UI (connect wallet, fill form, submit)
>
> **Acceptance Criteria:**
> Test: backend logs show `TokenCreated` event JSON
>
> **Dependencies:**
> - Iteration 4 complete (own program IDs generated and deployed)
>
> **Implementation Notes:**
> - Use `make validator-metaplex` to include Metaplex metadata program for token display
> - Copy `ORACLE_STATE_PUBKEY` from `make init` output into `backend/.env`
> - Frontend runs on port 7001

---

## Goals

1. **Validate the full local stack** — Confirm that the validator, both deployed programs, the backend price updater/event listener, and the frontend UI all work together as a cohesive system.
2. **Execute a complete token mint via the UI** — A user connects a wallet in the browser, fills in token details (name, ticker, URI), submits the minting transaction, and receives SPL tokens in their wallet.
3. **Verify event pipeline** — Confirm that the backend's WebSocket event listener captures and logs the `TokenCreated` event emitted by the `token_minter` program after a successful mint.
4. **Validate backend configuration** — Ensure `backend/.env` is correctly configured with localnet RPC endpoints, the oracle state PDA, and the admin keypair so that the price updater can submit `update_price` transactions to the oracle.

---

## User Stories

1. **As a student**, I want to run the entire stack locally so that I can see the full lifecycle of minting a token — from price feed updates to on-chain transaction to event logging.
2. **As a student**, I want to mint a token through the browser UI so that I can verify my program fixes and configuration changes produce a working end-to-end flow.
3. **As a developer**, I want the backend to log `TokenCreated` events so that I can confirm the event listener and the minter program's CPI/emit logic are functioning correctly.

---

## Scenarios

### Scenario 1: Start local validator with Metaplex support

- **Given** `solana-test-validator` is not running
- **When** the student runs `make validator-metaplex`
- **Then** a local validator starts with the Metaplex Token Metadata program (`metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s`) cloned from mainnet
- **And** the validator is accessible at `http://127.0.0.1:8899` (HTTP) and `ws://127.0.0.1:8900` (WebSocket)

### Scenario 2: Deploy and initialize both programs

- **Given** the local validator is running
- **When** the student runs `make deploy && make init`
- **Then** both `sol_usd_oracle` and `token_minter` are built and deployed to localnet
- **And** the oracle is initialized with a default price (e.g., $120) and the minter is initialized with a fee (e.g., $5) and treasury address
- **And** the init script outputs `ORACLE_STATE_PUBKEY` which is the PDA derived from seed `"oracle_state"`

### Scenario 3: Configure backend environment

- **Given** both programs are deployed and initialized
- **When** the student copies `backend/.env.example` to `backend/.env` and sets `ORACLE_STATE_PUBKEY` from the init output
- **Then** `backend/.env` contains valid localnet RPC URLs (`http://127.0.0.1:8899`, `ws://127.0.0.1:8900`), the correct `ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID`, the oracle state PDA, and the path to the admin keypair

### Scenario 4: Run backend and frontend concurrently

- **Given** `backend/.env` is configured and the validator is running with deployed programs
- **When** the student runs `make backend` in one terminal and `make frontend` in another
- **Then** the backend starts two tokio tasks: price updater (polling price and submitting `update_price` to oracle) and event listener (WebSocket subscription to minter program logs)
- **And** the frontend dev server starts on `http://localhost:7001` and polls oracle + minter PDA state every 5 seconds

### Scenario 5: Mint a token via the UI

- **Given** the backend and frontend are running, and the local validator has both programs deployed and initialized
- **When** the student opens `http://localhost:7001`, connects a Solana wallet (e.g., Phantom), fills in token name/ticker/URI, and submits the mint transaction
- **Then** the `token_minter` program computes the fee in lamports from the oracle price, transfers the fee to treasury, mints SPL tokens to the user's ATA, creates Metaplex metadata, and emits a `TokenCreated` event
- **And** the backend logs the `TokenCreated` event as JSON output
- **And** the frontend displays the transaction signature with an explorer link

### Scenario 6: Backend logs TokenCreated event (acceptance criteria)

- **Given** a token mint transaction has been submitted and confirmed on localnet
- **When** the backend's event listener processes the transaction logs
- **Then** the backend outputs a JSON object containing the `TokenCreated` event data (mint address, name, symbol, creator, etc.)

---

## Metrics

| Metric | Target |
|--------|--------|
| Local validator starts with Metaplex program | Yes |
| Both programs deploy and initialize without errors | Yes |
| Backend starts and submits `update_price` transactions | Yes |
| Frontend loads at `http://localhost:7001` and displays oracle price | Yes |
| Token mint transaction succeeds via UI | Yes |
| Backend logs show `TokenCreated` event JSON | Yes (acceptance criteria) |

---

## Constraints

1. **Iteration 4 must be complete** — Own program IDs must be generated and all references updated before deployment will succeed.
2. **`make validator-metaplex` required for metadata display** — The Metaplex Token Metadata program must be cloned from mainnet for the minter's `CreateMetadataAccountsV3` CPI to succeed. Without it, minting will fail.
3. **`ORACLE_STATE_PUBKEY` must be set manually** — The PDA address is output by `make init` and must be copied into `backend/.env`. This value is deterministic (derived from `"oracle_state"` seed + oracle program ID) but must match the deployed oracle program's ID.
4. **Wallet with SOL required** — The local wallet (`~/.config/solana/id.json`) needs SOL on localnet (the test validator airdrops SOL automatically to the default keypair). The browser wallet (Phantom) also needs localnet SOL to pay mint fees.
5. **Multiple terminal sessions needed** — The validator, backend, and frontend each run as long-lived processes requiring separate terminals (or background processes).
6. **Frontend runs on port 7001** — As specified in the phase document and Remix config.
7. **Backend program IDs in `.env` must match deployed programs** — The `ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID` in `backend/.env` must match the IDs in `program/Anchor.toml` and `frontend/app/config.ts`.

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `backend/.env` not updated from `.env.example` — stale program IDs or missing `ORACLE_STATE_PUBKEY` | High | High | The `.env.example` still contains old program IDs (`4cuvLFF...`, `E5erG...`). After copying, the student must update `ORACLE_PROGRAM_ID`, `MINTER_PROGRAM_ID`, and `ORACLE_STATE_PUBKEY` to match their locally generated values from Iteration 4. |
| Metaplex program not available — minting fails with program error | Medium | High | Must use `make validator-metaplex` instead of plain `make validator`. The token minter's `mint_token` instruction invokes Metaplex `CreateMetadataAccountsV3`, which requires the metadata program to be deployed. |
| Browser wallet not configured for localnet | Medium | Medium | Phantom/Solflare must be configured to use `http://127.0.0.1:8899` as the custom RPC endpoint. The student also needs to airdrop SOL to the browser wallet address on localnet. |
| Port 7001 already in use | Low | Low | Use `make kill-frontend` to free the port before starting the frontend. |
| Backend keypair is not the oracle admin | Medium | High | The `BACKEND_KEYPAIR_PATH` in `.env` must point to the same keypair used during `make init` (which sets the admin). By default this is `~/.config/solana/id.json`. |
| WebSocket connection drops or events are missed | Low | Medium | Restart the backend. The event listener uses a persistent WebSocket subscription to the minter program's logs. |

---

## Open Questions

None. The requirements are fully specified by the phase document. This iteration is an integration/validation step with no code changes required — it exercises the fixes and configuration from Iterations 1-4. The only manual steps are environment configuration (`backend/.env`) and wallet setup in the browser.

---

## Implementation Reference

### Task 5.1: Start validator

```bash
make validator-metaplex
```

This runs `solana-test-validator --clone-upgradeable-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s --url https://api.mainnet-beta.solana.com`, which starts a local validator with the Metaplex Token Metadata program cloned from mainnet.

### Task 5.2: Deploy and initialize

```bash
make deploy && make init
```

`make deploy` runs `anchor build` then `anchor deploy --provider.cluster localnet`, deploying both programs. `make init` runs `node scripts/init-local.js`, which initializes the oracle (with a default price of $120, 6 decimals) and the minter (with a $5 fee and the wallet as treasury). The init script outputs the `ORACLE_STATE_PUBKEY` PDA address.

### Task 5.3: Configure backend/.env

```bash
cp backend/.env.example backend/.env
```

Then edit `backend/.env`:
- Set `ORACLE_PROGRAM_ID` and `MINTER_PROGRAM_ID` to the new program IDs from Iteration 4 (matching `program/Anchor.toml` and `frontend/app/config.ts`)
- Set `ORACLE_STATE_PUBKEY` to the PDA address output by `make init`
- Verify `BACKEND_KEYPAIR_PATH` points to the correct admin keypair (default: `~/.config/solana/id.json`)
- RPC URLs should already be correct for localnet (`http://127.0.0.1:8899` / `ws://127.0.0.1:8900`)

### Task 5.4: Run backend and frontend

In separate terminal windows:

```bash
make backend    # Terminal 2: starts price updater + event listener
make frontend   # Terminal 3: starts Remix dev server on port 7001
```

### Task 5.5: Mint a token via UI

1. Open `http://localhost:7001` in a browser
2. Connect a Solana wallet (Phantom or Solflare) configured for localnet (`http://127.0.0.1:8899`)
3. Ensure the connected wallet has SOL (airdrop via `solana airdrop 2 <wallet-address>` if needed)
4. Fill in the token minting form (name, ticker/symbol, metadata URI)
5. Submit the transaction and approve in the wallet
6. Verify: the frontend shows a success message with transaction signature
7. Verify: the backend terminal logs a `TokenCreated` event in JSON format
