# Weighted Multisig Contract

Multisig with different voting weights per owner. Execution happens when total approval weight reaches threshold.

## Features

- **Weighted Voting**: Each owner has different voting power
- **Weight Threshold**: Transaction executes when total weight >= threshold
- **Auto-execution**: Immediate execution upon reaching threshold
- **View Methods**: Frontend-friendly query methods

## Use Cases

- Token holder governance (voting power proportional to holdings)
- Stakeholder voting (different decision power per stakeholder)
- Equity-based control (proportional to ownership percentage)
- Tiered access systems (admin, moderator, contributor weights)

## Build

```bash
chmod +x build.sh
./build.sh
```

Output: `target/near/weighted_multisig.wasm`

## Deploy

```bash
near deploy --accountId your-multisig.testnet --wasmFile target/near/weighted_multisig.wasm
```

## Initialize

```bash
near call your-multisig.testnet new '{
  "owners_with_weights": [
    ["alice.near", 40],
    ["bob.near", 30],
    ["charlie.near", 20],
    ["dave.near", 10]
  ],
  "approval_threshold": 60
}' --accountId your-multisig.testnet
```

**Parameters:**
- `owners_with_weights` - Array of [account_id, weight] tuples
- `approval_threshold` - Total weight needed for execution

**Example weight distributions:**
- **Majority vote**: Alice=50, Bob=30, Charlie=20, threshold=60 (Alice+Bob OR Alice+Charlie)
- **Token-based**: Weights = token holdings, threshold = 50% of total supply
- **Tiered**: Admin=100, Mod=50, User=10, threshold=150 (2 admins OR 3 mods, etc.)

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

Submitter automatically approves with their weight. Returns transaction ID.

### 2. Approve Transaction

```bash
near call your-multisig.testnet approve_transaction '{
  "tx_id": 0
}' --accountId bob.near
```

When total weight >= threshold, **transaction auto-executes**.

## View Methods

### Get Owners and Weights

```bash
near view your-multisig.testnet get_owners
```

Returns:
```json
[
  ["alice.near", 40],
  ["bob.near", 30],
  ["charlie.near", 20],
  ["dave.near", 10]
]
```

### Get Owner Weight

```bash
near view your-multisig.testnet get_owner_weight '{"account_id": "alice.near"}'
```

Returns: `40`

### Get Approval Threshold

```bash
near view your-multisig.testnet get_approval_threshold
```

### Get Total Weight

```bash
near view your-multisig.testnet get_total_weight
```

Returns sum of all owner weights.

### Get Pending Transactions

```bash
near view your-multisig.testnet get_pending_transactions
```

### Get Transaction Progress

```bash
near view your-multisig.testnet get_transaction_progress '{"tx_id": 0}'
```

Returns: `[current_weight, threshold]` (e.g., `[50, 60]`)

### Check if Account is Owner

```bash
near view your-multisig.testnet is_owner '{"account_id": "alice.near"}'
```

### Check Approval Status

```bash
near view your-multisig.testnet has_approved '{
  "tx_id": 0,
  "account_id": "alice.near"
}'
```

## Frontend Integration

See [../frontend/README.md](../frontend/README.md) for web UI.

The frontend displays:
- Weight-based progress bars
- Approval percentage (current_weight / threshold * 100%)
- List of approvers with their weights

## CLI Scripts

See [../../scripts/README.md](../../scripts/README.md) for shell helpers.

**Note:** Weighted multisig uses same script interface as basic multisig. The contract automatically handles weight calculations.

## Example Scenarios

### Scenario 1: Token Governance

Setup: Weighted by token holdings
- Alice: 1000 tokens = weight 1000
- Bob: 500 tokens = weight 500
- Charlie: 300 tokens = weight 300
- Threshold: 900 (50% of 1800 total)

Approval flow:
1. Alice submits (1000 weight)
2. Bob approves (+500 weight = 1500 total)
3. **Executes** (1500 >= 900)

### Scenario 2: Equity-Based

Setup: Weighted by ownership percentage
- Founder A: 60% = weight 60
- Founder B: 30% = weight 30
- Investor: 10% = weight 10
- Threshold: 51 (majority)

Approval flow:
1. Investor submits (10 weight)
2. Founder B approves (+30 weight = 40 total)
3. Founder A approves (+60 weight = 100 total)
4. **Executes** (100 >= 51)

### Scenario 3: Tiered Roles

Setup: Role-based weights
- Admin: 100
- Senior Mod: 50
- Junior Mod: 25
- Threshold: 100 (1 admin OR 2 senior mods OR 4 junior mods)

Approval flow:
1. Junior Mod submits (25 weight)
2. Another Junior Mod approves (+25 = 50)
3. Senior Mod approves (+50 = 100)
4. **Executes** (100 >= 100)

## Comparison with Other Templates

| Feature | Basic | Weighted | Timelock |
|---------|-------|----------|----------|
| **Voting** | Equal | Weighted | Equal |
| **Threshold** | Count | Weight sum | Count |
| **Execution** | Immediate | Immediate | After delay |
| **Use Case** | General | Governance | Security |

## Customization

### Dynamic Weight Updates

Add governance method to update weights:

```rust
pub fn update_owner_weight(&mut self, owner: AccountId, new_weight: u32) {
    // Require governance approval
    require!(self.owner_weights.contains_key(&owner), "Not an owner");
    require!(new_weight > 0, "Weight must be positive");

    self.owner_weights.insert(owner, new_weight);
}
```

### Add New Owner

```rust
pub fn add_owner(&mut self, new_owner: AccountId, weight: u32) {
    // Require governance approval
    require!(!self.owner_weights.contains_key(&new_owner), "Already an owner");
    require!(weight > 0, "Weight must be positive");

    self.owner_weights.insert(new_owner, weight);
}
```

### Change Threshold

```rust
pub fn update_threshold(&mut self, new_threshold: u32) {
    // Require governance approval
    let total_weight = self.get_total_weight();
    require!(new_threshold <= total_weight, "Threshold exceeds total weight");
    require!(new_threshold > 0, "Threshold must be positive");

    self.approval_threshold = new_threshold;
}
```

## Testing

```bash
# Build
./build.sh

# Deploy to testnet
near deploy --accountId weighted-multisig.testnet --wasmFile target/near/weighted_multisig.wasm

# Initialize
near call weighted-multisig.testnet new '{
  "owners_with_weights": [
    ["alice.testnet", 50],
    ["bob.testnet", 30],
    ["charlie.testnet", 20]
  ],
  "approval_threshold": 60
}' --accountId weighted-multisig.testnet

# Submit transaction (Alice auto-approves with 50 weight)
near call weighted-multisig.testnet submit_transaction '{
  "receiver_id": "recipient.testnet",
  "actions": [{"Transfer": {"amount": "1000000000000000000000000"}}]
}' --accountId alice.testnet

# Bob approves (adds 30 weight = 80 total, executes!)
near call weighted-multisig.testnet approve_transaction '{"tx_id": 0}' --accountId bob.testnet
```

## Security Considerations

**Centralization Risk:**
- If one owner has weight >= threshold, they control all decisions
- Design threshold to require multiple parties

**Weight Distribution:**
- Total weight should reflect actual stake/power
- Regular weight updates for token-based systems
- Prevent weight concentration attacks

**Threshold Safety:**
- Threshold too low = easy execution
- Threshold too high = deadlock risk
- Common: 50-66% of total weight

## Best Practices

1. **Initial Distribution**: Carefully set initial weights to match real stake
2. **Threshold Choice**: 50% for simple majority, 66% for supermajority
3. **Regular Review**: Update weights as stakes change (for token systems)
4. **Transparency**: Make weight distribution publicly visible
5. **Emergency Threshold**: Consider lower threshold for emergency actions

## License

MIT
