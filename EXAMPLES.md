# Multisig Examples

Real-world examples showing how to use each template.

> **Security Note:** All templates include security hardening (overflow protection, callback handling, input validation). See [SECURITY.md](SECURITY.md) for details.

## Two Ways to Start

**Option 1: CLI (Fast)**
```bash
near-multisig init my-project --template basic
```
Creates a contract project. Good for developers.

**Option 2: Copy Template (Complete)**
```bash
cp -r templates/basic my-project
```
Includes contract + frontend + scripts. Good for teams.

---

## 1. DAO Treasury (Basic)

**What:** 5-person council needs 3 approvals to spend funds.

**Use:** Community DAOs, team wallets, shared treasuries.

### Quick Start

```bash
# CLI approach
near-multisig init dao-treasury --template basic
cd dao-treasury
near-multisig build

# OR copy template with frontend
cp -r templates/basic dao-treasury
cd dao-treasury/contract
./build.sh
```

### Deploy & Initialize

```bash
# Deploy contract
near deploy --accountId dao-treasury.testnet \
  --wasmFile release/dao_treasury.wasm

# Initialize with 5 owners, need 3 approvals
near call dao-treasury.testnet new '{
  "owners": [
    "alice.testnet",
    "bob.testnet",
    "charlie.testnet",
    "dave.testnet",
    "eve.testnet"
  ],
  "num_confirmations": 3
}' --accountId dao-treasury.testnet
```

### Use It

**Web UI:**
```bash
cd frontend
npm install
npm run dev
# Open http://localhost:3000
# Connect wallet, approve transactions visually
```

**Shell Scripts:**
```bash
cd ../../scripts
export NEAR_ACCOUNT=alice.testnet

# View pending
./multisig.sh view-pending dao-treasury.testnet

# Submit transfer
./multisig.sh submit dao-treasury.testnet recipient.testnet 10

# Approve (as different owners)
export NEAR_ACCOUNT=bob.testnet
./multisig.sh approve dao-treasury.testnet 0

export NEAR_ACCOUNT=charlie.testnet
./multisig.sh approve dao-treasury.testnet 0
# ✓ Executes after 3rd approval
```

**Direct NEAR CLI:**
```bash
# Alice submits (requires 0.01 NEAR storage deposit)
near call dao-treasury.testnet submit_transaction '{
  "receiver_id": "recipient.testnet",
  "actions": [{
    "Transfer": {"amount": "10000000000000000000000000"}
  }],
  "expiration_hours": null
}' --accountId alice.testnet --deposit 0.01

# Bob confirms
near call dao-treasury.testnet confirm_transaction '{
  "tx_id": 0
}' --accountId bob.testnet

# Charlie confirms (reaches threshold)
near call dao-treasury.testnet confirm_transaction '{
  "tx_id": 0
}' --accountId charlie.testnet

# Any owner can now execute
near call dao-treasury.testnet execute_transaction '{
  "tx_id": 0
}' --accountId alice.testnet --gas 100000000000000
```

---

## 2. Protocol Upgrade (Timelock)

**What:** Contract upgrades require 2 approvals + 48-hour delay.

**Use:** Protocol governance, high-value operations, security-critical actions.

### Quick Start

```bash
# CLI approach
near-multisig init protocol-upgrade --template timelock

# OR copy template with frontend
cp -r templates/timelock protocol-upgrade
cd protocol-upgrade/contract
./build.sh
```

### Deploy & Initialize

```bash
# Deploy
near deploy --accountId upgrade-multisig.testnet \
  --wasmFile release/protocol_upgrade.wasm

# Initialize: 3 owners, need 2 approvals, 48-hour delay
near call upgrade-multisig.testnet new '{
  "owners": ["dev.testnet", "security.testnet", "admin.testnet"],
  "num_confirmations": 2,
  "timelock_duration": 172800000000000
}' --accountId upgrade-multisig.testnet
```

**Common timelock durations:**
- 1 hour: `3600000000000`
- 24 hours: `86400000000000`
- 48 hours: `172800000000000`
- 1 week: `604800000000000`

### Use It

**Web UI:**
```bash
cd frontend
npm install
npm run dev
# Frontend shows:
# - Pending tab (needs approvals)
# - Scheduled tab (approved, counting down)
# - Ready to Execute tab (timelock expired)
```

**Workflow:**

1. **Submit** (dev.testnet)
   ```bash
   near call upgrade-multisig.testnet submit_transaction '{
     "receiver_id": "main-contract.testnet",
     "actions": [{"Transfer": {"amount": "1000000000000000000000000"}}],
     "expiration_hours": null
   }' --accountId dev.testnet --deposit 0.01
   ```

2. **Approve** (security.testnet)
   ```bash
   near call upgrade-multisig.testnet confirm_transaction '{
     "tx_id": 0
   }' --accountId security.testnet
   # ✓ Transaction SCHEDULED (not executed yet)
   # Must wait 48 hours
   ```

3. **Wait** for timelock to expire

4. **Execute** (anyone can execute)
   ```bash
   near call upgrade-multisig.testnet execute_transaction '{
     "tx_id": 0
   }' --accountId anyone.testnet
   # ✓ Now executes
   ```

**Shell scripts:**
```bash
cd scripts

# View scheduled transactions
./multisig.sh view-pending upgrade-multisig.testnet

# Execute after timelock
export NEAR_ACCOUNT=anyone.testnet
./multisig.sh approve upgrade-multisig.testnet 0
```

---

## 3. Token Governance (Weighted)

**What:** Voting power based on token holdings.

**Use:** DAOs with token voting, equity-based decisions, stakeholder governance.

### Quick Start

```bash
# CLI approach
near-multisig init token-gov --template weighted

# OR copy template with frontend
cp -r templates/weighted token-gov
cd token-gov/contract
./build.sh
```

### Deploy & Initialize

```bash
# Deploy
near deploy --accountId token-gov.testnet \
  --wasmFile release/token_gov.wasm

# Initialize: owners with weights, need 60% approval
near call token-gov.testnet new '{
  "owners_with_weights": [
    ["whale.testnet", 50],
    ["holder1.testnet", 25],
    ["holder2.testnet", 15],
    ["holder3.testnet", 10]
  ],
  "approval_threshold": 60
}' --accountId token-gov.testnet
```

**Weight distributions:**
- **Token-based:** Weight = number of tokens
- **Equity-based:** Weight = ownership percentage
- **Role-based:** Admin=100, Moderator=50, User=10

### Use It

**Web UI:**
```bash
cd frontend
npm install
npm run dev
# Frontend shows:
# - Weight-based progress bars (50/60 = 83%)
# - Each approver's weight
# - Approval threshold
```

**Scenario: Whale + Holder1 (75% > 60%)**
```bash
# Whale submits (auto-approves with 50 weight)
near call token-gov.testnet submit_transaction '{
  "receiver_id": "recipient.testnet",
  "actions": [{"Transfer": {"amount": "1000000000000000000000000"}}],
  "expiration_hours": null
}' --accountId whale.testnet --deposit 0.01

# Holder1 approves (adds 25 weight = 75 total)
near call token-gov.testnet approve_transaction '{
  "tx_id": 0
}' --accountId holder1.testnet
# ✓ 75 >= 60, EXECUTES immediately
```

**Scenario: Smaller holders (50% < 60%)**
```bash
# Holder2 submits (15 weight)
near call token-gov.testnet submit_transaction '{
  "receiver_id": "recipient.testnet",
  "actions": [{"Transfer": {"amount": "1000000000000000000000000"}}],
  "expiration_hours": null
}' --accountId holder2.testnet --deposit 0.01

# Holder3 approves (15 + 10 = 25)
near call token-gov.testnet approve_transaction '{
  "tx_id": 0
}' --accountId holder3.testnet

# Holder1 approves (25 + 25 = 50)
near call token-gov.testnet approve_transaction '{
  "tx_id": 0
}' --accountId holder1.testnet
# ✗ 50 < 60, NOT executed (need whale or more holders)
```

---

## Common Customizations

### Add Function Call Action

Instead of transfers, call another contract:

```bash
near call multisig.testnet submit_transaction '{
  "receiver_id": "some-contract.testnet",
  "actions": [{
    "FunctionCall": {
      "method_name": "do_something",
      "args": [1, 2, 3],
      "gas": 30000000000000,
      "deposit": "0"
    }
  }],
  "expiration_hours": null
}' --accountId owner.testnet --deposit 0.01
```

### View Pending Transactions

```bash
near view multisig.testnet get_pending_transactions
```

### Check Who Approved

```bash
near view multisig.testnet get_transaction '{"tx_id": 0}'
```

Output shows:
```json
{
  "receiver_id": "recipient.testnet",
  "actions": [...],
  "confirmations": ["alice.testnet", "bob.testnet"],
  "executed": false
}
```

---

## Using the Frontends

All three templates include Next.js frontends.

### Setup

```bash
cd templates/<type>/frontend
npm install
cp .env.example .env.local
npm run dev
```

### Features

**Basic Frontend:**
- View pending transactions
- Approve with one click
- Shows M/N progress

**Timelock Frontend:**
- 3 tabs: Pending / Scheduled / Ready to Execute
- Live countdown timers
- Execute button when ready

**Weighted Frontend:**
- Weight-based progress bars
- Shows each voter's weight
- Percentage complete display

### Network Configuration

Edit `.env.local`:

**Testnet:**
```env
NEXT_PUBLIC_NETWORK_ID=testnet
NEXT_PUBLIC_NODE_URL=https://rpc.testnet.near.org
```

**Mainnet:**
```env
NEXT_PUBLIC_NETWORK_ID=mainnet
NEXT_PUBLIC_NODE_URL=https://rpc.mainnet.near.org
```

---

## Troubleshooting

### "Not an owner" Error

```bash
# Check who the owners are
near view multisig.testnet get_owners

# Make sure you're signing with an owner account
# Wrong: --accountId random.testnet
# Right: --accountId alice.testnet (if alice is an owner)
```

### Transaction Not Executing

**Basic multisig:**
- Check approvals: need M confirmations
- Use: `near view multisig.testnet get_transaction '{"tx_id": 0}'`

**Timelock multisig:**
- Check if scheduled: `scheduled_time` should be set
- Check current time vs scheduled time
- Call `execute_transaction` after timelock expires

**Weighted multisig:**
- Check total weight vs threshold
- Use: `near view multisig.testnet get_transaction_progress '{"tx_id": 0}'`
- Returns: `[current_weight, threshold]`

### Frontend Won't Connect

1. Check `.env.local` has correct network
2. Try different wallet (My NEAR Wallet, Meteor, etc.)
3. Clear browser cache
4. Check browser console for errors

### Build Fails

```bash
# Clean and rebuild
cd contract
cargo clean
./build.sh

# Check cargo-near version
cargo near --version
# Should be 0.18.0 or newer
```

---

## Next Steps

**Learn more:**
- [INITIALIZATION.md](INITIALIZATION.md) - Complete deployment guide
- [README.md](README.md) - SDK overview
- [templates/README.md](templates/README.md) - Template details
- [scripts/README.md](scripts/README.md) - CLI scripts

**Customize:**
- Edit contract code in `src/lib.rs`
- Modify frontend in `frontend/components/`
- Add custom validation logic
- Implement additional features

**Deploy:**
1. Build contract
2. Deploy to testnet first
3. Test thoroughly
4. Deploy to mainnet when ready
