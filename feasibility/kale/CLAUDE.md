# Galactic Playground

A full-stack Rust/TypeScript application for KALE token farming on Stellar Soroban. The
application implements a complete proof-of-work mining workflow: plant seeds, perform work
through browser-based mining, and harvest rewards from previous blocks.

The KALE contract is a Soroban smart contract that implements a proof-of-work farming
system. Read `docs/kale_contract.md` to learn how the contract works, or explore the
complete contract source at `external/KALE_SOROBAN_CONTRACT`.

## Prerequisites

- Rust (latest stable)
- Node.js and npm (for frontend)
- A browser with Albedo wallet access
- KALE trustline on Stellar testnet (instructions provided in the UI)

## Setup

### 1. Build the Frontend

```bash
cd frontend
npm install
npm run build
cd ..
```

### 2. Build the Rust Application

```bash
cargo build --release
```

## Running

```bash
cargo run
```

The application will:

1. Connect to the KALE contract on Stellar testnet
2. Display the current farm block index
3. Start a local server on `http://localhost:3737`
4. Open your browser to the farming interface
5. Allow you to connect your Albedo wallet (persisted in localStorage)
6. Provide a unified interface for Plant → Work → Harvest farming cycle

**KALE Farming Workflow:**

1. **Connect Wallet**: Authenticate once with Albedo (address persists across sessions)
2. **Plant (0 KALE)**: Enter the current farming block by planting seeds
3. **Work**: Mine for 10 seconds in the browser to find the best hash (leading zeros)
4. **Harvest**: Claim rewards from previous blocks where you've completed work
5. **Repeat**: Each block lasts 5 minutes; start the cycle again when a new block begins

**Trustline Requirement:**

- Your account must have a trustline to KALE token:
  `KALE:GCHPTWXMT3HYF4RLZHWBNRF4MPXLTJ76ISHMSYIWCCDXWUYOQG5MR2AB`
- The application provides step-by-step instructions if the trustline is missing

## How It Works

### Backend Architecture (`src/`)

The Rust backend (`src/albedo.rs`, `src/contracts/kale.rs`, `src/rpc.rs`) provides:

- **HTTP Server**: Axum-based server on `localhost:3737`
- **Frontend Serving**: HTML embedded at compile time, JS from `frontend/dist/`
- **REST API Endpoints**:
  - `/api/pubkey` - Receives authentication result from Albedo
  - `/api/plant/prepare` - Builds and simulates plant transaction, returns unsigned XDR
  - `/api/plant/submit` - Submits signed plant transaction to Stellar network
  - `/api/work/prepare` - Builds and simulates work transaction with nonce
  - `/api/work/submit` - Submits signed work transaction
  - `/api/harvest/prepare` - Builds and simulates harvest transaction for a block
  - `/api/harvest/submit` - Submits signed harvest transaction
  - `/api/check_planted` - Checks if user has planted in the current block
  - `/api/block_info` - Returns current block index and entropy for mining
  - `/api/pail_data` - Returns pail data (planted, worked, leading zeros) for a block
- **Transaction Building**: Constructs Soroban contract invocations with proper auth
- **RPC Simulation**: Uses Stellar RPC to simulate and calculate resource fees
- **Trustline Validation**: Checks for KALE trustline before allowing transactions

### Frontend Architecture (`frontend/src/`)

The React/TypeScript frontend provides a unified single-page farming interface:

- **Session Management**:
  - Persists wallet address in localStorage (no repeated Albedo popups)
  - Connect Wallet / Logout buttons
  - Auto-loads user data on startup if wallet previously connected

- **Unified UI Sections**:
  - **Contract Info**: Displays contract address with trust indicator
  - **Account Row**: Shows connected wallet or Connect Wallet button
  - **Plant Section**: Button to plant seeds (disabled if already planted)
  - **Work Section**: Button to mine for 10 seconds (enabled only after planting)
  - **Harvest Section**: Multiple buttons for each harvestable block

- **Browser-Based Mining**:
  - Implements Keccak256 proof-of-work in JavaScript
  - Mines for 10 seconds to find the hash with most leading zeros
  - Real-time progress display and best zeros counter
  - Uses `js-sha3` and `@stellar/stellar-base` libraries

- **Albedo Integration**:
  - `albedo.publicKey()` - One-time authentication (cached in localStorage)
  - `albedo.tx()` - Transaction signing with proper network passphrase
  - Popup-based user approval for each transaction

### Transaction Flow

1. **Build**: Construct Soroban contract invocation (InvokeHostFunction operation)
2. **Simulate**: Send to Stellar RPC to calculate resources and footprint
3. **Wrap**: Package as TransactionEnvelope for signing
4. **Sign**: User approves in Albedo popup, signs with private key
5. **Submit**: Send signed envelope to Stellar network via RPC
6. **Confirm**: Display transaction hash with Stellar Expert link

### Mining Process

The browser performs proof-of-work mining using the KALE contract's hash verification:

```
Hash = Keccak256(block_index || nonce || entropy || farmer_address)
```

- **Input**: 76 bytes (4 + 8 + 32 + 32)
- **Goal**: Maximize leading zeros in the resulting hash
- **Duration**: 10 seconds of hashing with incrementing nonce
- **Output**: Best nonce and corresponding hash

## Project Structure

```
.
├── src/
│   ├── main.rs              # Entry point, connects to KALE and starts server
│   ├── albedo.rs            # HTTP server, API endpoints, session management
│   ├── rpc.rs               # Soroban RPC client for simulation and submission
│   └── contracts/
│       ├── mod.rs           # Contract module exports
│       └── kale.rs          # KALE contract client (plant, work, harvest)
├── frontend/
│   ├── src/
│   │   ├── App.tsx          # Main React component (unified farming UI)
│   │   └── index.tsx        # React entry point
│   ├── public/
│   │   └── index.html       # HTML template (embedded at compile time)
│   ├── dist/                # Built frontend assets (served by Axum)
│   ├── package.json         # Dependencies: React, Albedo, Stellar SDK
│   └── build.js             # esbuild configuration
├── docs/
│   ├── kale_contract.md     # KALE contract documentation (632 lines)
│   └── albedo_wallet.md     # Albedo integration guide (762 lines)
├── prompts/
│   ├── 1_explore_kale.md    # Initial KALE exploration
│   ├── 2_implement_soroban_rpc.md
│   ├── 3_implement_albedo_wallet.md
│   ├── 4_implement_plant_invokation.md
│   ├── 5_implement_work_invokation.md
│   ├── 6_implement_harvest_invokation.md
│   ├── 7_unify_ui.md
│   ├── 8_session_management.md
│   └── 9_funding_and_trustline.md
├── external/
│   └── KALE_SOROBAN_CONTRACT/ # Complete KALE contract source code
├── Cargo.toml               # Rust dependencies (Axum, Stellar SDK, etc.)
├── CLAUDE.md                # This file
└── README.md
```

## Development

### Frontend Development

To automatically rebuild the frontend on changes:

```bash
cd frontend
npm run watch
```

Then in another terminal, run the Rust application:

```bash
cargo run
```

### Key Development Files

- **`src/albedo.rs:1-511`** - All HTTP endpoints and server logic
- **`src/contracts/kale.rs`** - Contract invocation logic for plant/work/harvest
- **`src/rpc.rs`** - Stellar RPC client implementation
- **`frontend/src/App.tsx:1-1115`** - Complete farming UI with all states

### Common Development Tasks

1. **Add a new contract function**: Extend `src/contracts/kale.rs` with the function
   invocation, then add corresponding prepare/submit endpoints in `src/albedo.rs`

2. **Modify the UI**: Edit `frontend/src/App.tsx`, rebuild with `npm run build`, and
   restart the Rust application

3. **Debug transactions**: Check browser console for Albedo responses and backend logs
   for RPC simulation results

### Notes

- The frontend must be built before running the Rust application
- If you see errors, ensure `npm run build` completed successfully in `frontend/`
- The HTML is embedded at compile time from `frontend/public/index.html`
- JavaScript is served from `frontend/dist/` directory

## Key Features

### Implemented Functionality

- ✅ **Plant**: Stake 0 KALE to enter the current farming block
- ✅ **Work**: 10-second browser-based Keccak256 proof-of-work mining
- ✅ **Harvest**: Claim rewards from completed blocks (can harvest multiple blocks)
- ✅ **Session Persistence**: Wallet address stored in localStorage
- ✅ **Unified UI**: All farming actions on a single page
- ✅ **Real-time Mining**: Live progress and leading zeros counter
- ✅ **Block Tracking**: Check planted/worked status for any block
- ✅ **Trustline Validation**: Checks for KALE trustline before transactions
- ✅ **Error Handling**: User-friendly error messages with recovery instructions

### KALE Contract Functions

The application supports the following KALE contract functions:

1. **`plant(farmer: Address, stake: i128)`**
   - Enters the current farming block
   - Contract generates entropy and creates a Pail for the farmer
   - Stake amount can be 0 (allows participation without initial KALE)

2. **`work(farmer: Address, nonce: u64)`**
   - Submits proof-of-work for the current block
   - Contract verifies the hash has valid leading zeros
   - Updates the Pail with gap (latency) and zeros count
   - Can be called multiple times to improve the solution

3. **`harvest(farmer: Address, block_index: u32)`**
   - Claims rewards from a completed block
   - Calculates reward based on normalized score (stake, gap, zeros)
   - Can only harvest from previous blocks (not the current active block)
   - Temporary storage persists for multiple ledgers, allowing batch harvesting

## Documentation

- **[KALE Contract Documentation](docs/kale_contract.md)** - Comprehensive 762-line
  guide covering all contract functions, data structures, mining mechanics, reward
  calculations, and normalization formulas

- **[Albedo Integration Guide](docs/albedo_wallet.md)** - Complete 632-line reference
  for Albedo wallet integration including authentication flow, transaction signing,
  error handling, popup management, and troubleshooting

## Network Configuration

- **Testnet RPC**: `https://soroban-testnet.stellar.org`
- **Contract Address**: `CDSWUUXGPWDZG76ISK6SUCVPZJMD5YUV66J2FXFXFGDX25XKZJIEITAO`
- **Network Passphrase**: `Test SDF Network ; September 2015`
- **KALE Token Issuer**: `GCHPTWXMT3HYF4RLZHWBNRF4MPXLTJ76ISHMSYIWCCDXWUYOQG5MR2AB`
- **Server Port**: `3737` (localhost only)

## Development History

The application was built through an iterative development process documented in
`prompts/`:

1. KALE contract exploration and understanding
2. Soroban RPC client implementation
3. Albedo wallet integration
4. Plant function invocation
5. Work function with browser-based mining
6. Harvest function with multi-block support
7. UI unification (single-page interface)
8. Session management (localStorage persistence)
9. Funding and trustline handling
