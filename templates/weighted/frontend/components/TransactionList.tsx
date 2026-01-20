'use client'

import { useEffect, useState } from 'react'
import { useWallet } from './providers/WalletProvider'
import { WeightedMultisigContract, Transaction } from '@/utils/contract'
import { TransactionCard } from './TransactionCard'

interface TransactionListProps {
  multisigAddress: string
}

export function TransactionList({ multisigAddress }: TransactionListProps) {
  const { account } = useWallet()
  const [transactions, setTransactions] = useState<Transaction[]>([])
  const [threshold, setThreshold] = useState<number>(0)
  const [totalWeight, setTotalWeight] = useState<number>(0)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    loadTransactions()
    const interval = setInterval(loadTransactions, 10000)
    return () => clearInterval(interval)
  }, [account, multisigAddress])

  async function loadTransactions() {
    if (!account) {
      setLoading(false)
      return
    }

    try {
      setLoading(true)
      setError(null)

      const contract = new WeightedMultisigContract(account, multisigAddress)

      const [txs, approvalThreshold, total] = await Promise.all([
        contract.getPendingTransactions(),
        contract.getApprovalThreshold(),
        contract.getTotalWeight(),
      ])

      setTransactions(txs)
      setThreshold(approvalThreshold)
      setTotalWeight(total)
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
    <div className="bg-white rounded-lg shadow p-6">
      <div className="mb-6">
        <h2 className="text-xl font-semibold mb-4">Pending Transactions</h2>
        <div className="flex items-center gap-6 text-sm text-gray-600">
          <div>
            <span className="font-medium">Approval Threshold:</span> {threshold}
          </div>
          <div>
            <span className="font-medium">Total Weight:</span> {totalWeight}
          </div>
        </div>
      </div>

      {loading ? (
        <p className="text-gray-500">Loading...</p>
      ) : error ? (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <p className="text-red-800">{error}</p>
        </div>
      ) : transactions.length === 0 ? (
        <p className="text-gray-500">No pending transactions</p>
      ) : (
        <div className="space-y-4">
          {transactions.map((tx, idx) => (
            <TransactionCard
              key={idx}
              transaction={tx}
              txId={idx}
              threshold={threshold}
              onApprove={loadTransactions}
              currentUser={account.accountId}
            />
          ))}
        </div>
      )}
    </div>
  )
}
