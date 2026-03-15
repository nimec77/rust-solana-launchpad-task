# Vision — Solana Mini-Launchpad

## Technologies

**On-chain programs**

| Technology | Version | Purpose |
|---|---|---|
| Rust | 2021 edition | Smart contract language |
| Anchor | 0.32.1 | Solana program framework (PDA, CPI, serialization) |
| SPL Token | via anchor-spl 0.32.1 | Token creation and minting |
| Metaplex Token Metadata | mpl-token-metadata 5.1.1 | Token metadata (name, symbol, URI) |
| Solana SDK | 2.1.13 | Solana base primitives |

**Backend**

| Technology | Version | Purpose |
|---|---|---|
| Rust | 2021 edition | Service language |
| Tokio | 1.40.0 | Async runtime (multi-threaded) |
| solana-client / solana-sdk | 2.1.13 | RPC and WebSocket blockchain interaction |
| reqwest | 0.11.27 | HTTP client (Binance API price request) |
| tracing / tracing-subscriber | — | Structured logging |
| dotenvy | 0.15.7 | Configuration loading from `.env` |
| regex | 1.11.1 | Parsing events from program logs |

**Frontend**

| Technology | Version | Purpose |
|---|---|---|
| Remix | 2.9.2 | React framework (SSR + SPA) |
| React | 18.3.1 | UI library |
| TypeScript | 5.6.3 | Type safety |
| @solana/web3.js | 1.98.4 | Solana interaction |
| Wallet Adapter | 0.15.39 | Wallet connection (Phantom, Solflare; Backpack recommended for localnet) |
| Vitest | 4.1.0 | Frontend unit tests |

- Frontend uses manual Borsh serialization (no Anchor client)

**Testing**

| Technology | Purpose |
|---|---|
| LiteSVM + ts-mocha/chai | Program tests (no validator needed) |
| Vitest | Frontend tests |
| cargo test | Backend tests |

**Tools and infrastructure**

| Tool | Purpose |
|---|---|
| Anchor CLI | Build, deploy, test programs |
| Solana CLI | Cluster management, keys, airdrop |
| Makefile | Command orchestration (build, deploy, init, test) |
| Yarn | Program/test dependencies |
| npm | Frontend dependencies |
| Cargo | Backend build |
| solana-test-validator | Local node for development |

## Development Principles

- **KISS** — simplest solution that works; no premature abstractions
- **Single-file services** — backend is one `main.rs`, frontend config is one `config.ts`; don't split until complexity demands it
- **Manual over magic** — frontend uses hand-written Borsh encoding instead of generated Anchor clients; understand what you build
- **Test without infrastructure** — LiteSVM for programs (no running validator), unit tests for backend, Vitest for frontend
- **Makefile as entry point** — all common operations are one `make` command away
- **Fail loudly** — programs reject invalid input (zero price, zero supply); backend logs errors with `tracing`; no silent failures
- **Fixed-point arithmetic** — all money math uses integer `u64` with 6 decimal places; no floats anywhere
- **Configuration via environment** — `.env` for backend; no complex config frameworks
- **Localnet-first development** — develop and test locally before devnet; same code path for both

## Project Structure

```
rust-solana-launchpad-task/
├── program/                          # Anchor workspace
│   ├── programs/
│   │   ├── sol_usd_oracle/src/       # Oracle program (lib.rs + state.rs)
│   │   └── token_minter/src/         # Minter program (lib.rs)
│   ├── tests/                        # LiteSVM TypeScript tests
│   ├── scripts/                      # Init + helper scripts (JS)
│   ├── Anchor.toml                   # Program IDs, cluster, wallet
│   ├── Cargo.toml                    # Rust workspace
│   └── package.json                  # Test dependencies (mocha, chai)
├── backend/                          # Off-chain Rust service
│   ├── src/main.rs                   # Single-file: price updater + event listener
│   ├── Cargo.toml                    # Dependencies
│   └── .env.example                  # Configuration template
├── frontend/                         # Remix web application
│   ├── app/
│   │   ├── config.ts                 # Program IDs, networks, seeds
│   │   ├── mintInstruction.ts        # Manual Borsh instruction builder
│   │   ├── components/               # TerminalApp.tsx, TerminalMint.tsx
│   │   ├── routes/                   # Remix routes
│   │   └── styles/                   # Terminal-themed CSS
│   └── package.json                  # Frontend dependencies
├── docs/                             # Documentation
├── Makefile                          # Build/deploy/test orchestration
└── CLAUDE.md                         # AI development guidance
```

**Key conventions:**
- Each component is self-contained with its own package manager and build tool
- Backend is a single `main.rs` — no module splitting
- Programs share one Cargo workspace under `program/`
- No monorepo tooling (Turborepo, Nx) — just a Makefile

## Project Architecture

```
┌──────────────────┐       account read       ┌──────────────────┐
│  sol_usd_oracle   │◄────────────────────────│   token_minter    │
│  (price feed PDA) │                          │  (mint + fee PDA) │
└────────▲─────────┘                          └────────▲─────────┘
         │ update_price tx                             │ mint_token tx
         │                                             │
┌────────┴─────────┐                          ┌────────┴─────────┐
│   Rust Backend    │                          │  Remix Frontend   │
│ • price updater   │                          │ • wallet connect  │
│ • event listener  │                          │ • manual Borsh    │
└──────────────────┘                          └──────────────────┘
```

**Component interactions:**

| From | To | Mechanism | Purpose |
|---|---|---|---|
| Backend | Oracle program | RPC transaction (`update_price`) | Push SOL/USD price on-chain |
| Minter program | Oracle PDA | Account read (CPI validation) | Get current price for fee calc |
| Frontend | Minter program | Wallet-signed transaction (`mint_token`) | User mints tokens |
| Minter program | SPL Token | CPI | Create mint + mint tokens to ATA |
| Minter program | Metaplex | CPI (optional) | Attach token metadata |
| Backend | Minter program | WebSocket log subscription | Listen for `TokenCreated` events |
| Backend | Binance API | HTTP GET | Fetch SOL/USD market price |

**Key design decisions:**
- **No direct CPI from minter to oracle** — minter reads oracle PDA as a passed account, validates it belongs to the oracle program
- **Frontend doesn't use Anchor client** — manual Borsh encoding for full control and educational value
- **Backend is read-only for minter** — only listens to events, never submits minter transactions
- **Two independent data flows**: price updates (backend → oracle) and token minting (frontend → minter)

## Data Model

**On-chain state (PDAs):**

**OracleState** — seed: `"oracle_state"`, program: `sol_usd_oracle`

| Field | Type | Size | Description |
|---|---|---|---|
| admin | Pubkey | 32B | Authority allowed to update price |
| price | u64 | 8B | SOL/USD price × 10^6 (e.g., 120_000_000 = $120.00) |
| decimals | u8 | 1B | Always 6 |
| last_updated_slot | u64 | 8B | Slot of last price update |
| bump | u8 | 1B | PDA bump seed |

**MinterConfig** — seed: `"minter_config"`, program: `token_minter`

| Field | Type | Size | Description |
|---|---|---|---|
| admin | Pubkey | 32B | Minter authority |
| treasury | Pubkey | 32B | Wallet receiving minting fees |
| mint_fee_usd | u64 | 8B | Fee in USD × 10^6 (e.g., 5_000_000 = $5.00) |
| oracle_program | Pubkey | 32B | Expected oracle program ID |
| oracle_state | Pubkey | 32B | Expected oracle PDA address |
| bump | u8 | 1B | PDA bump seed |
| _padding | [u8; 7] | 7B | Alignment padding |

**Events:**

**TokenCreated** — emitted by `mint_token`

| Field | Type | Description |
|---|---|---|
| creator | Pubkey | User who minted |
| mint | Pubkey | New token mint address |
| decimals | u8 | Token decimals |
| supply | u64 | Initial supply |
| fee_lamports | u64 | Fee paid in lamports |
| price | u64 | Oracle price at mint time |
| slot | u64 | Slot number |

**Off-chain state:**
- `backend/.env` — runtime configuration (no database, no persistent state)
- Frontend polls PDA raw bytes directly: oracle price at byte offset 40, minter fee at offset 72, treasury at offset 40–72

**Key constraints:**
- All monetary values are `u64` with 6 decimal places — no floating point
- Fee formula: `fee_lamports = mint_fee_usd × LAMPORTS_PER_SOL / price` (u128 intermediate to prevent overflow)
- `price == 0` causes minting to fail (division by zero guard)

## Workflows

**1. Price Update Flow**
```
Binance API → Backend (poll every N sec) → update_price tx → Oracle PDA updated
```
1. Backend fetches SOL/USD from Binance (or uses `MOCK_PRICE`)
2. Converts decimal string to fixed-6 u64 via `to_fixed_6()`
3. Submits `update_price` instruction signed by admin keypair
4. Oracle PDA stores new price + slot number

**2. Token Minting Flow**
```
User (Frontend) → wallet signs mint_token tx → Minter program → SPL token created
```
1. Frontend reads oracle price + minter fee from PDA raw bytes (polling every 5s)
2. User fills form: decimals, supply, name, symbol, URI
3. Frontend builds `mint_token` instruction (manual Borsh encoding)
4. Wallet signs and sends transaction
5. Minter program: validates oracle account → computes fee → transfers fee to treasury → creates mint → mints tokens to user's ATA → optionally creates Metaplex metadata → emits `TokenCreated`

**3. Event Listening Flow**
```
Minter program emits log → Backend (WebSocket) → parses TokenCreated → JSON to stdout
```
1. Backend subscribes to minter program logs via WebSocket
2. Regex matches `TokenCreated { ... }` in log messages
3. Parsed fields output as structured JSON

**4. Development Workflow**
```
code → anchor build → deploy to localnet → init → test via frontend/backend
```
1. `make validator` — start local validator
2. `make deploy` — build + deploy programs
3. `make init` — initialize oracle ($120) + minter ($5 fee)
4. `make backend` + `make frontend` — run services
5. Connect wallet, mint token, verify backend logs

**5. Testing Workflow**
```
anchor test --skip-build   (programs, LiteSVM, no validator)
cargo test                 (backend unit tests)
npm test                   (frontend unit tests)
```
All tests run without a network — fast feedback loop.

## Deployment

**Two target environments:**

| Environment | RPC | WebSocket | Purpose |
|---|---|---|---|
| Localnet | `http://127.0.0.1:8899` | `ws://127.0.0.1:8900` | Development and testing |
| Devnet | `https://api.devnet.solana.com` | `wss://api.devnet.solana.com` | Pre-production verification |

**Localnet deployment:**
```bash
solana-test-validator          # or: make validator-metaplex
make deploy                    # anchor build + deploy
make init                      # initialize oracle + minter
```

**Localnet wallet**: Use Backpack for local development — it connects to `localhost:8899` correctly. Phantom, Solflare, and most wallets route RPC through remote servers, so `localhost` resolves to the server, not your machine. For full wallet functionality (balance display, etc.), use an ngrok tunnel: `ngrok http 8899` → set the resulting URL as Custom RPC in wallet settings. See `docs/deploy-local.md` for detailed setup instructions.

**Devnet deployment:**
```bash
solana config set --url devnet
solana airdrop 2               # fund deployer wallet
make deploy-devnet             # deploy both programs
make init-devnet               # initialize on devnet
```

**Program IDs:**
- Generated once via `solana-keygen grind` or `anchor keys sync`
- Updated in 4 places: both `declare_id!()` in Rust, `Anchor.toml`, `frontend/app/config.ts`
- Rebuilt with `anchor build` after ID changes

**Frontend:**
- Runs as dev server (`npm run dev`, port 7001)
- No production build/hosting — local dev only for this project
- Network switching (localnet/devnet) via UI toggle

**Backend:**
- Runs directly via `cargo run` (or `make backend`)
- No containerization, no process manager — single process, two tokio tasks
- Configure target cluster via `.env` variables

**No CI/CD pipeline** — manual deploy via Makefile commands. Fits the educational scope.

## Configuration Approach

**Principle:** environment variables via `.env` for runtime config; hardcoded constants for protocol-level values.

**Backend configuration** (`backend/.env`):

| Variable | Required | Default | Description |
|---|---|---|---|
| `SOLANA_RPC_HTTP` | yes | `http://127.0.0.1:8899` | RPC endpoint |
| `SOLANA_RPC_WS` | yes | `ws://127.0.0.1:8900` | WebSocket endpoint |
| `ORACLE_PROGRAM_ID` | yes | — | Oracle program ID |
| `MINTER_PROGRAM_ID` | yes | — | Minter program ID |
| `ORACLE_STATE_PUBKEY` | yes | — | Oracle PDA (from `make init` output) |
| `BACKEND_KEYPAIR_PATH` | yes | `~/.config/solana/id.json` | Admin keypair (supports `~`) |
| `PRICE_POLL_INTERVAL_SEC` | no | `600` | Price update interval |
| `MOCK_PRICE` | no | unset | Fixed price for testing (skips Binance) |
| `PRICE_API_URL` | no | Binance SOL/USDT | Custom price endpoint |

- Loaded via `dotenvy` at startup
- Template provided in `.env.example`
- `.env` is gitignored

**Program configuration:**
- Program IDs hardcoded in `declare_id!()` (Rust) and `Anchor.toml`
- PDA seeds hardcoded as string constants (`"oracle_state"`, `"minter_config"`)
- Fee and price set at initialization time via `make init` script, updatable on-chain

**Frontend configuration** (`frontend/app/config.ts`):
- Program IDs, network endpoints, PDA seeds — all in one file
- Network switching (localnet/devnet) via UI, not env vars
- No `.env` for frontend — everything is in `config.ts`

**No config frameworks, no YAML, no TOML for runtime.** Just `.env` + hardcoded constants.

## Logging Approach

**Backend:**
- Uses `tracing` + `tracing-subscriber` for structured logging
- Log levels: `info` for price updates and events, `error` for failures
- `TokenCreated` events parsed from WebSocket logs → output as JSON to stdout
- No log files, no external log aggregation — stdout only

**On-chain programs:**
- Anchor's `emit!()` macro for `TokenCreated` event (minter)
- Anchor's `msg!()` for debug messages during instruction execution
- Events are base64-encoded in transaction logs, parseable by backend via regex

**Frontend:**
- Browser console only (`console.log` / `console.error`)
- No telemetry, no analytics, no error reporting service

**No centralized logging, no ELK/Grafana, no log rotation.** Backend writes to stdout; redirect to file if needed (`cargo run >> app.log 2>&1`).
