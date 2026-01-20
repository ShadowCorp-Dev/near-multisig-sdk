# Multisig Initialization & Usage Guide

Complete guide for deploying, initializing, and using your NEAR multisig contract.

## How It Works

**The multisig contract becomes the controlled account:**

When you deploy a multisig contract to `treasury.dao.near`, that account is now controlled by the contract's logic. The owner wallets (alice, bob, charlie) can submit and approve transactions, but they sign FROM THEIR OWN WALLETS. The actual execution happens from the multisig account.

**Flow:**
1. Alice (from alice.near) → calls `submit_transaction` on treasury.dao.near
2. Bob (from bob.near) → calls `confirm_transaction` on treasury.dao.near (reaches threshold)
3. Charlie (from charlie.near) → calls `execute_transaction` → treasury.dao.near sends the funds

The owners never hold keys to the treasury account - they control it through the contract.

## Quick Start

```bash
# 1. Build the contract
cd my-multisig
near-multisig build

# 2. Deploy to the account you want to protect
near deploy --accountId treasury.dao.near --wasmFile release/my_multisig.wasm

# 3. Initialize with owner wallets (who will control it)
near call treasury.dao.near new '{
  "owners": ["alice.near", "bob.near", "charlie.near"],
  "num_confirmations": 2
}' --accountId treasury.dao.near

# 4. (Recommended) Delete old full access keys
near keys treasury.dao.near
near delete-key treasury.dao.near ed25519:YourOldKey... --accountId treasury.dao.near
```

Done! Now alice, bob, and charlie collectively control treasury.dao.near.

### Funding the Multisig

The multisig account needs NEAR balance for:
- Executing transactions (gas costs)
- Sending funds (if it's a treasury)

**Anyone can send funds to it:**
```bash
# From your personal wallet
near send alice.near treasury.dao.near 100

# From another account
near send company.near treasury.dao.near 1000
```

The multisig contract doesn't need to approve incoming transfers - it only requires approvals for OUTGOING transactions.

---

## Template-Specific Initialization

### Basic Multisig

**Use case:** Simple M-of-N approval (e.g., 3 of 5 owners must approve)

```bash
near call my-multisig.near new '{
  "owners": [
    "alice.near",
    "bob.near",
    "charlie.near",
    "dave.near",
    "eve.near"
  ],
  "num_confirmations": 3
}' --accountId my-multisig.near
```

**Parameters:**
- `owners` - Array of NEAR account IDs who can approve transactions
- `num_confirmations` - Number of approvals needed (must be ≤ number of owners)

### Timelock Multisig

**Use case:** Mandatory delay after approval (security for high-risk operations)

```bash
near call timelock.near new '{
  "owners": ["alice.near", "bob.near", "charlie.near"],
  "num_confirmations": 2,
  "timelock_duration": 172800000000000
}' --accountId timelock.near
```

**Parameters:**
- `owners` - Array of owner account IDs
- `num_confirmations` - Approvals needed before scheduling
- `timelock_duration` - Delay in **nanoseconds** before execution

**Common timelock durations:**
- 1 hour: `3600000000000`
- 24 hours: `86400000000000`
- 48 hours: `172800000000000`
- 7 days: `604800000000000`

### Weighted Multisig

**Use case:** Different voting power per owner (token-based governance)

```bash
near call weighted.near new '{
  "owners_with_weights": [
    ["whale.near", 50],
    ["medium1.near", 25],
    ["medium2.near", 15],
    ["small.near", 10]
  ],
  "approval_threshold": 60
}' --accountId weighted.near
```

**Parameters:**
- `owners_with_weights` - Array of `[account_id, weight]` pairs
- `approval_threshold` - Total weight needed to execute (60 = need 60% of votes)

**Example calculations:**
- whale (50) + medium1 (25) = 75 ✓ Passes
- medium1 (25) + medium2 (15) + small (10) = 50 ✗ Below 60
- whale (50) + medium2 (15) = 65 ✓ Passes

---

## Using Your Multisig

### Understanding Who Signs What

**Important:** Owners sign confirmations from their own wallets, but the multisig account executes the transaction.

```
Scenario: treasury.dao.near (multisig) wants to send 10 NEAR to recipient.near
Owners: alice.near, bob.near, charlie.near (need 2 confirmations)

┌─────────────────────────────────────────────────────────┐
│ Step 1: Alice submits (signs from alice.near)          │
│ → Pays gas from alice.near                              │
│ → Creates transaction #0 in treasury.dao.near contract  │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│ Step 2: Bob confirms (signs from bob.near)             │
│ → Pays gas from bob.near                                │
│ → Reaches 2/3 threshold (ready to execute)              │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│ Step 3: Charlie executes (signs from charlie.near)     │
│ → Pays gas from charlie.near                            │
│ → Contract sends 10 NEAR FROM treasury.dao.near         │
│ → recipient.near receives 10 NEAR                       │
└─────────────────────────────────────────────────────────┘
```

**Key point:** Alice and Bob only pay gas for their confirmation calls. The actual transfer comes from the multisig account balance.

### Submit a Transaction

Any owner can submit a transaction for approval (requires 0.01 NEAR storage deposit):

```bash
near call my-multisig.near submit_transaction '{
  "receiver_id": "recipient.near",
  "actions": [{
    "Transfer": {"amount": "5000000000000000000000000"}
  }],
  "expiration_hours": null
}' --accountId alice.near --deposit 0.01
```

This creates transaction #0 (first transaction) waiting for confirmations.

### Confirm a Transaction

Other owners approve by confirming:

```bash
# Bob confirms
near call my-multisig.near confirm_transaction '{
  "tx_id": 0
}' --accountId bob.near

# Charlie confirms (reaches threshold if 3 needed)
near call my-multisig.near confirm_transaction '{
  "tx_id": 0
}' --accountId charlie.near
```

**Basic multisig:** Must call `execute_transaction` after threshold is reached

**Timelock multisig:** Must call `execute_transaction` after timelock delay expires

### Execute Timelock Transaction

After the timelock expires, anyone can execute:

```bash
near call timelock.near execute_transaction '{
  "tx_id": 0
}' --accountId anyone.near --gas 100000000000000
```

### View Functions

Check multisig state without spending gas:

```bash
# List all owners
near view my-multisig.near get_owners

# Check specific transaction
near view my-multisig.near get_transaction '{"tx_id": 0}'

# Get confirmation threshold
near view my-multisig.near get_num_confirmations

# (Weighted only) Check voting weights
near view weighted.near get_owner_weight '{"owner": "whale.near"}'
```

---

## Common Action Types

### Transfer NEAR

Send NEAR tokens:

```json
{
  "Transfer": {
    "amount": "5000000000000000000000000"
  }
}
```

Amount is in yoctoNEAR (1 NEAR = 10^24 yoctoNEAR)

**Common amounts:**
- 1 NEAR: `"1000000000000000000000000"`
- 10 NEAR: `"10000000000000000000000000"`
- 100 NEAR: `"100000000000000000000000000"`

### Function Call

Call a method on another contract:

```json
{
  "FunctionCall": {
    "method_name": "set_value",
    "args": [101, 121, 34, 107, 101, 121, 34, 58, 34, 118, 97, 108, 117, 101, 34, 125],
    "gas": 30000000000000,
    "deposit": "0"
  }
}
```

- `args` must be base64-encoded JSON
- `gas` in gas units (30 TGas = 30000000000000)
- `deposit` in yoctoNEAR

### Add Key

Add an access key to the multisig account:

```json
{
  "type": "AddKey",
  "public_key": "ed25519:...",
  "permission": "FullAccess"
}
```

### Delete Key

Remove an access key:

```json
{
  "type": "DeleteKey",
  "public_key": "ed25519:..."
}
```

---

## Complete Examples

### DAO Treasury (3-of-5 multisig)

```bash
# Deploy
near deploy --accountId dao-treasury.near --wasmFile release/dao_treasury.wasm

# Initialize with council members
near call dao-treasury.near new '{
  "owners": [
    "council1.near",
    "council2.near",
    "council3.near",
    "council4.near",
    "council5.near"
  ],
  "num_confirmations": 3
}' --accountId dao-treasury.near

# Submit grant proposal (50 NEAR) - requires 0.01 NEAR deposit
near call dao-treasury.near submit_transaction '{
  "receiver_id": "grantee.near",
  "actions": [{
    "Transfer": {"amount": "50000000000000000000000000"}
  }],
  "expiration_hours": null
}' --accountId council1.near --deposit 0.01

# Approve (need 3 total)
near call dao-treasury.near confirm_transaction '{"tx_id": 0}' --accountId council2.near
near call dao-treasury.near confirm_transaction '{"tx_id": 0}' --accountId council3.near

# Execute after threshold reached
near call dao-treasury.near execute_transaction '{"tx_id": 0}' --accountId council1.near --gas 100000000000000
```

### Protocol Upgrade (Timelock, 48h delay)

```bash
# Deploy
near deploy --accountId protocol.near --wasmFile release/protocol.wasm

# Initialize with 48-hour timelock
near call protocol.near new '{
  "owners": ["dev-team.near", "security-team.near"],
  "num_confirmations": 2,
  "timelock_duration": 172800000000000
}' --accountId protocol.near

# Submit upgrade call (requires 0.01 NEAR deposit)
near call protocol.near submit_transaction '{
  "receiver_id": "main-contract.near",
  "actions": [{
    "FunctionCall": {
      "method_name": "upgrade",
      "args": [123, 125],
      "gas": 300000000000000,
      "deposit": "0"
    }
  }],
  "expiration_hours": null
}' --accountId dev-team.near --deposit 0.01

# Security team confirms
near call protocol.near confirm_transaction '{"tx_id": 0}' --accountId security-team.near

# Transaction is now READY (but timelock not expired yet)
# Wait 48 hours...

# After 48 hours, execute
near call protocol.near execute_transaction '{"tx_id": 0}' --accountId anyone.near --gas 100000000000000
```

### Token Governance (Weighted voting)

```bash
# Deploy
near deploy --accountId token-gov.near --wasmFile release/token_gov.wasm

# Initialize with token holder weights
near call token-gov.near new '{
  "owners_with_weights": [
    ["whale.near", 40],
    ["medium1.near", 25],
    ["medium2.near", 20],
    ["small1.near", 10],
    ["small2.near", 5]
  ],
  "approval_threshold": 51
}' --accountId token-gov.near

# Submit proposal (requires 0.01 NEAR deposit)
near call token-gov.near submit_transaction '{
  "receiver_id": "treasury.near",
  "actions": [{
    "Transfer": {"amount": "100000000000000000000000000"}
  }],
  "expiration_hours": null
}' --accountId whale.near --deposit 0.01

# Whale approves (40 weight)
near call token-gov.near approve_transaction '{"tx_id": 0}' --accountId whale.near

# Medium1 approves (25 weight) = 65 total (> 51 threshold)
near call token-gov.near approve_transaction '{"tx_id": 0}' --accountId medium1.near

# Execute after threshold reached
near call token-gov.near execute_transaction '{"tx_id": 0}' --accountId anyone.near --gas 100000000000000
```

---

## Security Best Practices

**All contracts include security hardening:**
- Transaction ID overflow protection
- Promise callback failure handling
- Checked arithmetic (no overflows)
- Input validation on all parameters

See [SECURITY.md](SECURITY.md) for audit details and contract limits.

**Choose appropriate thresholds:**
- 2-of-3 minimum for small teams
- 3-of-5 or higher for DAOs
- Never use 1-of-N (defeats multisig purpose)

**Timelock durations:**
- 24-48 hours for protocol upgrades
- 7 days for critical changes
- Balance security vs operational speed

**Owner selection:**
- Distribute across trusted parties
- Use hardware wallets for high-value multisigs
- Have backup owners in case of key loss

**Testing:**
- Test on testnet first
- Start with small transfers
- Verify all owners can sign

---

## Troubleshooting

**"Invalid confirmation threshold"**
- `num_confirmations` must be > 0 and ≤ number of owners
- Example: Can't do 4-of-3, must be at most 3-of-3

**"Already initialized"**
- Contract can only be initialized once
- Redeploy if you need different owners

**"Not an owner"**
- Only accounts in the `owners` list can submit/confirm
- Check: `near view my-multisig.near get_owners`

**"Timelock not expired"**
- Must wait full duration before executing
- Check transaction status: `near view multisig.near get_transaction '{"tx_id": 0}'`

**"Already executed"**
- Transaction already completed
- Check with `get_transaction` view call

---

## Advanced: Multiple Actions

Submit a transaction with multiple actions (atomic batch):

```bash
near call my-multisig.near submit_transaction '{
  "receiver_id": "recipient.near",
  "actions": [
    {
      "Transfer": {"amount": "5000000000000000000000000"}
    },
    {
      "FunctionCall": {
        "method_name": "on_receive",
        "args": [123, 125],
        "gas": 10000000000000,
        "deposit": "0"
      }
    }
  ],
  "expiration_hours": null
}' --accountId alice.near --deposit 0.01
```

All actions execute together or all fail (atomicity).

---

**Need help?** Check [README.md](README.md) for SDK usage or [EXAMPLES.md](EXAMPLES.md) for more patterns.
