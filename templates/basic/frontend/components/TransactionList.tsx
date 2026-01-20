'use client'

import { useEffect, useState } from 'react'
import { useWallet } from './providers/WalletProvider'
import { MultisigContract, Transaction } from '@/utils/contract'
import TransactionCard from './TransactionCard'

interface TransactionListProps {
  multisigAddress: string
}

export default function TransactionList({ multisigAddress }: TransactionListProps) {
  const { account, accountId } = useWallet()
  const [transactions, setTransactions] = useState<Transaction[]>([])
  const [threshold, setThreshold] = useState<number>(0)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!account || !multisigAddress) return

    const loadTransactions = async () => {
      try {
        setLoading(true)
        setError(null)

        const contract = new MultisigContract(account, multisigAddress)
        const [pendingTxs, numConfirmations] = await Promise.all([
          contract.getPendingTransactions(),
          contract.getNumConfirmations(),
        ])

        setTransactions(pendingTxs)
        setThreshold(numConfirmations)
      } catch (err) {
        console.error('Error loading transactions:', err)
        setError('Failed to load transactions. Check the multisig address.')
      } finally {
        setLoading(false)
      }
    }

    loadTransactions()
  }, [account, multisigAddress])

  const handleApprove = async (txId: number) => {
    if (!account) return

    try {
      const contract = new MultisigContract(account, multisigAddress)
      await contract.confirmTransaction(txId)

      // Reload transactions after approval
      const pendingTxs = await contract.getPendingTransactions()
      setTransactions(pendingTxs)
    } catch (err) {
      console.error('Error approving transaction:', err)
      alert('Failed to approve transaction')
    }
  }

  if (loading) {
    return (
      <div className="text-center py-8">
        <div className="inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-current border-r-transparent"></div>
        <p className="mt-4 text-gray-600 dark:text-gray-400">Loading transactions...</p>
      </div>
    )
  }

  if (error) {
    return (
      <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
        <p className="text-red-800 dark:text-red-200">{error}</p>
      </div>
    )
  }

  if (transactions.length === 0) {
    return (
      <div className="text-center py-12 bg-gray-50 dark:bg-gray-800 rounded-lg">
        <p className="text-gray-600 dark:text-gray-400">No pending transactions</p>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      {transactions.map((tx) => (
        <TransactionCard
          key={tx.id}
          transaction={tx}
          threshold={threshold}
          onApprove={handleApprove}
          currentUser={accountId}
        />
      ))}
    </div>
  )
}
