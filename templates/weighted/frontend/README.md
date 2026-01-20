# Weighted Multisig Frontend

Next.js web interface for managing weighted multisig wallets on NEAR.

## Features

- **Wallet Integration**: Connect with My NEAR Wallet
- **Weight-Based Voting**: Visual display of approval weights
- **Progress Tracking**: Percentage-based progress bars
- **Approval List**: Shows who approved and their voting weight
- **Transaction Management**: Approve transactions with weighted voting
- **Auto-execution**: Transaction executes when weight threshold reached

## Quick Start

### 1. Install Dependencies

```bash
npm install
```

### 2. Configure Environment

```bash
cp .env.example .env.local
```

For testnet (default):
```env
NEXT_PUBLIC_NETWORK_ID=testnet
NEXT_PUBLIC_NODE_URL=https://rpc.testnet.near.org
NEXT_PUBLIC_WALLET_URL=https://testnet.mynearwallet.com/
NEXT_PUBLIC_HELPER_URL=https://helper.testnet.near.org
```

For mainnet:
```env
NEXT_PUBLIC_NETWORK_ID=mainnet
NEXT_PUBLIC_NODE_URL=https://rpc.mainnet.near.org
NEXT_PUBLIC_WALLET_URL=https://app.mynearwallet.com/
NEXT_PUBLIC_HELPER_URL=https://helper.mainnet.near.org
```

### 3. Run Development Server

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000)

## Usage

### Connect Wallet

1. Click "Connect Wallet"
2. Select wallet (My NEAR Wallet)
3. Approve connection

### Load Multisig

1. Enter multisig contract address (e.g., `weighted-multisig.testnet`)
2. Click "Load"
3. View approval threshold and total weight

### View Transactions

All pending transactions are displayed with:
- **Transaction details** (receiver, actions)
- **Weight progress bar** (current weight / threshold)
- **Percentage complete** (visual indicator)
- **Approval list** (who approved + their weight)
- **Approve button** (if you haven't approved)

### Approve Transaction

1. Review transaction details
2. Check current weight vs threshold
3. Click "Approve"
4. Sign transaction in wallet
5. Your weight is added to total

When `total_weight >= threshold`, **transaction auto-executes**.

## Example Workflow

**Setup:** Alice=40, Bob=30, Charlie=20, threshold=60

1. **Alice submits** transfer (40 weight automatically added)
2. **Charlie approves** (+20 weight = 60 total)
3. **Transaction executes** (60 >= 60 threshold)

Alternative:

1. **Charlie submits** transfer (20 weight)
2. **Bob approves** (+30 weight = 50 total)
3. **Alice approves** (+40 weight = 90 total)
4. **Transaction executes** (90 >= 60 threshold)

## Components

### WalletProvider
Context provider for NEAR wallet state. Handles:
- Wallet selector setup
- Account connection
- Sign in/out
- Account persistence

### WalletConnect
Connect/disconnect button with account display.

### TransactionList
Fetches and displays pending transactions:
- Calls `get_pending_transactions()`
- Shows approval threshold and total weight
- Auto-refreshes every 10 seconds
- Displays loading/error states

### TransactionCard
Individual transaction with weight-based UI:
- Transaction details (receiver, actions)
- Weight progress bar with percentage
- Approval list showing each approver's weight
- Approve button (disabled if already approved)
- Auto-updates when new approvals added

## Utilities

### contract.ts
`WeightedMultisigContract` class wrapping NEAR contract calls:

**View methods:**
- `getPendingTransactions()`
- `getTransaction(txId)`
- `getOwners()` - Returns `[account_id, weight][]`
- `getOwnerWeight(accountId)` - Get specific owner's weight
- `getApprovalThreshold()` - Weight needed for execution
- `getTotalWeight()` - Sum of all owner weights
- `getTransactionProgress(txId)` - Returns `[current_weight, threshold]`

**Call methods:**
- `approveTransaction(txId)`

### near.ts
Helper functions:
- `formatNearAmount()` - Format yoctoNEAR to NEAR
- `parseNearAmount()` - Parse NEAR to yoctoNEAR
- `NEAR_CONFIG` - Network configuration

## Customization

### Display Owner Weights

Add owners list component:

```typescript
'use client'

import { useEffect, useState } from 'react'
import { WeightedMultisigContract } from '@/utils/contract'

export function OwnersList({ multisigAddress }: { multisigAddress: string }) {
  const [owners, setOwners] = useState<[string, number][]>([])

  useEffect(() => {
    async function load() {
      const contract = new WeightedMultisigContract(account, multisigAddress)
      const ownerData = await contract.getOwners()
      setOwners(ownerData)
    }
    load()
  }, [multisigAddress])

  return (
    <div>
      <h3>Owners and Weights</h3>
      {owners.map(([account, weight]) => (
        <div key={account}>
          {account}: {weight}
        </div>
      ))}
    </div>
  )
}
```

### Add Weight Visualization

Use pie chart or bar chart to show weight distribution:

```typescript
const totalWeight = owners.reduce((sum, [_, w]) => sum + w, 0)

{owners.map(([account, weight]) => (
  <div key={account} className="mb-2">
    <div className="flex justify-between mb-1">
      <span>{account}</span>
      <span>{((weight / totalWeight) * 100).toFixed(1)}%</span>
    </div>
    <div className="bg-gray-200 rounded h-2">
      <div
        className="bg-blue-600 h-2 rounded"
        style={{ width: `${(weight / totalWeight) * 100}%` }}
      />
    </div>
  </div>
))}
```

### Change Wallet Providers

Edit `components/providers/WalletProvider.tsx`:

```typescript
import { setupMyNearWallet } from '@near-wallet-selector/my-near-wallet'
import { setupMeteorWallet } from '@near-wallet-selector/meteor-wallet'

const selector = await setupWalletSelector({
  network: NEAR_CONFIG.networkId as any,
  modules: [
    setupMyNearWallet(),
    setupMeteorWallet(),
  ],
})
```

### Styling

Uses Tailwind CSS. Edit `tailwind.config.js`:

```javascript
module.exports = {
  theme: {
    extend: {
      colors: {
        primary: '#your-color',
      },
    },
  },
}
```

## Deployment

### Build for Production

```bash
npm run build
```

### Deploy to Vercel

```bash
npm install -g vercel
vercel
```

Or connect GitHub repo to Vercel dashboard.

### Deploy to Netlify

```bash
npm install -g netlify-cli
netlify deploy --prod
```

## Troubleshooting

**"Failed to load transactions"**
- Verify multisig contract address
- Check contract is deployed and initialized
- Ensure correct network (testnet/mainnet)

**Wallet won't connect**
- Check `.env.local` has correct `NEXT_PUBLIC_WALLET_URL`
- Try different browser
- Clear cache and reconnect

**Approval doesn't add weight**
- Ensure connected account is an owner
- Check account has enough NEAR for gas
- Verify you haven't already approved

**Transaction doesn't execute**
- Check if threshold is reached (current weight >= threshold)
- Look at progress bar percentage
- Review approval list to see current total weight

**Progress bar shows >100%**
- This is normal if approvals exceed threshold
- Transaction will have already executed

## Differences from Other Templates

| Feature | Basic | Weighted | Timelock |
|---------|-------|----------|----------|
| **Voting** | Equal | **Weighted** | Equal |
| **Progress** | Count | **Weight** | Count |
| **Display** | Confirmations | **Weights** | Confirmations |
| **Threshold** | Count | **Weight sum** | Count |
| **Execution** | Immediate | **Immediate** | Delayed |

## Key Differences in UI

### Basic Multisig
- Shows "2 / 3 confirmations"
- Progress bar based on count
- Simple approval list

### Weighted Multisig
- Shows "60 / 100 weight (60%)"
- Progress bar based on weight percentage
- Approval list with individual weights
- Displays total weight and threshold

### Timelock Multisig
- Shows countdown timer
- Scheduled vs executable states
- Execute button (separate from approve)

## Development

```bash
# Install dependencies
npm install

# Run dev server
npm run dev

# Build
npm run build

# Start production server
npm start

# Lint
npm run lint
```

## Tech Stack

- **Framework**: Next.js 14 (App Router)
- **Styling**: Tailwind CSS
- **NEAR Integration**:
  - @near-wallet-selector
  - near-api-js
- **Language**: TypeScript

## Example Use Cases

### DAO Treasury
- Core team: 100 weight each
- Community members: 50 weight each
- Threshold: 200 (needs 2 core OR 4 community)

### Token Governance
- Weight = token holdings
- Threshold = 50% of total supply
- Proportional voting power

### Multi-tier Access
- Admins: 100 weight
- Moderators: 50 weight
- Contributors: 25 weight
- Threshold: 150 (flexible combinations)

## License

MIT
