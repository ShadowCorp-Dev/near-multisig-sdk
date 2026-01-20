'use client'

import { Transaction } from '@/utils/contract'
import { formatNearAmount, shortenAddress } from '@/utils/near'

interface TransactionCardProps {
  transaction: Transaction
  threshold: number
  onApprove: (txId: number) => void
  currentUser: string | null
}

export default function TransactionCard({
  transaction,
  threshold,
  onApprove,
  currentUser,
}: TransactionCardProps) {
  const hasConfirmed = currentUser && transaction.confirmations.includes(currentUser)
  const canApprove = currentUser && !hasConfirmed && !transaction.executed
  const confirmationProgress = `${transaction.confirmations.length}/${threshold}`

  // Get action description
  const getActionDescription = () => {
    if (transaction.actions.length === 0) return 'No actions'
    const action = transaction.actions[0]

    if ('Transfer' in action) {
      return `Transfer ${formatNearAmount(action.Transfer.amount)} NEAR`
    }
    if ('FunctionCall' in action) {
      return `Call ${action.FunctionCall.method_name}()`
    }
    return 'Unknown action'
  }

  return (
    <div className="border border-gray-200 dark:border-gray-700 rounded-lg p-6 bg-white dark:bg-gray-800">
      <div className="flex justify-between items-start mb-4">
        <div>
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
            Transaction #{transaction.id}
          </h3>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
            To: {shortenAddress(transaction.receiver_id)}
          </p>
        </div>
        <div className="flex items-center gap-2">
          {transaction.executed ? (
            <span className="px-3 py-1 bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200 rounded-full text-sm font-medium">
              Executed
            </span>
          ) : (
            <span className="px-3 py-1 bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200 rounded-full text-sm font-medium">
              Pending
            </span>
          )}
        </div>
      </div>

      <div className="mb-4">
        <p className="text-gray-700 dark:text-gray-300">{getActionDescription()}</p>
      </div>

      <div className="mb-4">
        <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">
          Confirmations: {confirmationProgress}
        </p>
        <div className="flex flex-wrap gap-2">
          {transaction.confirmations.map((confirmer) => (
            <span
              key={confirmer}
              className="px-2 py-1 bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200 rounded text-xs"
            >
              {shortenAddress(confirmer)}
            </span>
          ))}
        </div>
      </div>

      {canApprove && (
        <button
          onClick={() => onApprove(transaction.id)}
          className="w-full px-4 py-2 bg-green-500 text-white rounded-lg hover:bg-green-600 transition-colors font-medium"
        >
          Approve Transaction
        </button>
      )}

      {hasConfirmed && !transaction.executed && (
        <div className="text-center py-2 text-green-600 dark:text-green-400 font-medium">
          You have approved this transaction
        </div>
      )}
    </div>
  )
}
