import { Account } from 'near-api-js'

export interface Transaction {
  id: number
  receiver_id: string
  actions: Action[]
  confirmations: string[]
  executed: boolean
  cancelled: boolean
  storage_depositor: string
  expiration: number | null
}

export interface Action {
  Transfer?: { amount: string }
  FunctionCall?: {
    method_name: string
    args: number[]
    gas: number
    deposit: string
  }
}

export class MultisigContract {
  constructor(
    private account: Account,
    private contractId: string
  ) {}

  async getPendingTransactions(fromIndex: number = 0, limit: number = 100): Promise<Transaction[]> {
    return this.account.viewFunction({
      contractId: this.contractId,
      methodName: 'get_pending_transactions_paginated',
      args: { from_index: fromIndex, limit },
    })
  }

  async getTransaction(txId: number): Promise<Transaction | null> {
    return this.account.viewFunction({
      contractId: this.contractId,
      methodName: 'get_transaction',
      args: { tx_id: txId },
    })
  }

  async getOwners(): Promise<string[]> {
    return this.account.viewFunction({
      contractId: this.contractId,
      methodName: 'get_owners',
      args: {},
    })
  }

  async getNumConfirmations(): Promise<number> {
    return this.account.viewFunction({
      contractId: this.contractId,
      methodName: 'get_num_confirmations',
      args: {},
    })
  }

  async confirmTransaction(txId: number): Promise<void> {
    await this.account.functionCall({
      contractId: this.contractId,
      methodName: 'confirm_transaction',
      args: { tx_id: txId },
      gas: '30000000000000',
    })
  }

  async submitTransaction(
    receiverId: string,
    actions: Action[],
    expirationHours?: number
  ): Promise<number> {
    const result = await this.account.functionCall({
      contractId: this.contractId,
      methodName: 'submit_transaction',
      args: {
        receiver_id: receiverId,
        actions,
        expiration_hours: expirationHours ?? null
      },
      gas: '30000000000000',
      attachedDeposit: '10000000000000000000000', // 0.01 NEAR storage deposit
    })
    return result as any
  }

  async cancelTransaction(txId: number): Promise<void> {
    await this.account.functionCall({
      contractId: this.contractId,
      methodName: 'cancel_transaction',
      args: { tx_id: txId },
      gas: '30000000000000',
    })
  }

  async revokeConfirmation(txId: number): Promise<void> {
    await this.account.functionCall({
      contractId: this.contractId,
      methodName: 'revoke_confirmation',
      args: { tx_id: txId },
      gas: '30000000000000',
    })
  }

  async executeTransaction(txId: number): Promise<void> {
    await this.account.functionCall({
      contractId: this.contractId,
      methodName: 'execute_transaction',
      args: { tx_id: txId },
      gas: '100000000000000', // 100 TGas for execution
    })
  }

  async addOwner(newOwner: string): Promise<void> {
    await this.account.functionCall({
      contractId: this.contractId,
      methodName: 'add_owner',
      args: { new_owner: newOwner },
      gas: '30000000000000',
    })
  }

  async removeOwner(ownerToRemove: string): Promise<void> {
    await this.account.functionCall({
      contractId: this.contractId,
      methodName: 'remove_owner',
      args: { owner_to_remove: ownerToRemove },
      gas: '30000000000000',
    })
  }

  async changeThreshold(newThreshold: number): Promise<void> {
    await this.account.functionCall({
      contractId: this.contractId,
      methodName: 'change_threshold',
      args: { new_threshold: newThreshold },
      gas: '30000000000000',
    })
  }

  async getStorageDeposit(): Promise<string> {
    return this.account.viewFunction({
      contractId: this.contractId,
      methodName: 'get_storage_deposit',
      args: {},
    })
  }

  async cleanupOldTransactions(beforeIndex: number): Promise<number> {
    const result = await this.account.functionCall({
      contractId: this.contractId,
      methodName: 'cleanup_old_transactions',
      args: { before_index: beforeIndex },
      gas: '100000000000000', // 100 TGas for cleanup
    })
    return result as any
  }
}
