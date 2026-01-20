# Multisig Contract Templates

Complete, production-ready multisig implementations with frontends and documentation.

## Overview

Each template includes:
- **Contract** - Fully implemented Rust smart contract
- **Frontend** - Next.js web UI with wallet integration
- **Documentation** - Deployment and usage guides

## Available Templates

### Basic Multisig

**Location:** `basic/`

Simple M-of-N approval system. Transaction must be explicitly executed after M owners confirm.

**Use cases:**
- DAO treasuries
- Shared custody
- Team wallets

**Features:**
- Submit transactions (transfers, function calls) with storage deposit
- Confirm transactions
- Manual execution after threshold reached
- Revocation support
- Owner management (add/remove owners, change threshold)
- O(1) transaction lookups for gas efficiency

**Quick start:**
```bash
cd basic/contract && ./build.sh
cd ../frontend && npm install && npm run dev
```

### Timelock Multisig

Adds mandatory delay between approval and execution.

**Use cases:**
- Protocol upgrades
- High-value transactions
- Security-critical operations

### Weighted Multisig

Different voting weights per owner.

**Use cases:**
- Token holder governance
- Stakeholder voting
- Proportional control

## Usage Modes

### 1. Web UI (Recommended)

User-friendly interface for non-technical users.

```bash
cd basic/frontend
npm install
npm run dev
```

Open http://localhost:3000, connect wallet, and manage transactions.

### 2. CLI Scripts

Shell scripts for technical users.

```bash
cd ../scripts
./multisig.sh view-pending multisig.testnet
./multisig.sh approve multisig.testnet 0
```

### 3. Direct NEAR CLI

Full control via command line.

```bash
near view multisig.testnet get_pending_transactions
near call multisig.testnet confirm_transaction '{"tx_id": 0}' --accountId you.near
```

## Getting Started

### 1. Choose a Template

Pick the template that matches your use case:
- **Basic** - For most DAO/team wallets
- **Timelock** - When you need safety delays
- **Weighted** - For token-based governance

### 2. Build the Contract

```bash
cd basic/contract
./build.sh
```

### 3. Deploy

```bash
near deploy --accountId your-multisig.near --wasmFile target/near/basic_multisig.wasm
```

### 4. Initialize

```bash
near call your-multisig.near new '{
  "owners": ["alice.near", "bob.near", "charlie.near"],
  "num_confirmations": 2
}' --accountId your-multisig.near
```

### 5. Use the UI

```bash
cd ../frontend
npm install
cp .env.example .env.local
npm run dev
```

## Customization

### Modify Contract

Edit `contract/src/lib.rs` to:
- Add custom validation logic
- Implement additional features
- Change execution behavior

After changes, rebuild:
```bash
cd contract && ./build.sh
```

### Customize Frontend

Edit frontend components in `frontend/components/`:
- **WalletProvider.tsx** - Add more wallet options
- **TransactionCard.tsx** - Change UI layout
- **page.tsx** - Add new features

Styling via Tailwind:
```bash
# Edit tailwind.config.js for theme changes
cd frontend && npm run dev
```

### Network Configuration

For mainnet deployment:

**Contract** - Deploy to mainnet account
**Frontend** - Update `.env.local`:
```
NEXT_PUBLIC_NETWORK_ID=mainnet
NEXT_PUBLIC_NODE_URL=https://rpc.mainnet.near.org
```

## Comparison

| Feature | Basic | Timelock | Weighted |
|---------|-------|----------|----------|
| **Approval Model** | M-of-N | M-of-N + delay | Weight threshold |
| **Execution** | Immediate | After timelock | Immediate |
| **Complexity** | Simple | Medium | Medium |
| **Use Case** | General | Security | Governance |

## Documentation

Each template has comprehensive documentation:

- `contract/README.md` - Contract deployment and methods
- `frontend/README.md` - Frontend setup and customization
- `../scripts/README.md` - CLI usage

## Support

- Main docs: `../README.md`
- Initialization guide: `../INITIALIZATION.md`
- Examples: `../EXAMPLES.md`

## License

MIT
