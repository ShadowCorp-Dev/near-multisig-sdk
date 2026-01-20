'use client'

import { useState } from 'react'
import WalletConnect from '@/components/WalletConnect'
import TransactionList from '@/components/TransactionList'
import { useWallet } from '@/components/providers/WalletProvider'

export default function Home() {
  const { isConnected } = useWallet()
  const [multisigAddress, setMultisigAddress] = useState('')
  const [submittedAddress, setSubmittedAddress] = useState('')

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    setSubmittedAddress(multisigAddress)
  }

  return (
    <main className="min-h-screen p-8">
      <div className="max-w-4xl mx-auto">
        <div className="flex justify-between items-center mb-8">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
            Basic Multisig
          </h1>
          <WalletConnect />
        </div>

        {!isConnected ? (
          <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-6 text-center">
            <p className="text-blue-900 dark:text-blue-200 mb-4">
              Connect your NEAR wallet to view and approve transactions
            </p>
          </div>
        ) : (
          <>
            <div className="bg-white dark:bg-gray-800 rounded-lg p-6 mb-8 border border-gray-200 dark:border-gray-700">
              <h2 className="text-xl font-semibold mb-4 text-gray-900 dark:text-white">
                Multisig Contract Address
              </h2>
              <form onSubmit={handleSubmit} className="flex gap-4">
                <input
                  type="text"
                  value={multisigAddress}
                  onChange={(e) => setMultisigAddress(e.target.value)}
                  placeholder="multisig.testnet"
                  className="flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500"
                />
                <button
                  type="submit"
                  className="px-6 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors font-medium"
                >
                  Load
                </button>
              </form>
            </div>

            {submittedAddress && (
              <div>
                <h2 className="text-2xl font-semibold mb-4 text-gray-900 dark:text-white">
                  Pending Transactions
                </h2>
                <TransactionList multisigAddress={submittedAddress} />
              </div>
            )}
          </>
        )}
      </div>
    </main>
  )
}
