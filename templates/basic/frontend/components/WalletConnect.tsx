'use client'

import { useWallet } from './providers/WalletProvider'
import { shortenAddress } from '@/utils/near'

export default function WalletConnect() {
  const { accountId, signIn, signOut, isConnected } = useWallet()

  if (isConnected && accountId) {
    return (
      <div className="flex items-center gap-4">
        <span className="text-sm text-gray-600 dark:text-gray-300">
          {shortenAddress(accountId)}
        </span>
        <button
          onClick={signOut}
          className="px-4 py-2 bg-red-500 text-white rounded-lg hover:bg-red-600 transition-colors"
        >
          Disconnect
        </button>
      </div>
    )
  }

  return (
    <button
      onClick={signIn}
      className="px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors font-medium"
    >
      Connect Wallet
    </button>
  )
}
