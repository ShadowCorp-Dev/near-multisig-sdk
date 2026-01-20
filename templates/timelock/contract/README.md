# Timelock Multisig Contract

M-of-N multisig with mandatory delay between approval and execution.

## Features

- **M-of-N Approval**: Transaction requires M confirmations from N owners
- **Timelock Delay**: Mandatory waiting period after approval before execution
- **Scheduled Execution**: Anyone can execute once timelock expires
- **View Methods**: Frontend-friendly query methods

## Use Cases

- Protocol upgrades with safety delay
- High-value transactions requiring review period
- Security-critical operations with cancellation window
- DAO governance with execution delay

## Build

```bash
chmod +x build.sh
./build.sh
```

Output: `target/near/timelock_multisig.wasm`

## Deploy

```bash
near deploy --accountId your-multisig.testnet --wasmFile target/near/timelock_multisig.wasm
```

## Initialize

```bash
near call your-multisig.testnet new '{
  "owners": ["alice.near", "bob.near", "charlie.near"],
  "num_confirmations": 2,
  "timelock_duration": 86400000000000
}' --accountId your-multisig.testnet
```

**Parameters:**
- `owners` - Array of account IDs that can approve transactions
- `num_confirmations` - Number of approvals required (M)
- `timelock_duration` - Delay in nanoseconds (86400000000000 = 1 day)

**Common timelock durations:**
- 1 hour: 3600000000000
- 1 day: 86400000000000
- 1 week: 604800000000000

## Usage

### 1. Submit Transaction

```bash
near call your-multisig.testnet submit_transaction '{
  "receiver_id": "recipient.near",
  "actions": [{
    "Transfer": {
      "amount": "5000000000000000000000000"
    }
  }]
}' --accountId alice.near
```

Returns transaction ID (e.g., 0).

### 2. Approve Transaction

Other owners confirm:

```bash
near call your-multisig.testnet confirm_transaction '{
  "tx_id": 0
}' --accountId bob.near
```

When threshold is reached, transaction is **scheduled** (not executed).

### 3. Wait for Timelock

Check when transaction can be executed:

```bash
near view your-multisig.testnet get_transaction '{"tx_id": 0}'
```

Look at `scheduled_time` field. Transaction is executable when current time ≥ scheduled time.

### 4. Execute Transaction

After timelock expires, **anyone** can execute:

```bash
near call your-multisig.testnet execute_transaction '{
  "tx_id": 0
}' --accountId anyone.near
```

## View Methods

### Get Pending Transactions

```bash
near view your-multisig.testnet get_pending_transactions
```

Returns all unexecuted transactions.

### Get Scheduled Transactions

```bash
near view your-multisig.testnet get_scheduled_transactions
```

Returns transactions that reached threshold and are scheduled.

### Get Executable Transactions

```bash
near view your-multisig.testnet get_executable_transactions
```

Returns transactions ready to execute (timelock expired).

### Get Owners

```bash
near view your-multisig.testnet get_owners
```

### Check if Account is Owner

```bash
near view your-multisig.testnet is_owner '{"account_id": "alice.near"}'
```

### Check Confirmation Status

```bash
near view your-multisig.testnet has_confirmed '{
  "tx_id": 0,
  "account_id": "alice.near"
}'
```

### Get Transaction Details

```bash
near view your-multisig.testnet get_transaction '{"tx_id": 0}'
```

Returns:
```json
{
  "receiver_id": "recipient.near",
  "actions": [...],
  "confirmations": ["alice.near", "bob.near"],
  "scheduled_time": 1234567890000000000,
  "executed": false
}
```

## Frontend Integration

See [../frontend/README.md](../frontend/README.md) for web UI.

The frontend displays:
- Pending transactions
- Scheduled transactions (with countdown)
- Executable transactions (execute button enabled)
- Confirmation status

## CLI Scripts

See [../../scripts/README.md](../../scripts/README.md) for shell helpers.

Additional timelock-specific scripts:
- `view-scheduled.sh` - List scheduled transactions
- `execute.sh` - Execute timelock-expired transaction

## Differences from Basic Multisig

| Feature | Basic | Timelock |
|---------|-------|----------|
| **Execution** | Immediate | After delay |
| **Scheduling** | Auto-execute | Manual execute |
| **Use Case** | General | Security |
| **Safety** | Approval only | Approval + time |

## Security Considerations

**Benefits:**
- Prevents hasty execution of critical actions
- Provides window to detect malicious proposals
- Allows time for owner review

**Trade-offs:**
- Slower execution (by design)
- Requires someone to call execute after timelock
- No built-in cancellation (would need governance extension)

## Customization

### Change Timelock Duration

Modify `timelock_duration` in initialization. Or add a governance method:

```rust
pub fn update_timelock_duration(&mut self, new_duration: u64) {
    require!(self.owners.contains(&env::predecessor_account_id()), "Not an owner");
    // Add additional governance logic here
    self.timelock_duration = new_duration;
}
```

### Add Cancellation

Add cancel method for emergency stops:

```rust
pub fn cancel_transaction(&mut self, tx_id: u64) {
    require!(self.owners.contains(&env::predecessor_account_id()), "Not an owner");

    let mut tx = self.transactions.get(tx_id as u32).expect("Transaction not found").clone();
    require!(!tx.executed, "Already executed");

    // Require unanimous consent to cancel
    // Or add separate cancellation threshold
    tx.executed = true; // Mark as executed to prevent execution
    self.transactions.replace(tx_id as u32, tx);
}
```

## Testing

```bash
# Build
./build.sh

# Deploy to testnet
near deploy --accountId timelock-multisig.testnet --wasmFile target/near/timelock_multisig.wasm

# Initialize with 1-hour timelock
near call timelock-multisig.testnet new '{
  "owners": ["alice.testnet", "bob.testnet"],
  "num_confirmations": 2,
  "timelock_duration": 3600000000000
}' --accountId timelock-multisig.testnet

# Submit transaction
near call timelock-multisig.testnet submit_transaction '{
  "receiver_id": "recipient.testnet",
  "actions": [{"Transfer": {"amount": "1000000000000000000000000"}}]
}' --accountId alice.testnet

# Confirm (schedules it)
near call timelock-multisig.testnet confirm_transaction '{"tx_id": 0}' --accountId bob.testnet

# Wait 1 hour...

# Execute
near call timelock-multisig.testnet execute_transaction '{"tx_id": 0}' --accountId anyone.testnet
```

## License

MIT
