export const NEAR_CONFIG = {
  networkId: process.env.NEXT_PUBLIC_NETWORK_ID || 'testnet',
  nodeUrl: process.env.NEXT_PUBLIC_NODE_URL || 'https://rpc.testnet.near.org',
  walletUrl: process.env.NEXT_PUBLIC_WALLET_URL || 'https://testnet.mynearwallet.com',
  helperUrl: process.env.NEXT_PUBLIC_HELPER_URL || 'https://helper.testnet.near.org',
}

export function formatNearAmount(amount: string): string {
  const near = Number(BigInt(amount)) / 1e24
  return near.toFixed(2)
}

export function parseNearAmount(amount: string): string {
  const yocto = (parseFloat(amount) * 1e24).toLocaleString('fullwide', {
    useGrouping: false,
  })
  return yocto
}

export function shortenAddress(address: string): string {
  if (address.length < 20) return address
  return `${address.slice(0, 10)}...${address.slice(-8)}`
}
