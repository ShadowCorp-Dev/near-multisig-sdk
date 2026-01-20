'use client'

import { useState } from 'react'
import { WalletConnect } from '@/components/WalletConnect'
import { TransactionList } from '@/components/TransactionList'

export default function Home() {
  const [multisigAddress, setMultisigAddress] = useState('')
  const [submittedAddress, setSubmittedAddress] = useState('')

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    setSubmittedAddress(multisigAddress)
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-4xl mx-auto p-6">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">Weighted Multisig</h1>
          <p className="text-gray-600">Manage multisig transactions with weighted voting</p>
        </div>

        <div className="mb-6">
          <WalletConnect />
        </div>

        <div className="bg-white rounded-lg shadow p-6 mb-6">
          <h2 className="text-lg font-semibold mb-4">Load Multisig Contract</h2>
          <form onSubmit={handleSubmit} className="flex gap-2">
            <input
              type="text"
              value={multisigAddress}
              onChange={(e) => setMultisigAddress(e.target.value)}
              placeholder="multisig.testnet"
              className="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <button
              type="submit"
              className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            >
              Load
            </button>
          </form>
        </div>

        {submittedAddress && (
          <TransactionList multisigAddress={submittedAddress} />
        )}
      </div>
    </div>
  )
}
