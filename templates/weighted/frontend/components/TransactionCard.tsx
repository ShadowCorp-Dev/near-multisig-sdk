'use client'

import { useState } from 'react'
import { useWallet } from './providers/WalletProvider'
import { WeightedMultisigContract, Transaction } from '@/utils/contract'
import { formatNearAmount } from '@/utils/near'

interface TransactionCardProps {
  transaction: Transaction
  txId: number
  threshold: number
  onApprove: () => void
  currentUser: string | null
}

export function TransactionCard({
  transaction,
  txId,
  threshold,
  onApprove,
  currentUser,
}: TransactionCardProps) {
  const { account } = useWallet()
  const [approving, setApproving] = useState(false)

  const hasApproved =
    currentUser && transaction.approvals.some(([account]) => account === currentUser)
  const currentWeight = transaction.total_weight
  const progressPercent = Math.min((currentWeight / threshold) * 100, 100)

  async function handleApprove() {
    if (!account) return

    try {
      setApproving(true)
      const contract = new WeightedMultisigContract(
        account,
        account.connection.networkId === 'testnet'
          ? 'multisig.testnet'
          : 'multisig.near'
      )
      await contract.approveTransaction(txId)
      onApprove()
    } catch (err: any) {
      alert(`Failed to approve: ${err.message}`)
    } finally {
      setApproving(false)
    }
  }

  function getActionDescription(action: any): string {
    if (action.Transfer) {
      return `Transfer ${formatNearAmount(action.Transfer.amount)} NEAR`
    }
    if (action.FunctionCall) {
      return `Call ${action.FunctionCall.method}()`
    }
    return 'Unknown action'
  }

  return (
    <div className="border border-gray-200 rounded-lg p-4">
      <div className="flex justify-between items-start mb-4">
        <div>
          <div className="text-sm text-gray-500 mb-1">Transaction #{txId}</div>
          <div className="font-medium">{transaction.receiver_id}</div>
        </div>
        <span className="px-3 py-1 bg-blue-100 text-blue-800 rounded-full text-sm">
          Pending
        </span>
      </div>

      <div className="space-y-2 mb-4">
        {transaction.actions.map((action, idx) => (
          <div key={idx} className="text-sm text-gray-700">
            • {getActionDescription(action)}
          </div>
        ))}
      </div>

      <div className="mb-4">
        <div className="flex items-center justify-between text-sm text-gray-600 mb-2">
          <div>
            Weight: {currentWeight} / {threshold}
          </div>
          <div>{progressPercent.toFixed(1)}%</div>
        </div>
        <div className="flex-1 bg-gray-200 rounded-full h-2">
          <div
            className="bg-blue-600 h-2 rounded-full transition-all"
            style={{ width: `${progressPercent}%` }}
          />
        </div>
      </div>

      <div className="mb-4">
        <div className="text-sm font-medium text-gray-700 mb-2">
          Approvals ({transaction.approvals.length}):
        </div>
        <div className="space-y-1">
          {transaction.approvals.map(([account, weight], idx) => (
            <div
              key={idx}
              className="flex items-center justify-between text-sm text-gray-600 bg-gray-50 px-3 py-2 rounded"
            >
              <span>{account.split('.')[0]}</span>
              <span className="font-medium text-blue-600">Weight: {weight}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="flex gap-2">
        {!hasApproved ? (
          <button
            onClick={handleApprove}
            disabled={approving}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
          >
            {approving ? 'Approving...' : 'Approve'}
          </button>
        ) : (
          <div className="px-4 py-2 bg-green-100 text-green-800 rounded-lg">
            ✓ You approved this
          </div>
        )}
      </div>
    </div>
  )
}
