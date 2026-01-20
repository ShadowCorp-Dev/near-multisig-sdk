# Multisig Shell Scripts

CLI helpers for technical users who prefer terminal commands over web UIs.

## Setup

Make scripts executable:

```bash
chmod +x *.sh
```

Set your NEAR account (for submit/approve):

```bash
export NEAR_ACCOUNT=your-account.near
```

## Usage

### Main Wrapper

```bash
./multisig.sh <command> [args]
```

### View Pending Transactions

```bash
./multisig.sh view-pending multisig.testnet

# Or directly:
./view-pending.sh multisig.testnet
```

Output shows all pending transactions with their IDs, receivers, actions, and confirmations.

### Submit Transaction

```bash
./multisig.sh submit multisig.testnet recipient.near 5

# Or directly:
./submit.sh multisig.testnet recipient.near 5
```

Submits a new transaction to send 5 NEAR from the multisig to recipient.near.

### Approve Transaction

```bash
./multisig.sh approve multisig.testnet 0

# Or directly:
./approve.sh multisig.testnet 0
```

Approves transaction #0. When enough owners approve, it auto-executes.

## Scripts

### multisig.sh
Main wrapper that routes to individual scripts. Use this for convenience.

### view-pending.sh
View all pending transactions.
- Requires: multisig address
- Uses: `near view` (read-only, no gas)

### submit.sh
Submit a new transfer transaction.
- Requires: multisig address, receiver, amount in NEAR
- Uses: `near call` (requires gas, signs as NEAR_ACCOUNT)
- Note: Amount is in NEAR, not yoctoNEAR

### approve.sh
Approve a pending transaction.
- Requires: multisig address, transaction ID
- Uses: `near call` (requires gas, signs as NEAR_ACCOUNT)

## Requirements

- NEAR CLI installed (`npm install -g near-cli`)
- `jq` for JSON parsing (optional, for better formatting)
- `bc` for amount calculations (used by submit.sh)

## Examples

**Complete workflow:**

```bash
# Set your account
export NEAR_ACCOUNT=alice.near

# View current pending transactions
./multisig.sh view-pending dao-treasury.near

# Submit a new transaction (Alice)
./multisig.sh submit dao-treasury.near recipient.near 10

# View pending (should see transaction #0)
./multisig.sh view-pending dao-treasury.near

# Approve as Bob
export NEAR_ACCOUNT=bob.near
./multisig.sh approve dao-treasury.near 0

# If 2-of-3, this will execute the transaction
```

**Advanced: Submit function call**

For function calls, you'll need to manually craft the JSON:

```bash
near call multisig.testnet submit_transaction '{
  "receiver_id": "contract.testnet",
  "actions": [{
    "FunctionCall": {
      "method_name": "do_something",
      "args": [123, 456],
      "gas": 30000000000000,
      "deposit": "0"
    }
  }]
}' --accountId $NEAR_ACCOUNT
```

## Troubleshooting

**"NEAR_ACCOUNT not set"**
- Run: `export NEAR_ACCOUNT=your-account.near`

**"Not an owner"**
- Verify you're using an account that was added as an owner
- Check: `near view multisig.near get_owners`

**"Already confirmed"**
- You already approved this transaction
- Check confirmations: `./view-pending.sh multisig.near`

**"Transaction not found"**
- Wrong transaction ID
- Transaction may have been executed already

## Alternative: Web UI

If you prefer a graphical interface, use the frontend:

```bash
cd ../templates/basic/frontend
npm install
npm run dev
```

See `../templates/basic/frontend/README.md` for details.
