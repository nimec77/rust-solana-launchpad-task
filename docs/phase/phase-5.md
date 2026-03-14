# Iteration 5: Local End-to-End Cycle

**Goal:** Run the full stack locally — validator, programs, backend, and frontend — and mint a token via the UI.

## Tasks

- [ ] 5.1 Start validator: `make validator-metaplex`
- [ ] 5.2 Deploy & init: `make deploy && make init`
- [ ] 5.3 Configure `backend/.env` (copy `.env.example`, set `ORACLE_STATE_PUBKEY` from init output)
- [ ] 5.4 Run backend + frontend: `make backend` + `make frontend`
- [ ] 5.5 Mint a token via UI (connect wallet, fill form, submit)

## Acceptance Criteria

**Test:** backend logs show `TokenCreated` event JSON

## Dependencies

- Iteration 4 complete (own program IDs generated and deployed)

## Implementation Notes

- Use `make validator-metaplex` to include Metaplex metadata program for token display
- Copy `ORACLE_STATE_PUBKEY` from `make init` output into `backend/.env`
- Frontend runs on port 7001
