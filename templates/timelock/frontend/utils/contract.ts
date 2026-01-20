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
  confirmations: string[]
  scheduled_time: number | null
  executed: boolean
}

export class TimelockMultisigContract {
  constructor(private account: Account, private contractId: string) {}

  async getPendingTransactions(): Promise<Transaction[]> {
    return this.account.viewFunction({
      contractId: this.contractId,
      methodName: 'get_pending_transactions',
      args: {},
    })
  }

  async getScheduledTransactions(): Promise<Transaction[]> {
    return this.account.viewFunction({
      contractId: this.contractId,
      methodName: 'get_scheduled_transactions',
      args: {},
    })
  }

  async getExecutableTransactions(): Promise<Transaction[]> {
    return this.account.viewFunction({
      contractId: this.contractId,
      methodName: 'get_executable_transactions',
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

  async getTimelockDuration(): Promise<number> {
    return this.account.viewFunction({
      contractId: this.contractId,
      methodName: 'get_timelock_duration',
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

  async executeTransaction(txId: number): Promise<void> {
    await this.account.functionCall({
      contractId: this.contractId,
      methodName: 'execute_transaction',
      args: { tx_id: txId },
      gas: '100000000000000',
    })
  }
}
