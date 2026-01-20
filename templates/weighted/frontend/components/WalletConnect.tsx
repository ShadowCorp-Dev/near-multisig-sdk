'use client'

import { useWallet } from './providers/WalletProvider'

export function WalletConnect() {
  const { accountId, signIn, signOut, isConnected } = useWallet()

  if (isConnected && accountId) {
    return (
      <div className="flex items-center gap-4">
        <div className="px-4 py-2 bg-green-100 text-green-800 rounded-lg">
          Connected: {accountId}
        </div>
        <button
          onClick={signOut}
          className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
        >
          Disconnect
        </button>
      </div>
    )
  }

  return (
    <button
      onClick={signIn}
      className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
    >
      Connect Wallet
    </button>
  )
}
