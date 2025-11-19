# KALE Soroban Contract - Comprehensive Documentation

## Overview

KALE is a Stellar Soroban smart contract that implements a proof-of-work mining system for the KALE token. It's designed as a "farming" contract where users (farmers) can mine KALE tokens by completing computational work. The contract uses a three-step process: plant, work, and harvest.

## Core Architecture

### Key Constants
- **BLOCK_INTERVAL**: 300 seconds (5 minutes) - Time window for each mining block
- **BLOCK_REWARD**: ~2,505 KALE per block (calculated as 501 KALE per minute × 5 minutes)
- **BLOCKS_PER_MONTH**: 8,640 blocks (30 days of 5-minute blocks)
- **DECAY_RATE**: 5% per month (rewards decrease monthly)
- **V2_GENESIS_BLOCK**: Block 30,558 (starting point for V2 reward calculations)

### Data Structures

#### Block
Represents a mining period where multiple miners compete:
```rust
Block {
    timestamp: u64,           // When the block started
    min_gap: u32,            // Minimum ledger gap among all miners
    min_stake: i128,         // Minimum stake amount
    min_zeros: u32,          // Minimum zero count in hashes
    max_gap: u32,            // Maximum ledger gap
    max_stake: i128,         // Maximum stake amount
    max_zeros: u32,          // Maximum zero count
    entropy: BytesN<32>,     // Current block entropy (hash seed)
    staked_total: i128,      // Total KALE staked in this block
    normalized_total: i128,  // Total normalized scores
}
```

#### Pail
Individual miner's participation record for a specific block:
```rust
Pail {
    sequence: u32,           // Ledger sequence when planted
    gap: Option<u32>,        // Ledger gap between plant and work
    stake: i128,             // Amount of KALE staked
    zeros: Option<u32>,      // Number of leading zeros in hash
}
```

## Mining Process

### 1. Plant Phase (`plant` function)
- **Purpose**: Enter a mining block by staking KALE tokens
- **Process**:
  1. Farmer authorizes the transaction
  2. Checks if farming is not paused
  3. Creates new block if 5+ minutes have passed since last block
  4. Burns the staked KALE tokens (removed from circulation)
  5. Creates a Pail entry for the farmer with initial stake
  6. Updates block statistics (min/max stake values)
- **Requirements**:
  - Amount must be ≥ 0
  - Cannot plant twice in the same block
  - Farm must not be paused

### 2. Work Phase (`work` function)
- **Purpose**: Perform proof-of-work computation to earn mining score
- **Process**:
  1. Calculate hash using: `keccak256(block_index || nonce || block_entropy || farmer_address)`
  2. Count leading zeros in the hash (more zeros = better)
  3. Calculate ledger gap (time waited since planting)
  4. Normalize three factors: gap, stake, zeros (explained below)
  5. Add normalized score to block total
  6. Update block entropy with the new hash
- **Requirements**:
  - Must wait at least 1 ledger after planting
  - Can be called multiple times to improve score
  - Each attempt must have more zeros than the previous
- **Note**: No auth required - anyone can call work on behalf of a farmer

### 3. Harvest Phase (`harvest` function)
- **Purpose**: Claim rewards after block completion
- **Process**:
  1. Verify block is complete (new block has started)
  2. Calculate farmer's share: `(farmer_normalized_score / block_total_score) × (block_reward + total_staked)`
  3. Mint KALE tokens: original stake + earned rewards
  4. Remove farmer's Pail entry
- **Requirements**:
  - Block must be complete (index < current farm index)
  - Must have completed work phase
  - Cannot harvest from active block

## Block System

### Block Creation and Timing
- **New blocks start** when the first `plant` call occurs ≥5 minutes after the previous block's timestamp
- **Block duration** is variable but has a minimum of 5 minutes
- **Block completion** occurs when a new block starts (blocks don't have a fixed end time)
- **Active block** continues accepting plants/works until a new block is created

### Block Dependencies
- Blocks are **sequential** but **independent** in terms of rewards
- Each block has its own:
  - Entropy (inherited from previous, updated during work)
  - Min/max ranges for normalization
  - Total stake and normalized totals
- Block index increments when a new block starts
- Historical blocks remain accessible for harvesting

### Block Lifecycle
1. **Genesis/Creation**: First plant after 5+ minutes creates new block
2. **Active Phase**: Accepts plants and works, accumulates scores
3. **Completion**: Becomes historical when next block starts
4. **Harvest Phase**: Farmers can harvest their rewards
5. **Eviction**: After TTL expires, block data may be removed (but can be recreated if needed)

## Reward Distribution Mechanism

### Normalization System
The contract normalizes three performance metrics to create fair competition:

1. **Gap Normalization** (time factor):
   - Rewards waiting between plant and work
   - Normalized to 0-1 scale based on block's min/max gaps
   - Encourages strategic timing

2. **Stake Normalization** (investment factor):
   - Rewards higher stakes
   - Normalized to 0-1 scale based on block's min/max stakes
   - Balances risk vs reward

3. **Zeros Normalization** (computation factor):
   - Rewards computational work (finding hashes with leading zeros)
   - Normalized to 0-1 scale based on block's min/max zeros
   - Proof of work component

### Reward Calculation
```
farmer_reward = (normalized_gap + normalized_stake + normalized_zeros)
                × (block_reward + block_staked_total)
                / block_normalized_total
```

### Distribution Characteristics
- **NOT constant per block** - varies based on participation
- **NOT equal shares** - proportional to normalized performance
- **Includes stake return** - farmers get back their stake plus rewards
- **Monthly decay** - base reward decreases 5% each month from genesis

### Reward Decay
- Starts at ~2,505 KALE per block
- Decreases by 5% every 8,640 blocks (30 days)
- Calculated dynamically at harvest time
- Formula: `base_reward × (0.95^months_elapsed)`

## Storage Architecture

### Instance Storage (Persistent)
- `Homesteader`: Contract admin address
- `HomesteadAsset`: KALE token contract address
- `FarmIndex`: Current active block index
- `FarmBlock`: Current block being mined
- `FarmPaused`: Pause state

### Temporary Storage (TTL-based)
- `Block(index)`: Historical block data
- `Pail(farmer, index)`: Farmer participation records
- Auto-extends TTL on contract interactions
- Data may be evicted but can be recreated

## Administrative Functions

### Constructor (`__constructor`)
- Initializes contract with admin and KALE token address
- Creates initial farm block
- One-time setup only

### Upgrade (`upgrade`)
- Updates contract WASM code
- Requires homesteader authorization
- Preserves all storage data

### Pause/Unpause (`pause`/`unpause`)
- Temporarily stops new mining activity
- Existing blocks can still be harvested
- Requires homesteader authorization

### Remove Block (`remove_block`)
- Removes specific block data from storage
- Administrative cleanup function
- Requires homesteader authorization

## Important Implementation Details

### Hash Generation
- Uses Keccak256 algorithm
- Input: 76 bytes total
  - 4 bytes: block index (big-endian)
  - 8 bytes: nonce (big-endian)
  - 32 bytes: block entropy
  - 32 bytes: farmer address
- Leading zeros counted in hex representation (4-bit groups)

### Race Condition Handling
- Multiple farmers can plant in same ledger
- First plant after 5 minutes creates new block
- Others join existing block
- Contract pre-reads future blocks to handle budget constraints

### Security Features
- Minimum 1 ledger gap prevents immediate work after plant
- Work improvements require strictly more zeros
- Farmers cannot plant twice in same block
- All operations extend storage TTL

## Key Mining Parameters Explained

### Block Index
- **What it is**: A sequential counter identifying each mining block (starts at 0)
- **How it's determined**:
  - Stored in contract storage as `FarmIndex`
  - Increments by 1 when a new block is created
  - New block creation triggered by first `plant` call after 5+ minutes
- **Current value**: The active block being mined
- **Usage**: Identifies which block you're mining in and harvesting from

### Nonce
- **What it is**: A number (u64) that miners choose to generate different hash attempts
- **How it works**:
  - Miners iterate through different nonce values (0, 1, 2, ...)
  - Each nonce produces a different hash when combined with other inputs
  - Goal is to find a nonce that produces a hash with many leading zeros
- **Chosen by**: The miner (client-side) during proof-of-work
- **Strategy**: Start at 0 and increment, or use random values

### Block Entropy
- **What it is**: A 32-byte hash value that serves as a seed for mining
- **Initial value**: Array of 32 zeros for the first block
- **How it changes**:
  - Updated every time a miner successfully submits work
  - New entropy = the hash submitted by the last successful worker
  - Ensures mining difficulty is dynamic and unpredictable
- **Purpose**: Prevents pre-computation of hashes and ensures fairness

## Data Fetching for UI Development

### Using Stellar RPC with Rust

The Stellar RPC server allows you to interact with Soroban contracts through JSON-RPC calls. Here's how to fetch the required data using Rust.

#### Required Dependencies
Add these to your `Cargo.toml`:
```toml
[dependencies]
soroban-client = "21.0.0"
stellar-xdr = "21.0.0"
stellar-strkey = "0.0.9"
sha3 = "0.10"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

#### 1. Direct Storage Access
Soroban contracts store data with specific keys that can be queried:

```rust
use soroban_client::{Client, ContractId, Error};
use stellar_xdr::curr::{ScVal, ScSymbol, ScVec, ScAddress, ScU32};
use stellar_strkey::StrKey;

// Initialize client
let client = Client::new("https://soroban-testnet.stellar.org").unwrap();
let contract_address = ContractId::from_str("CONTRACT_ADDRESS_HERE").unwrap();

// Read storage keys directly
async fn get_contract_data(
    client: &Client,
    contract_address: &ContractId,
    user_address: &str,
    block_index: u32
) -> Result<MiningData, Error> {
    // Get current block index
    let farm_index_key = ScVal::Symbol(ScSymbol("FarmIndex".try_into().unwrap()));
    let farm_index_entry = client
        .get_contract_data(contract_address, &farm_index_key)
        .await?;

    // Get current farm block (contains entropy and current stats)
    let farm_block_key = ScVal::Symbol(ScSymbol("FarmBlock".try_into().unwrap()));
    let farm_block_entry = client
        .get_contract_data(contract_address, &farm_block_key)
        .await?;

    // Get specific block data
    let block_key = ScVal::Vec(Some(ScVec(vec![
        ScVal::Symbol(ScSymbol("Block".try_into().unwrap())),
        ScVal::U32(block_index)
    ].try_into().unwrap())));
    let block_entry = client
        .get_contract_data(contract_address, &block_key)
        .await?;

    // Get user's pail (participation record)
    let user_strkey = StrKey::from_str(user_address).unwrap();
    let pail_key = ScVal::Vec(Some(ScVec(vec![
        ScVal::Symbol(ScSymbol("Pail".try_into().unwrap())),
        ScVal::Address(ScAddress::from(user_strkey)),
        ScVal::U32(block_index)
    ].try_into().unwrap())));
    let pail_entry = client
        .get_contract_data(contract_address, &pail_key)
        .await?;

    // Parse and return the data
    Ok(MiningData {
        farm_index: parse_u32(farm_index_entry),
        farm_block: parse_block(farm_block_entry),
        current_block: parse_block(block_entry),
        user_pail: parse_pail(pail_entry),
    })
}

// Helper functions to parse ScVal responses
fn parse_u32(val: ScVal) -> u32 {
    match val {
        ScVal::U32(n) => n,
        _ => 0,
    }
}

fn parse_block(val: ScVal) -> Block {
    // Parse the block struct from ScVal
    // Implementation depends on your Block struct definition
    todo!()
}
```

#### 2. Contract Invocation (Simulated Transactions)
Simulate transactions to check state or prepare actual invocations:

```rust
use soroban_client::{TransactionBuilder, Operation};
use stellar_xdr::curr::{HostFunction, InvokeContractArgs};

// Simulate a harvest call to check if a block is ready
async fn check_harvestable(
    client: &Client,
    contract_address: &ContractId,
    user_address: &str,
    block_index: u32
) -> Result<bool, Error> {
    // Build the harvest invocation
    let harvest_args = vec![
        ScVal::Address(ScAddress::from(StrKey::from_str(user_address)?)),
        ScVal::U32(block_index),
    ];

    let invoke_contract = InvokeContractArgs {
        contract_address: contract_address.clone().into(),
        function_name: ScSymbol("harvest".try_into()?),
        args: harvest_args.try_into()?,
    };

    let host_function = HostFunction::InvokeContract(invoke_contract);

    // Simulate the transaction
    let sim_result = client
        .simulate_transaction(&host_function)
        .await?;

    // Check if simulation succeeded (harvest is ready)
    Ok(sim_result.error.is_none())
}

// Execute actual contract calls
async fn plant_kale(
    client: &Client,
    contract_address: &ContractId,
    farmer_address: &str,
    amount: i128,
) -> Result<(), Error> {
    let args = vec![
        ScVal::Address(ScAddress::from(StrKey::from_str(farmer_address)?)),
        ScVal::I128(amount.into()),
    ];

    let invoke_contract = InvokeContractArgs {
        contract_address: contract_address.clone().into(),
        function_name: ScSymbol("plant".try_into()?),
        args: args.try_into()?,
    };

    let host_function = HostFunction::InvokeContract(invoke_contract);

    // Submit the transaction
    client.submit_transaction(&host_function).await?;
    Ok(())
}
```

#### 3. Event Monitoring
Monitor contract events for real-time updates:

```rust
use soroban_client::EventFilter;
use futures::StreamExt;

// Subscribe to contract events
async fn watch_contract_events(
    client: &Client,
    contract_address: &ContractId,
    start_ledger: u32
) -> Result<(), Error> {
    let filter = EventFilter {
        contract_ids: vec![contract_address.clone()],
        ..Default::default()
    };

    let mut event_stream = client
        .get_events(start_ledger, Some(filter))
        .await?;

    while let Some(event) = event_stream.next().await {
        match event {
            Ok(contract_event) => {
                // Process events like new blocks, work submissions, etc.
                println!("Contract event: {:?}", contract_event);
                handle_event(contract_event);
            }
            Err(e) => eprintln!("Event stream error: {:?}", e),
        }
    }

    Ok(())
}
```

### Data Requirements for UI

To build a complete UI, fetch and monitor:

1. **On page load**:
   - `FarmIndex` - Current block index
   - `FarmBlock` - Current block metadata (entropy, timestamp, min/max values)
   - `Block(farmIndex)` - Current block details
   - `Pail(user, farmIndex)` - User's current participation

2. **Periodically poll** (every 5-10 seconds):
   - `FarmBlock.timestamp` - Detect new blocks
   - `Block.normalized_total` - Update reward estimates
   - `FarmBlock.entropy` - Detect other miners' work

3. **Before operations**:
   - Check `FarmPaused` - Ensure mining isn't paused
   - Verify block index hasn't changed
   - Calculate optimal nonce range for work

### Example: Complete Data Fetch Function
```rust
use tokio::join;

#[derive(Debug)]
struct MiningDataDisplay {
    block_index: u32,
    block_entropy: [u8; 32],
    block_timestamp: u64,
    min_max_ranges: MinMaxRanges,
    total_staked: i128,
    normalized_total: i128,
    user_status: UserStatus,
}

#[derive(Debug)]
struct MinMaxRanges {
    gap: (u32, u32),
    stake: (i128, i128),
    zeros: (u32, u32),
}

#[derive(Debug)]
struct UserStatus {
    has_planted: bool,
    stake: i128,
    zeros: u32,
    gap: u32,
}

async fn fetch_mining_data(
    client: &Client,
    contract_address: &ContractId,
    user_address: &str
) -> Result<MiningDataDisplay, Error> {
    // Get all required data in parallel
    let (farm_index, farm_block, current_block, user_pail) = join!(
        get_storage_value(client, contract_address, "FarmIndex"),
        get_storage_value(client, contract_address, "FarmBlock"),
        get_storage_value(client, contract_address, &format!("Block:{}", current_index)),
        get_storage_value(client, contract_address, &format!("Pail:{}:{}", user_address, current_index))
    );

    Ok(MiningDataDisplay {
        block_index: farm_index?,
        block_entropy: parse_entropy(farm_block?),
        block_timestamp: parse_timestamp(farm_block?),
        min_max_ranges: MinMaxRanges {
            gap: (farm_block.min_gap, farm_block.max_gap),
            stake: (farm_block.min_stake, farm_block.max_stake),
            zeros: (farm_block.min_zeros, farm_block.max_zeros),
        },
        total_staked: current_block?.staked_total,
        normalized_total: current_block?.normalized_total,
        user_status: UserStatus {
            has_planted: user_pail.is_ok(),
            stake: user_pail.as_ref().map_or(0, |p| p.stake),
            zeros: user_pail.as_ref().and_then(|p| p.zeros).unwrap_or(0),
            gap: user_pail.as_ref().and_then(|p| p.gap).unwrap_or(0),
        },
    })
}
```

## User Interface Considerations

### Key Metrics to Display
1. **Current Block Info**:
   - Block index
   - Time remaining (5 minutes - elapsed)
   - Total staked amount
   - Number of participants
   - Current min/max ranges (gap, stake, zeros)

2. **User Position**:
   - Stake amount
   - Current zeros achieved
   - Normalized score components
   - Estimated reward share
   - Ability to improve (work again)

3. **Historical Data**:
   - Completed blocks awaiting harvest
   - Past rewards claimed
   - Total KALE mined

4. **Network Stats**:
   - Current block reward (with decay)
   - Total KALE in circulation
   - Mining difficulty trends
   - Active miners count

### Recommended UI Flow
1. **Dashboard**: Show current block, user positions, claimable rewards
2. **Plant Interface**: Input stake amount, show potential returns
3. **Work Monitor**: Display hash attempts, zeros achieved, option to retry
4. **Harvest Manager**: List completed blocks with claimable amounts
5. **Analytics**: Historical performance, network statistics, decay schedule

### Real-time Updates
- Poll block timestamp to detect new blocks
- Monitor normalized totals for reward estimates
- Track entropy changes from other miners' work
- Update countdown timer for block intervals

## Contract Invocation Examples

### Typical Mining Sequence
```rust
use sha3::{Digest, Keccak256};
use stellar_xdr::curr::{ScVal, ScSymbol, ScI128, ScU32, ScU64};

// 1. Plant (stake 1000 KALE)
async fn plant_step(
    client: &Client,
    contract_address: &ContractId,
    farmer_address: &str,
) -> Result<(), Error> {
    let amount: i128 = 1000_0000000; // 7 decimal places

    let args = vec![
        ScVal::Address(ScAddress::from(StrKey::from_str(farmer_address)?)),
        ScVal::I128(ScI128::from(amount)),
    ];

    let invoke = InvokeContractArgs {
        contract_address: contract_address.clone().into(),
        function_name: ScSymbol("plant".try_into()?),
        args: args.try_into()?,
    };

    client.submit_transaction(&HostFunction::InvokeContract(invoke)).await?;
    Ok(())
}

// 2. Work (find proof-of-work)
async fn work_step(
    client: &Client,
    contract_address: &ContractId,
    farmer_address: &str,
    block_index: u32,
    entropy: [u8; 32],
) -> Result<u32, Error> {
    let mut best_zeros = 0;
    let mut nonce: u64 = 0;

    while can_improve(best_zeros) {
        let hash = calculate_hash(block_index, nonce, &entropy, farmer_address);
        let zeros = count_leading_zeros(&hash);

        if zeros > best_zeros {
            let args = vec![
                ScVal::Address(ScAddress::from(StrKey::from_str(farmer_address)?)),
                ScVal::BytesN(hash.try_into()?),
                ScVal::U64(nonce),
            ];

            let invoke = InvokeContractArgs {
                contract_address: contract_address.clone().into(),
                function_name: ScSymbol("work".try_into()?),
                args: args.try_into()?,
            };

            let result = client
                .submit_transaction(&HostFunction::InvokeContract(invoke))
                .await?;

            best_zeros = zeros;
        }
        nonce += 1;
    }

    Ok(best_zeros)
}

// Hash calculation function
fn calculate_hash(
    index: u32,
    nonce: u64,
    entropy: &[u8; 32],
    farmer_address: &str
) -> [u8; 32] {
    let mut hasher = Keccak256::new();

    // Construct the 76-byte input
    hasher.update(&index.to_be_bytes());      // 4 bytes
    hasher.update(&nonce.to_be_bytes());      // 8 bytes
    hasher.update(entropy);                    // 32 bytes

    // Convert farmer address to 32 bytes
    let farmer_bytes = StrKey::from_str(farmer_address)
        .unwrap()
        .to_bytes();
    hasher.update(&farmer_bytes[farmer_bytes.len()-32..]);  // Last 32 bytes

    hasher.finalize().into()
}

fn count_leading_zeros(hash: &[u8; 32]) -> u32 {
    let mut zeros = 0;
    for byte in hash {
        if *byte == 0 {
            zeros += 2;  // Two hex digits
        } else {
            zeros += byte.leading_zeros() / 4;
            break;
        }
    }
    zeros
}

// 3. Harvest (after block completes)
async fn harvest_step(
    client: &Client,
    contract_address: &ContractId,
    farmer_address: &str,
    block_index: u32,
) -> Result<i128, Error> {
    let args = vec![
        ScVal::Address(ScAddress::from(StrKey::from_str(farmer_address)?)),
        ScVal::U32(block_index),
    ];

    let invoke = InvokeContractArgs {
        contract_address: contract_address.clone().into(),
        function_name: ScSymbol("harvest".try_into()?),
        args: args.try_into()?,
    };

    let result = client
        .submit_transaction(&HostFunction::InvokeContract(invoke))
        .await?;

    // Parse the i128 reward from result
    parse_i128_from_result(result)
}
```

### Reading Contract State
```rust
use std::time::{SystemTime, UNIX_EPOCH};

// Get current block info
async fn get_current_block_info(
    client: &Client,
    contract_address: &ContractId,
) -> Result<BlockInfo, Error> {
    // Get farm index
    let farm_index_key = ScVal::Symbol(ScSymbol("FarmIndex".try_into()?));
    let farm_index = parse_u32(
        client.get_contract_data(contract_address, &farm_index_key).await?
    );

    // Get farm block
    let farm_block_key = ScVal::Symbol(ScSymbol("FarmBlock".try_into()?));
    let farm_block = parse_block(
        client.get_contract_data(contract_address, &farm_block_key).await?
    );

    // Get current block
    let block_key = ScVal::Vec(Some(ScVec(vec![
        ScVal::Symbol(ScSymbol("Block".try_into()?)),
        ScVal::U32(farm_index)
    ].try_into()?)));
    let block = parse_block(
        client.get_contract_data(contract_address, &block_key).await?
    );

    Ok(BlockInfo {
        index: farm_index,
        farm_block,
        current_block: block,
    })
}

// Check user participation
async fn get_user_participation(
    client: &Client,
    contract_address: &ContractId,
    user_address: &str,
    block_index: u32,
) -> Result<Option<Pail>, Error> {
    let user_strkey = StrKey::from_str(user_address)?;
    let pail_key = ScVal::Vec(Some(ScVec(vec![
        ScVal::Symbol(ScSymbol("Pail".try_into()?)),
        ScVal::Address(ScAddress::from(user_strkey)),
        ScVal::U32(block_index)
    ].try_into()?)));

    match client.get_contract_data(contract_address, &pail_key).await {
        Ok(val) => Ok(Some(parse_pail(val)?)),
        Err(_) => Ok(None),
    }
}

// Calculate time until next block
fn calculate_time_remaining(block_timestamp: u64) -> u64 {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let elapsed = current_time.saturating_sub(block_timestamp);
    300_u64.saturating_sub(elapsed) // 300 seconds = 5 minutes
}
```

## Summary

The KALE contract implements a sophisticated mining system that balances three factors: time (gap), investment (stake), and computation (zeros). It operates on 5-minute block intervals with proportional reward distribution based on normalized performance scores. The monthly decay mechanism ensures long-term sustainability, while the normalization system prevents any single strategy from dominating. Understanding these mechanics is crucial for building an effective user interface that helps farmers optimize their mining strategy and track their rewards.