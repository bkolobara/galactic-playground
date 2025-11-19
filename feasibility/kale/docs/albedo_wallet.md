# Albedo Wallet Integration Guide

This document provides detailed information about the Albedo wallet integration in the
Galactic Playground application, including authentication, transaction signing, and
important implementation details.

## Overview

Albedo is a security-centric, browser-based delegated signer and keystore for the Stellar
Network. It provides secure key management without exposing secret keys to third-party
applications, using a sandboxed popup window for all sensitive operations.

**Key Benefits:**

- Secret keys never leave the Albedo domain
- User-friendly transaction approval flow
- Support for multiple accounts
- Works entirely in the browser (no extensions needed)

## Architecture

The integration consists of three main components:

1. **Backend (Rust)** - Prepares unsigned transactions
2. **Frontend (React)** - Coordinates with Albedo via postMessage
3. **Albedo Popup** - Handles user authentication and transaction signing

```
┌─────────────┐         ┌──────────────┐         ┌───────────────┐
│   Backend   │◄────────┤   Frontend   │◄────────┤ Albedo Popup  │
│   (Rust)    │         │   (React)    │         │  (albedo.link)│
└─────────────┘         └──────────────┘         └───────────────┘
      │                        │                         │
      │ 1. Prepare TX          │ 2. Request signature    │
      │                        │ 3. User approves        │
      │                        │ 4. Return signed TX     │
      │ 5. Submit TX           │                         │
      │                        │                         │
```

## Dependencies

### Frontend

```json
{
  "@albedo-link/intent": "^0.12.0"
}
```

Import in React/TypeScript:

```typescript
import albedo from "@albedo-link/intent";
```

### Backend

No direct Albedo dependency needed - the backend only needs to prepare valid Stellar
transactions.

## Authentication Flow

### 1. Request Public Key

The first step is to authenticate the user and obtain their Stellar public key.

**Frontend code:**

```typescript
import albedo from "@albedo-link/intent";

// Request public key from Albedo
const response = await albedo.publicKey({
  token: crypto.randomUUID(), // Optional: unique request identifier
});

console.log(response.pubkey); // "GCIIWJT5LROESQ65NOUNAO23FELJVODJHLDU3EWO6GULFV4MY6S7FY3L"
```

**Parameters:**

- `token` (optional) - Unique identifier for the request

**Returns:**

```typescript
{
  pubkey: string;     // The selected Stellar public key
  signed_message?: string; // Optional signature proof
}
```

**User Experience:**

1. Albedo popup opens
2. User selects an account (if multiple)
3. User confirms the request
4. Popup closes and returns the public key

### 2. Store Authentication State

Once authenticated, store the public key for subsequent operations:

```typescript
const [publicKey, setPublicKey] = useState<string | null>(null);

// After successful authentication
setPublicKey(response.pubkey);
```

## Transaction Signing Flow

### Step 1: Backend Prepares Transaction

The backend builds and simulates the transaction, then returns an unsigned
TransactionEnvelope.

**Critical Requirements:**

1. Transaction must be wrapped in a `TransactionEnvelope` (not raw `Transaction`)
2. Envelope must have empty signatures array
3. XDR must be base64-encoded

**Rust code example:**

```rust
use stellar_xdr::curr::{Transaction, TransactionEnvelope, TransactionV1Envelope, WriteXdr};

// After building and simulating the transaction
let tx_envelope = TransactionEnvelope::Tx(
    TransactionV1Envelope {
        tx: transaction,
        signatures: VecM::default(), // Empty - will be filled by Albedo
    },
);

// Convert to base64 XDR
let tx_xdr = tx_envelope.to_xdr_base64(Limits::none())?;
```

**Why TransactionEnvelope?**

- Albedo expects the standard Stellar transaction format
- A `TransactionEnvelope` wraps the transaction with space for signatures
- Raw `Transaction` XDR will be rejected with "Invalid transaction XDR" error

### Step 2: Frontend Requests Signature

Send the transaction to Albedo for signing.

**Frontend code:**

```typescript
const signResponse = await albedo.tx({
  xdr: prepareData.xdr, // Base64 TransactionEnvelope XDR
  network: "Test SDF Network ; September 2015", // Full network passphrase
  submit: false, // Don't auto-submit
  description: "Plant 0 KALE in farming contract", // Optional
});

console.log(signResponse.signed_envelope_xdr); // Signed transaction
console.log(signResponse.tx_hash); // Transaction hash
```

**Parameters:**

- `xdr` **(required)** - Base64-encoded TransactionEnvelope XDR
- `network` **(required)** - Full Stellar network passphrase (NOT "testnet" or "public")
- `submit` (optional) - If true, Albedo will submit to Horizon (default: true)
- `pubkey` (optional) - Specific public key to use for signing
- `description` (optional) - Human-friendly transaction description
- `callback` (optional) - URL callback for signed transaction

**Critical: Network Passphrase**

You must use the **full network passphrase**, not a short identifier:

**Correct:**

```typescript
// Testnet
network: "Test SDF Network ; September 2015";

// Mainnet
network: "Public Global Stellar Network ; September 2015";
```

**Wrong:**

```typescript
network: "testnet"; // Will cause "Invalid transaction XDR" error
network: "public"; // Will cause "Invalid transaction XDR" error
```

**Returns:**

```typescript
{
  xdr: string;                    // Original transaction XDR
  tx_hash: string;                // HEX-encoded transaction hash
  signed_envelope_xdr: string;    // Signed TransactionEnvelope XDR
  network: string;                // Network passphrase
  result?: object;                // Horizon response (if submit: true)
}
```

**User Experience:**

1. Albedo popup opens
2. Transaction details are displayed
3. User reviews and approves
4. Popup closes and returns signed transaction

### Step 3: Backend Submits Transaction

Submit the signed transaction to the Stellar network.

**Rust code:**

```rust
use stellar_xdr::curr::{TransactionEnvelope, ReadXdr, Limits};

// Parse the signed envelope
let envelope = TransactionEnvelope::from_xdr_base64(signed_xdr, Limits::none())?;

// Submit to Stellar RPC
let response = rpc_client.send_transaction(&envelope).await?;

// Extract transaction hash
let tx_hash = hex::encode(response.0);
```

## Complete Implementation Example

### Backend: Rust/Axum

```rust
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use stellar_xdr::curr::{WriteXdr, Limits, TransactionEnvelope, TransactionV1Envelope};

#[derive(Deserialize)]
struct PrepareRequest {
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub amount: String,
}

#[derive(Serialize)]
struct PrepareResponse {
    pub xdr: String,
    pub network: String,
}

async fn prepare_transaction(
    Json(payload): Json<PrepareRequest>,
) -> Result<Json<PrepareResponse>, StatusCode> {
    // Build transaction
    let transaction = build_transaction(&payload.public_key, &payload.amount)?;

    // Simulate transaction
    let simulation = simulate_transaction(&transaction).await?;

    // Apply simulation results
    let prepared_tx = apply_simulation(transaction, &simulation)?;

    // Wrap in envelope
    let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
        tx: prepared_tx,
        signatures: VecM::default(),
    });

    // Convert to XDR
    let xdr = envelope.to_xdr_base64(Limits::none())?;

    Ok(Json(PrepareResponse {
        xdr,
        network: "Test SDF Network ; September 2015".to_string(),
    }))
}

#[derive(Deserialize)]
struct SubmitRequest {
    #[serde(rename = "signedXdr")]
    pub signed_xdr: String,
}

#[derive(Serialize)]
struct SubmitResponse {
    pub hash: String,
}

async fn submit_transaction(
    Json(payload): Json<SubmitRequest>,
) -> Result<Json<SubmitResponse>, StatusCode> {
    // Parse signed envelope
    let envelope = TransactionEnvelope::from_xdr_base64(
        &payload.signed_xdr,
        Limits::none()
    )?;

    // Submit to network
    let response = rpc_client.send_transaction(&envelope).await?;

    Ok(Json(SubmitResponse {
        hash: hex::encode(response.0),
    }))
}
```

### Frontend: React/TypeScript

```typescript
import albedo from "@albedo-link/intent";
import { useState } from "react";

function TransactionFlow() {
  const [publicKey, setPublicKey] = useState<string>("");
  const [txHash, setTxHash] = useState<string>("");

  const handleTransaction = async () => {
    try {
      // 1. Prepare transaction (backend)
      const prepareResponse = await fetch("/api/plant/prepare", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          publicKey: publicKey,
          amount: "0",
        }),
      });
      const prepareData = await prepareResponse.json();

      // 2. Sign with Albedo
      const signResponse = await albedo.tx({
        xdr: prepareData.xdr,
        network: prepareData.network,
        submit: false,
      });

      // 3. Submit signed transaction (backend)
      const submitResponse = await fetch("/api/plant/submit", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          signedXdr: signResponse.signed_envelope_xdr,
        }),
      });
      const submitData = await submitResponse.json();

      setTxHash(submitData.hash);
      console.log("Success!", submitData.hash);
    } catch (error) {
      console.error("Transaction failed:", error);
    }
  };

  return (
    <button onClick={handleTransaction}>Sign and Submit Transaction</button>
  );
}
```

## Error Handling

### Common Errors

#### 1. "Invalid transaction XDR"

**Cause:**

- Sending raw `Transaction` instead of `TransactionEnvelope`
- Using short network identifier ("testnet") instead of full passphrase

**Solution:**

```rust
// Wrap transaction in envelope
let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
    tx: transaction,
    signatures: VecM::default(),
});
```

```typescript
// Use full network passphrase
network: "Test SDF Network ; September 2015";
```

#### 2. Popup Closes Immediately

**Cause:**

- JavaScript error in frontend
- Invalid parameters passed to Albedo

**Solution:**
Add error handling and logging:

```typescript
try {
  const response = await albedo.tx({
    xdr: txXdr,
    network: networkPassphrase,
    submit: false,
  });
  console.log("Success:", response);
} catch (error) {
  console.error("Albedo error:", error);
  alert(`Error: ${JSON.stringify(error, null, 2)}`);
  throw error;
}
```

#### 3. User Rejection

**Cause:** User clicks "Cancel" or "Reject" in Albedo popup

**Solution:**
Handle the rejection gracefully:

```typescript
try {
  const response = await albedo.tx({...});
} catch (error) {
  if (error.code === -1) {
    console.log('User rejected the transaction');
    // Show user-friendly message
  } else {
    console.error('Transaction error:', error);
  }
}
```

### Debugging Tips

1. **Log everything before calling Albedo:**

```typescript
console.log("Albedo parameters:", {
  xdr: prepareData.xdr,
  network: prepareData.network,
  submit: false,
});
```

2. **Keep popup open on error:**

```typescript
catch (error) {
  alert(`Error: ${JSON.stringify(error)}`); // Prevents popup closing
  console.error(error);
}
```

3. **Validate XDR before sending:**

```rust
// Ensure it can be parsed back
let test_parse = TransactionEnvelope::from_xdr_base64(&xdr, Limits::none())?;
```

## Best Practices

### 1. Always Use Full Network Passphrase

Store network passphrases as constants:

```rust
const TESTNET_PASSPHRASE: &str = "Test SDF Network ; September 2015";
const MAINNET_PASSPHRASE: &str = "Public Global Stellar Network ; September 2015";
```

### 2. Validate Before Signing

Perform all validation on the backend before sending to Albedo:

- Check trustlines
- Verify balances
- Simulate transaction
- Check for errors

### 3. Don't Auto-Submit

Set `submit: false` and submit via your backend for better control:

- Centralized error handling
- Transaction monitoring
- Retry logic
- Better UX feedback

### 4. Provide Transaction Descriptions

Help users understand what they're signing:

```typescript
await albedo.tx({
  xdr: txXdr,
  network: networkPassphrase,
  description: "Plant 0 KALE tokens in farming contract",
  submit: false,
});
```

### 5. Handle Network Changes

Ensure the correct network is being used:

```typescript
const getNetworkPassphrase = (isTestnet: boolean) => {
  return isTestnet
    ? "Test SDF Network ; September 2015"
    : "Public Global Stellar Network ; September 2015";
};
```

## Security Considerations

### 1. Validate Signed Transactions

Always verify the signed transaction matches your expectations:

- Check source account
- Verify operations
- Confirm network

### 2. Don't Trust Client-Side Data

The backend should:

- Re-validate all parameters
- Build transactions from scratch
- Not rely on client-provided XDR

### 3. Show Transaction Details

Before signing, users should see:

- What contract is being invoked
- What parameters are being passed
- What fees will be charged
- The network being used

## Testing

### Test Scenarios

1. **Happy Path:**

   - Authenticate successfully
   - Sign transaction successfully
   - Submit transaction successfully

2. **User Rejection:**

   - User clicks "Cancel" during authentication
   - User clicks "Reject" during signing

3. **Invalid Transaction:**

   - Malformed XDR
   - Wrong network passphrase
   - Missing required fields

4. **Network Errors:**
   - Backend unavailable
   - RPC endpoint down
   - Transaction simulation fails

### Manual Testing Checklist

- [ ] Authentication popup opens
- [ ] Account selection works (if multiple accounts)
- [ ] Public key is returned correctly
- [ ] Transaction popup shows correct details
- [ ] Transaction signature is valid
- [ ] Signed XDR can be parsed
- [ ] Transaction submits successfully
- [ ] Transaction hash is returned
- [ ] Error messages are clear

## Resources

- [Albedo Documentation](https://github.com/stellar-expert/albedo)
- [Stellar Transaction Guide](https://developers.stellar.org/docs/learn/fundamentals/transactions)
- [Soroban RPC Documentation](https://developers.stellar.org/docs/data/rpc)
- [XDR Specification](https://developers.stellar.org/docs/encyclopedia/xdr)

## Troubleshooting

### Transaction Fails to Sign

1. Check browser console for JavaScript errors
2. Verify XDR is valid TransactionEnvelope
3. Confirm network passphrase is correct
4. Test with a simple transaction first

### Popup Blocked

Some browsers block popups by default:

1. Check browser popup blocker settings
2. Trigger Albedo from user action (button click)
3. Don't call Albedo from page load or timers

### Signature Invalid

1. Ensure transaction hasn't been modified after signing
2. Check network passphrase matches when submitting
3. Verify the signed envelope includes the signature

### Transaction Rejected by Network

1. Simulate transaction before signing
2. Check account has sufficient XLM for fees
3. Verify all required trustlines exist
4. Check transaction sequence number

## Summary

Albedo provides a secure, user-friendly way to sign Stellar transactions without exposing
private keys. Key points to remember:

- Use **TransactionEnvelope** format, not raw Transaction
- Provide **full network passphrase**, not short identifiers
- **Simulate** before signing to calculate fees
- **Validate** everything on the backend
- **Handle errors** gracefully with good UX
- **Log** extensively for debugging

Following these practices will result in a robust, secure integration with Albedo wallet.
