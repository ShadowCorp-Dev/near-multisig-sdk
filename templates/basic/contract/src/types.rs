use near_sdk::{near, AccountId};

/// Transaction submitted for multisig approval
#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Transaction {
    pub id: u64,
    pub receiver_id: AccountId,
    pub actions: Vec<Action>,
    pub confirmations: Vec<AccountId>,
    pub executed: bool,
    pub cancelled: bool,
    pub storage_depositor: AccountId, // Who paid storage deposit (gets refund)
    pub expiration: Option<u64>,      // Optional expiration timestamp (nanoseconds)
}

/// Actions that can be performed in a transaction
#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub enum Action {
    Transfer {
        amount: u128,
    },
    FunctionCall {
        method_name: String,
        args: Vec<u8>,
        gas: u64,
        deposit: u128,
    },
}

/// Events emitted for off-chain indexing
#[near(event_json(standard = "multisig"))]
pub enum MultisigEvent {
    #[event_version("1.0.0")]
    TransactionSubmitted {
        tx_id: u64,
        submitter: AccountId,
        receiver_id: AccountId,
    },

    #[event_version("1.0.0")]
    TransactionConfirmed {
        tx_id: u64,
        confirmer: AccountId,
        confirmations: u32,
    },

    #[event_version("1.0.0")]
    TransactionExecuted { tx_id: u64, success: bool },

    #[event_version("1.0.0")]
    TransactionCancelled { tx_id: u64, canceller: AccountId },

    #[event_version("1.0.0")]
    ConfirmationRevoked {
        tx_id: u64,
        revoker: AccountId,
        confirmations: u32,
    },

    #[event_version("1.0.0")]
    CallbackGasChanged {
        old_gas: u64,
        new_gas: u64,
        changer: AccountId,
    },

    #[event_version("1.0.0")]
    StorageDepositChanged {
        old_deposit: u128,
        new_deposit: u128,
        changer: AccountId,
    },

    #[event_version("1.0.0")]
    TransactionReady { tx_id: u64, confirmations: u32 },

    #[event_version("1.0.0")]
    TransactionsCleanedUp {
        count: u64,
        from_index: u64,
        to_index: u64,
        cleaner: AccountId,
    },
}
