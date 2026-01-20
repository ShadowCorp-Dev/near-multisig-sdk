'use client'

import { useState, useEffect } from 'react'
import { useWallet } from './providers/WalletProvider'
import { TimelockMultisigContract, Transaction } from '@/utils/contract'
import { formatNearAmount, formatTimestamp, getTimeRemaining } from '@/utils/near'

interface TransactionCardProps {
  transaction: Transaction
  txId: number
  threshold: number
  timelockDuration: number
  onApprove: () => void
  onExecute: () => void
  currentUser: string | null
}

export function TransactionCard({
  transaction,
  txId,
  threshold,
  timelockDuration,
  onApprove,
  onExecute,
  currentUser,
}: TransactionCardProps) {
  const { account } = useWallet()
  const [approving, setApproving] = useState(false)
  const [executing, setExecuting] = useState(false)
  const [timeRemaining, setTimeRemaining] = useState('')

  const hasApproved = currentUser && transaction.confirmations.includes(currentUser)
  const approvalCount = transaction.confirmations.length
  const isScheduled = transaction.scheduled_time !== null
  const canExecute =
    isScheduled &&
    transaction.scheduled_time !== null &&
    Date.now() * 1_000_000 >= transaction.scheduled_time

  useEffect(() => {
    if (transaction.scheduled_time) {
      const updateTime = () => {
        setTimeRemaining(getTimeRemaining(transaction.scheduled_time!))
      }
      updateTime()
      const interval = setInterval(updateTime, 1000)
      return () => clearInterval(interval)
    }
  }, [transaction.scheduled_time])

  async function handleApprove() {
    if (!account) return

    try {
      setApproving(true)
      const contract = new TimelockMultisigContract(
        account,
        account.connection.networkId === 'testnet'
          ? 'multisig.testnet'
          : 'multisig.near'
      )
      await contract.confirmTransaction(txId)
      onApprove()
    } catch (err: any) {
      alert(`Failed to approve: ${err.message}`)
    } finally {
      setApproving(false)
    }
  }

  async function handleExecute() {
    if (!account) return

    try {
      setExecuting(true)
      const contract = new TimelockMultisigContract(
        account,
        account.connection.networkId === 'testnet'
          ? 'multisig.testnet'
          : 'multisig.near'
      )
      await contract.executeTransaction(txId)
      onExecute()
    } catch (err: any) {
      alert(`Failed to execute: ${err.message}`)
    } finally {
      setExecuting(false)
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
        {isScheduled ? (
          canExecute ? (
            <span className="px-3 py-1 bg-green-100 text-green-800 rounded-full text-sm">
              Ready to Execute
            </span>
          ) : (
            <span className="px-3 py-1 bg-yellow-100 text-yellow-800 rounded-full text-sm">
              Scheduled
            </span>
          )
        ) : (
          <span className="px-3 py-1 bg-blue-100 text-blue-800 rounded-full text-sm">
            Pending Approval
          </span>
        )}
      </div>

      <div className="space-y-2 mb-4">
        {transaction.actions.map((action, idx) => (
          <div key={idx} className="text-sm text-gray-700">
            • {getActionDescription(action)}
          </div>
        ))}
      </div>

      <div className="flex items-center gap-2 mb-4">
        <div className="text-sm text-gray-600">
          Confirmations: {approvalCount} / {threshold}
        </div>
        <div className="flex-1 bg-gray-200 rounded-full h-2">
          <div
            className="bg-blue-600 h-2 rounded-full transition-all"
            style={{ width: `${(approvalCount / threshold) * 100}%` }}
          />
        </div>
      </div>

      {isScheduled && transaction.scheduled_time && (
        <div className="mb-4 p-3 bg-gray-50 rounded-lg">
          <div className="text-sm text-gray-600 mb-1">
            Scheduled for: {formatTimestamp(transaction.scheduled_time)}
          </div>
          <div className="text-sm font-medium text-gray-900">{timeRemaining}</div>
        </div>
      )}

      <div className="flex items-center justify-between text-sm text-gray-600 mb-4">
        <div>
          Confirmed by: {transaction.confirmations.map((c) => c.split('.')[0]).join(', ')}
        </div>
      </div>

      <div className="flex gap-2">
        {!isScheduled && !hasApproved && (
          <button
            onClick={handleApprove}
            disabled={approving}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
          >
            {approving ? 'Approving...' : 'Approve'}
          </button>
        )}

        {canExecute && (
          <button
            onClick={handleExecute}
            disabled={executing}
            className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
          >
            {executing ? 'Executing...' : 'Execute Transaction'}
          </button>
        )}

        {hasApproved && !isScheduled && (
          <div className="px-4 py-2 bg-green-100 text-green-800 rounded-lg">
            ✓ You approved this
          </div>
        )}
      </div>
    </div>
  )
}
