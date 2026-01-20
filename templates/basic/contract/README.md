# Basic Multisig Contract

Simple M-of-N multisig contract where M owners must approve before a transaction can be executed.

## Features

- **M-of-N approval** - Require specific number of confirmations
- **Manual execution** - Transaction must be explicitly executed after approval threshold is reached
- **Multiple action types** - Transfer NEAR, call functions
- **Transaction expiration** - Optional time-based expiration for transactions
- **Owner management** - Add/remove owners and change approval threshold
- **Storage management** - Cleanup old transactions to reduce storage costs
- **Revocation support** - Owners can revoke their confirmations before execution
- **O(1) transaction lookups** - Optimized for performance at scale
- **State migration** - Contract can be upgraded without losing pending transactions

## Build

```bash
./build.sh
```

## Deploy

```bash
near deploy --accountId your-multisig.near --wasmFile target/near/basic_multisig.wasm
```

## Initialize

```bash
near call your-multisig.near new '{
  "owners": ["alice.near", "bob.near", "charlie.near"],
  "num_confirmations": 2
}' --accountId your-multisig.near
```

## Usage

### Submit Transaction

**IMPORTANT**: Requires 0.01 NEAR storage deposit (refunded on execution/cancellation)

```bash
# Submit without expiration
near call your-multisig.near submit_transaction '{
  "receiver_id": "recipient.near",
  "actions": [{
    "Transfer": {
      "amount": "1000000000000000000000000"
    }
  }],
  "expiration_hours": null
}' --accountId alice.near --deposit 0.01

# Submit with 24-hour expiration
near call your-multisig.near submit_transaction '{
  "receiver_id": "recipient.near",
  "actions": [{
    "Transfer": {
      "amount": "1000000000000000000000000"
    }
  }],
  "expiration_hours": 24
}' --accountId alice.near --deposit 0.01
```

### Confirm Transaction

```bash
near call your-multisig.near confirm_transaction '{
  "tx_id": 0
}' --accountId bob.near
```

### Execute Transaction

After threshold is reached, any owner can execute:

```bash
near call your-multisig.near execute_transaction '{
  "tx_id": 0
}' --accountId bob.near --gas 100000000000000
```

### Cancel Transaction

Only the original submitter can cancel:

```bash
near call your-multisig.near cancel_transaction '{
  "tx_id": 0
}' --accountId alice.near
```

### Revoke Confirmation

Any owner who confirmed can revoke their confirmation:

```bash
near call your-multisig.near revoke_confirmation '{
  "tx_id": 0
}' --accountId bob.near
```

### Owner Management

```bash
# Add a new owner
near call your-multisig.near add_owner '{
  "new_owner": "dave.near"
}' --accountId alice.near

# Remove an owner
near call your-multisig.near remove_owner '{
  "owner_to_remove": "dave.near"
}' --accountId alice.near

# Change approval threshold
near call your-multisig.near change_threshold '{
  "new_threshold": 3
}' --accountId alice.near
```

### Storage Management

```bash
# Cleanup old executed/cancelled transactions
near call your-multisig.near cleanup_old_transactions '{
  "before_index": 100
}' --accountId alice.near --gas 100000000000000
```

### View Methods

```bash
# View pending transactions
near view your-multisig.near get_pending_transactions

# Get specific transaction
near view your-multisig.near get_transaction '{"tx_id": 0}'

# Get storage deposit requirement
near view your-multisig.near get_storage_deposit
```

## Contract Methods

### Initialization

- `new(owners, num_confirmations)` - Initialize contract with owners and approval threshold
- `migrate()` - Migrate contract state to new version (owner-only, requires contract upgrade)

### Transaction Management

- `submit_transaction(receiver_id, actions, expiration_hours)` - Submit new transaction (requires 0.01 NEAR deposit)
- `confirm_transaction(tx_id)` - Confirm pending transaction
- `execute_transaction(tx_id)` - Execute fully-approved transaction (manual execution required)
- `cancel_transaction(tx_id)` - Cancel transaction (submitter-only, refunds deposit)
- `revoke_confirmation(tx_id)` - Revoke your confirmation from a pending transaction

### Owner Management (Owner-Only)

- `add_owner(new_owner)` - Add a new owner to the multisig
- `remove_owner(owner_to_remove)` - Remove an owner (cannot reduce below threshold)
- `change_threshold(new_threshold)` - Change the number of required confirmations

### Configuration (Owner-Only)

- `set_callback_gas(gas)` - Adjust gas allocated for execution callbacks
- `set_storage_deposit(deposit)` - Adjust storage deposit requirement (0.001-1 NEAR)

### Storage Management

- `cleanup_old_transactions(before_index)` - Remove old executed/cancelled transactions (owner-only)

### View Methods

- `get_owners()` - List all owners
- `get_num_confirmations()` - Get approval threshold
- `get_transaction(tx_id)` - Get specific transaction by ID (O(1) lookup)
- `get_pending_transactions()` - Get all pending transactions
- `get_transactions(from_index, limit)` - Get paginated transactions
- `get_transaction_count()` - Total transaction count
- `is_owner(account_id)` - Check if account is an owner
- `get_storage_deposit()` - Get current storage deposit requirement
- `get_callback_gas()` - Get current callback gas allocation

## Security Features

- **Storage deposit anti-spam** - Requires 0.01 NEAR deposit to submit (refunded on execution/cancellation)
- **Minimum contract balance** - Enforces 0.1 NEAR minimum balance to prevent contract drain
- **Maximum transaction limit** - Enforces 1000 transaction limit to prevent unbounded storage
- **Transaction expiration** - Optional time-based expiration to prevent stale transactions
- **Owner validation** - All mutations restricted to contract owners
- **Submitter-only cancellation** - Only original submitter can cancel their transaction
- **Balance reservation** - Pending transaction deposits are reserved to prevent double-spending
