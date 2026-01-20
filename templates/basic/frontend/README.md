# Basic Multisig Frontend

Web interface for the Basic Multisig contract. Connect your NEAR wallet, view pending transactions, and approve them with one click.

## Features

- **Wallet Connection** - Connect via My NEAR Wallet
- **View Pending Transactions** - See all transactions awaiting approval
- **One-Click Approve** - Approve transactions directly from the UI
- **Real-time Updates** - Transaction list refreshes after approvals

## Setup

### Install Dependencies

```bash
npm install
```

### Configure Environment

Copy `.env.example` to `.env.local`:

```bash
cp .env.example .env.local
```

Edit `.env.local` if you need custom RPC endpoints or want to use mainnet.

### Run Development Server

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser.

## Usage

1. **Connect Wallet** - Click "Connect Wallet" and select My NEAR Wallet
2. **Enter Multisig Address** - Enter the deployed multisig contract address (e.g., `multisig.testnet`)
3. **View Transactions** - See all pending transactions
4. **Approve** - Click "Approve Transaction" on any pending transaction you want to confirm

## How It Works

### Wallet Connection

Uses `@near-wallet-selector` for seamless wallet integration:
- Connects to My NEAR Wallet
- Stores session in browser local storage
- Auto-reconnects on page reload

### Transaction Viewing

Calls contract view methods:
```typescript
contract.get_pending_transactions() // Get all pending
contract.get_num_confirmations()    // Get threshold
```

### Approving Transactions

Calls contract write method:
```typescript
contract.confirm_transaction(tx_id)
```

Your wallet will pop up to sign the transaction. After confirmation, the UI refreshes to show updated status.

## Components

- **WalletProvider** - React context for wallet state
- **WalletConnect** - Connect/disconnect button
- **TransactionList** - Fetches and displays pending transactions
- **TransactionCard** - Individual transaction with approve button

## Customization

### Styling

Uses Tailwind CSS. Customize `tailwind.config.js` for theme changes.

### Network

Change network in `.env.local`:
- `NEXT_PUBLIC_NETWORK_ID=testnet` or `mainnet`

### Supported Wallets

Currently supports My NEAR Wallet. To add more wallets, edit `WalletProvider.tsx`:

```typescript
import { setupMyNearWallet } from '@near-wallet-selector/my-near-wallet'
import { setupSender } from '@near-wallet-selector/sender'
import { setupHereWallet } from '@near-wallet-selector/here-wallet'

const _selector = await setupWalletSelector({
  network: NEAR_CONFIG.networkId as any,
  modules: [
    setupMyNearWallet(),
    setupSender(),
    setupHereWallet(),
  ],
})
```

## Build for Production

```bash
npm run build
npm start
```

## Deploy

Can be deployed to Vercel, Netlify, or any static host:

```bash
npm run build
# Upload 'out' directory or '.next' folder
```

## Troubleshooting

**"Failed to load transactions"**
- Check multisig address is correct
- Verify contract is deployed
- Ensure you're on the right network (testnet vs mainnet)

**Wallet not connecting**
- Clear browser local storage
- Try in incognito mode
- Check wallet extension is installed

**Approve not working**
- Ensure you're an owner of the multisig
- Check you haven't already approved
- Verify transaction hasn't been executed

## Related Files

- `../contract/` - The multisig smart contract
- `../../scripts/` - CLI scripts for technical users
