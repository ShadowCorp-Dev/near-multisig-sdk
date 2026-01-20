import { Account } from 'near-api-js'

export interface Action {
  Transfer?: { amount: string }
  FunctionCall?: {
    method: string
    args: number[]
    gas: number
    deposit: string
  }
}

export interface Transaction {
  receiver_id: string
  actions: Action[]
  approvals: [string, number][]  // [account_id, weight]
  total_weight: number
  executed: boolean
}

export class WeightedMultisigContract {
  constructor(private account: Account, private contractId: string) {}

  async getPendingTransactions(): Promise<Transaction[]> {
    return this.account.viewFunction({
      contractId: this.contractId,
      methodName: 'get_pending_transactions',
      args: {},
    })
  }

  async getTransaction(txId: number): Promise<Transaction | null> {
    return this.account.viewFunction({
      contractId: this.contractId,
      methodName: 'get_transaction',
      args: { tx_id: txId },
    })
  }

  async getOwners(): Promise<[string, number][]> {
    return this.account.viewFunction({
      contractId: this.contractId,
      methodName: 'get_owners',
      args: {},
    })
  }

  async getOwnerWeight(accountId: string): Promise<number | null> {
    return this.account.viewFunction({
      contractId: this.contractId,
      methodName: 'get_owner_weight',
      args: { account_id: accountId },
    })
  }

  async getApprovalThreshold(): Promise<number> {
    return this.account.viewFunction({
      contractId: this.contractId,
      methodName: 'get_approval_threshold',
      args: {},
    })
  }

  async getTotalWeight(): Promise<number> {
    return this.account.viewFunction({
      contractId: this.contractId,
      methodName: 'get_total_weight',
      args: {},
    })
  }

  async getTransactionProgress(txId: number): Promise<[number, number] | null> {
    return this.account.viewFunction({
      contractId: this.contractId,
      methodName: 'get_transaction_progress',
      args: { tx_id: txId },
    })
  }

  async approveTransaction(txId: number): Promise<void> {
    await this.account.functionCall({
      contractId: this.contractId,
      methodName: 'approve_transaction',
      args: { tx_id: txId },
      gas: '100000000000000',
    })
  }
}
