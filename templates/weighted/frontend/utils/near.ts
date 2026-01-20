import { parseNearAmount, formatNearAmount as formatAmount } from 'near-api-js/lib/utils/format'

export function formatNearAmount(amount: string): string {
  return formatAmount(amount, 2)
}

export { parseNearAmount }

export const NEAR_CONFIG = {
  networkId: process.env.NEXT_PUBLIC_NETWORK_ID || 'testnet',
  nodeUrl: process.env.NEXT_PUBLIC_NODE_URL || 'https://rpc.testnet.near.org',
  walletUrl: process.env.NEXT_PUBLIC_WALLET_URL || 'https://testnet.mynearwallet.com/',
  helperUrl: process.env.NEXT_PUBLIC_HELPER_URL || 'https://helper.testnet.near.org',
}
