import { parseNearAmount, formatNearAmount as formatAmount } from 'near-api-js/lib/utils/format'

export function formatNearAmount(amount: string): string {
  return formatAmount(amount, 2)
}

export { parseNearAmount }

export function formatTimestamp(timestamp: number): string {
  return new Date(timestamp / 1_000_000).toLocaleString()
}

export function getTimeRemaining(timestamp: number): string {
  const now = Date.now() * 1_000_000
  const remaining = timestamp - now

  if (remaining <= 0) {
    return 'Ready to execute'
  }

  const seconds = Math.floor(remaining / 1_000_000_000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)

  if (days > 0) {
    return `${days}d ${hours % 24}h remaining`
  } else if (hours > 0) {
    return `${hours}h ${minutes % 60}m remaining`
  } else if (minutes > 0) {
    return `${minutes}m ${seconds % 60}s remaining`
  } else {
    return `${seconds}s remaining`
  }
}

export const NEAR_CONFIG = {
  networkId: process.env.NEXT_PUBLIC_NETWORK_ID || 'testnet',
  nodeUrl: process.env.NEXT_PUBLIC_NODE_URL || 'https://rpc.testnet.near.org',
  walletUrl: process.env.NEXT_PUBLIC_WALLET_URL || 'https://testnet.mynearwallet.com/',
  helperUrl: process.env.NEXT_PUBLIC_HELPER_URL || 'https://helper.testnet.near.org',
}
