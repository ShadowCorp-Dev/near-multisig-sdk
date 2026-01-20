# Timelock Multisig Frontend

Next.js web interface for managing timelock multisig wallets on NEAR.

## Features

- **Wallet Integration**: Connect with My NEAR Wallet
- **Transaction Views**:
  - Pending approval transactions
  - Scheduled transactions (approved but timelock not expired)
  - Executable transactions (timelock expired, ready to execute)
- **Live Countdown**: Real-time display of time remaining until execution
- **Approve Transactions**: Confirm pending transactions
- **Execute Transactions**: Execute transactions after timelock expires
- **Transaction Details**: View receiver, actions, confirmations, and schedule

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

1. Enter multisig contract address (e.g., `timelock-multisig.testnet`)
2. Click "Load"

### View Transactions

**Tabs:**
- **Pending Approval**: Transactions needing confirmations
- **Scheduled**: Transactions with enough approvals, waiting for timelock
- **Ready to Execute**: Transactions past timelock, ready to execute

### Approve Transaction

1. Go to "Pending Approval" tab
2. Review transaction details
3. Click "Approve"
4. Sign transaction in wallet

When threshold is reached, transaction moves to "Scheduled" tab.

### Execute Transaction

1. Go to "Ready to Execute" tab
2. Click "Execute Transaction"
3. Sign transaction in wallet

**Note:** Anyone can execute after timelock expires (not just owners).

## Workflow Example

**Setup:** 3-of-5 multisig with 24-hour timelock

1. **Alice submits** transfer of 100 NEAR to `recipient.near`
2. **Bob approves** → 2 confirmations (needs 3)
3. **Charlie approves** → **Transaction scheduled** for 24 hours from now
4. **Wait 24 hours** → Transaction shows in "Ready to Execute" tab
5. **Anyone executes** → Transfer completes

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
Fetches and displays transactions with tab navigation:
- Calls `get_pending_transactions()`, `get_scheduled_transactions()`, or `get_executable_transactions()`
- Auto-refreshes every 10 seconds
- Shows loading/error states

### TransactionCard
Individual transaction display with:
- Transaction details (receiver, actions)
- Approval progress bar
- Countdown timer (for scheduled transactions)
- Approve button (pending transactions)
- Execute button (executable transactions)
- Real-time status updates

## Utilities

### contract.ts
`TimelockMultisigContract` class wrapping NEAR contract calls:

**View methods:**
- `getPendingTransactions()`
- `getScheduledTransactions()`
- `getExecutableTransactions()`
- `getTransaction(txId)`
- `getOwners()`
- `getNumConfirmations()`
- `getTimelockDuration()`

**Call methods:**
- `confirmTransaction(txId)`
- `executeTransaction(txId)`

### near.ts
Helper functions:
- `formatNearAmount()` - Format yoctoNEAR to NEAR
- `parseNearAmount()` - Parse NEAR to yoctoNEAR
- `formatTimestamp()` - Convert nanoseconds to readable date
- `getTimeRemaining()` - Calculate time until execution
- `NEAR_CONFIG` - Network configuration

## Customization

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

Available wallets: [NEAR Wallet Selector](https://github.com/near/wallet-selector)

### Change Styling

Uses Tailwind CSS. Edit `tailwind.config.js` for theme:

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

### Add Submit Transaction Form

Create `components/SubmitTransaction.tsx`:

```typescript
async function handleSubmit() {
  const contract = new TimelockMultisigContract(account, multisigAddress)

  // Submit via contract call (not view method)
  await account.functionCall({
    contractId: multisigAddress,
    methodName: 'submit_transaction',
    args: {
      receiver_id: receiverId,
      actions: [{
        Transfer: { amount: parseNearAmount(amount) }
      }]
    },
    gas: '30000000000000',
  })
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

**Environment variables:** Set in deployment platform's dashboard.

## Troubleshooting

**"Failed to load transactions"**
- Verify multisig contract address is correct
- Check contract is deployed and initialized
- Ensure using correct network (testnet/mainnet)

**Wallet won't connect**
- Check `.env.local` has correct `NEXT_PUBLIC_WALLET_URL`
- Try different browser (some wallets have compatibility issues)
- Clear browser cache and reconnect

**Transaction approval fails**
- Ensure connected account is an owner
- Check account has enough NEAR for gas
- View transaction in NEAR Explorer for error details

**Execute button doesn't appear**
- Verify timelock has expired (check countdown)
- Refresh page to reload transaction state
- Check transaction is in "Ready to Execute" tab

**Countdown shows negative time**
- Click execute button (timelock has passed)
- This happens if page was open when timelock expired

## Differences from Basic Multisig Frontend

| Feature | Basic | Timelock |
|---------|-------|----------|
| **Tabs** | Single pending view | 3 tabs (pending/scheduled/executable) |
| **Execution** | Auto on approval | Manual after timelock |
| **UI Elements** | Approve button | Approve + Execute buttons |
| **Time Display** | None | Countdown timer |
| **Status** | Pending/Complete | Pending/Scheduled/Ready |

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

## License

MIT
