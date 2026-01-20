'use client'

import { useEffect, useState } from 'react'
import { useWallet } from './providers/WalletProvider'
import { TimelockMultisigContract, Transaction } from '@/utils/contract'
import { TransactionCard } from './TransactionCard'

interface TransactionListProps {
  multisigAddress: string
}

type TabType = 'pending' | 'scheduled' | 'executable'

export function TransactionList({ multisigAddress }: TransactionListProps) {
  const { account } = useWallet()
  const [transactions, setTransactions] = useState<Transaction[]>([])
  const [threshold, setThreshold] = useState<number>(0)
  const [timelockDuration, setTimelockDuration] = useState<number>(0)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<TabType>('pending')

  useEffect(() => {
    loadTransactions()
    const interval = setInterval(loadTransactions, 10000)
    return () => clearInterval(interval)
  }, [account, multisigAddress, activeTab])

  async function loadTransactions() {
    if (!account) {
      setLoading(false)
      return
    }

    try {
      setLoading(true)
      setError(null)

      const contract = new TimelockMultisigContract(account, multisigAddress)

      const [txs, confirmations, duration] = await Promise.all([
        activeTab === 'pending'
          ? contract.getPendingTransactions()
          : activeTab === 'scheduled'
          ? contract.getScheduledTransactions()
          : contract.getExecutableTransactions(),
        contract.getNumConfirmations(),
        contract.getTimelockDuration(),
      ])

      setTransactions(txs)
      setThreshold(confirmations)
      setTimelockDuration(duration)
    } catch (err: any) {
      setError(err.message || 'Failed to load transactions')
    } finally {
      setLoading(false)
    }
  }

  if (!account) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <p className="text-gray-500">Connect wallet to view transactions</p>
      </div>
    )
  }

  return (
    <div className="bg-white rounded-lg shadow">
      <div className="border-b border-gray-200">
        <div className="flex gap-4 px-6">
          <button
            onClick={() => setActiveTab('pending')}
            className={`py-4 px-2 border-b-2 transition-colors ${
              activeTab === 'pending'
                ? 'border-blue-600 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700'
            }`}
          >
            Pending Approval
          </button>
          <button
            onClick={() => setActiveTab('scheduled')}
            className={`py-4 px-2 border-b-2 transition-colors ${
              activeTab === 'scheduled'
                ? 'border-blue-600 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700'
            }`}
          >
            Scheduled
          </button>
          <button
            onClick={() => setActiveTab('executable')}
            className={`py-4 px-2 border-b-2 transition-colors ${
              activeTab === 'executable'
                ? 'border-blue-600 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700'
            }`}
          >
            Ready to Execute
          </button>
        </div>
      </div>

      <div className="p-6">
        {loading ? (
          <p className="text-gray-500">Loading...</p>
        ) : error ? (
          <div className="bg-red-50 border border-red-200 rounded-lg p-4">
            <p className="text-red-800">{error}</p>
          </div>
        ) : transactions.length === 0 ? (
          <p className="text-gray-500">
            {activeTab === 'pending' && 'No pending transactions'}
            {activeTab === 'scheduled' && 'No scheduled transactions'}
            {activeTab === 'executable' && 'No transactions ready to execute'}
          </p>
        ) : (
          <div className="space-y-4">
            {transactions.map((tx, idx) => (
              <TransactionCard
                key={idx}
                transaction={tx}
                txId={idx}
                threshold={threshold}
                timelockDuration={timelockDuration}
                onApprove={loadTransactions}
                onExecute={loadTransactions}
                currentUser={account.accountId}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
