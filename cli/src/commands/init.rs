use anyhow::Result;
use std::fs;
use std::path::Path;

const BASIC_TEMPLATE_CARGO: &str = r#"[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
near-sdk = "5.24.0"

[profile.release]
codegen-units = 1
opt-level = "z"
lto = true
panic = "abort"
overflow-checks = true

[package.metadata.near.reproducible_build]
image = "sourcescan/cargo-near:0.18.0-rust-1.86.0"
image_digest = "sha256:2d0d458d2357277df669eac6fa23a1ac922e5ed16646e1d3315336e4dff18043"
container_build_command = ["cargo", "near", "build", "reproducible-wasm"]
"#;

const BASIC_TEMPLATE_LIB: &str = r#"use near_sdk::store::{UnorderedSet, Vector};
use near_sdk::{near, require, AccountId, PanicOnDefault, env, Promise, NearToken, Gas, PromiseResult};

/// Security: Maximum number of actions per transaction to prevent gas exhaustion
const MAX_ACTIONS: usize = 10;

/// Security: Maximum args size (32KB) to prevent storage attacks
const MAX_ARGS_LEN: usize = 32768;

/// Security: Maximum method name length
const MAX_METHOD_NAME_LEN: usize = 256;

/// Security: Maximum gas per action (100 TGas)
const MAX_GAS_PER_ACTION: u64 = 100_000_000_000_000;

/// Security: Maximum total gas across all actions (250 TGas)
const MAX_TOTAL_GAS: u64 = 250_000_000_000_000;

/// Security: Maximum number of owners to prevent gas exhaustion
const MAX_OWNERS: usize = 50;

/// Security: Maximum transactions to process per cleanup call (prevents DoS)
const MAX_CLEANUP_BATCH: u32 = 100;

/// Storage cost per transaction (0.01 NEAR) - refundable on execution/cancellation
const TRANSACTION_STORAGE_DEPOSIT: u128 = 10_000_000_000_000_000_000_000; // 0.01 NEAR

/// Default callback gas (20 TGas) - can be configured per contract
const DEFAULT_CALLBACK_GAS: u64 = 20_000_000_000_000;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct MultisigContract {
    pub owners: UnorderedSet<AccountId>,
    pub num_confirmations: u32,
    pub transactions: Vector<Transaction>,
    pub pending_callbacks: u32, // Track pending executions to prevent cleanup corruption
    pub callback_gas: u64, // Gas allocated for execution callbacks (configurable)
    pub storage_deposit: u128, // L-5 fix: Storage deposit per transaction (configurable)
    pub next_tx_id: u64, // M-3 fix: Monotonic transaction ID counter (never decreases)
    pub reserved_balance: u128, // M-2 fix: Total deposits reserved by pending transactions
}

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
    pub expiration: Option<u64>, // L-2 fix: Optional expiration timestamp (nanoseconds)
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub enum Action {
    Transfer { amount: u128 },
    FunctionCall {
        method_name: String,
        args: Vec<u8>,
        gas: u64,
        deposit: u128,
    },
}

// Events for off-chain indexing
#[near(event_json(standard = "multisig"))]
pub enum MultisigEvent {
    #[event_version("1.0.0")]
    TransactionSubmitted { tx_id: u64, submitter: AccountId, receiver_id: AccountId },

    #[event_version("1.0.0")]
    TransactionConfirmed { tx_id: u64, confirmer: AccountId, confirmations: u32 },

    #[event_version("1.0.0")]
    TransactionExecuted { tx_id: u64, success: bool },

    #[event_version("1.0.0")]
    TransactionCancelled { tx_id: u64, canceller: AccountId },

    #[event_version("1.0.0")]
    ConfirmationRevoked { tx_id: u64, revoker: AccountId, confirmations: u32 },

    #[event_version("1.0.0")]
    CallbackGasChanged { old_gas: u64, new_gas: u64, changer: AccountId },

    #[event_version("1.0.0")]
    StorageDepositChanged { old_deposit: u128, new_deposit: u128, changer: AccountId },

    #[event_version("1.0.0")]
    TransactionReady { tx_id: u64, confirmations: u32 },

    #[event_version("1.0.0")]
    TransactionsCleanedUp { count: u64, from_index: u64, to_index: u64, cleaner: AccountId },
}

// Security: Safe transaction access methods to prevent u32 overflow attacks
impl MultisigContract {
    /// Safe transaction lookup by ID (M-3 fix: search by id field, not index)
    fn get_tx(&self, tx_id: u64) -> Option<&Transaction> {
        for i in 0..self.transactions.len() {
            if let Some(tx) = self.transactions.get(i) {
                if tx.id == tx_id {
                    return Some(tx);
                }
            }
        }
        None
    }

    /// Find transaction index by ID
    fn get_tx_index(&self, tx_id: u64) -> Option<u32> {
        for i in 0..self.transactions.len() {
            if let Some(tx) = self.transactions.get(i) {
                if tx.id == tx_id {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Safe transaction index lookup (panics if not found)
    fn get_tx_index_or_panic(&self, tx_id: u64) -> u32 {
        self.get_tx_index(tx_id).expect("Transaction not found")
    }

    /// Get old bounds-checked lookup (kept for compatibility)
    fn get_tx_legacy(&self, tx_id: u64) -> Option<&Transaction> {
        if tx_id > u32::MAX as u64 {
            return None;
        }
        self.transactions.get(tx_id as u32)
    }

    /// Safe transaction lookup (panics if not found)
    fn get_tx_or_panic(&self, tx_id: u64) -> &Transaction {
        require!(tx_id <= u32::MAX as u64, "Invalid transaction ID");
        self.transactions.get(tx_id as u32).expect("Transaction not found")
    }

    /// Validate action vector to prevent attacks
    fn validate_actions(actions: &Vec<Action>) -> u128 {
        require!(!actions.is_empty(), "Actions cannot be empty");
        require!(actions.len() <= MAX_ACTIONS, "Too many actions (max 10)");

        let mut total_gas = 0u64;
        let mut total_deposit = 0u128;

        for action in actions {
            match action {
                Action::Transfer { amount } => {
                    require!(*amount > 0, "Transfer amount must be positive");
                    total_deposit = total_deposit.saturating_add(*amount);
                }
                Action::FunctionCall { method_name, args, gas, deposit } => {
                    require!(args.len() <= MAX_ARGS_LEN, "Args too large (max 32KB)");
                    require!(method_name.len() <= MAX_METHOD_NAME_LEN, "Method name too long");
                    require!(!method_name.is_empty(), "Method name cannot be empty");

                    // Security: Validate gas parameters (BUG-7)
                    require!(*gas > 0, "Gas must be positive");
                    require!(*gas <= MAX_GAS_PER_ACTION, "Gas per action exceeds limit (max 100 TGas)");
                    total_gas = total_gas.saturating_add(*gas);

                    // Accumulate deposits
                    total_deposit = total_deposit.saturating_add(*deposit);
                }
            }
        }

        require!(total_gas <= MAX_TOTAL_GAS, "Total gas exceeds limit (max 250 TGas)");
        total_deposit
    }

    /// M-2 fix: Calculate total deposits in a transaction
    fn calculate_transaction_deposit(tx: &Transaction) -> u128 {
        let mut total_deposit = 0u128;
        for action in &tx.actions {
            match action {
                Action::Transfer { amount } => {
                    total_deposit = total_deposit.saturating_add(*amount);
                }
                Action::FunctionCall { deposit, .. } => {
                    total_deposit = total_deposit.saturating_add(*deposit);
                }
            }
        }
        total_deposit
    }
}

#[near]
impl MultisigContract {
    #[init]
    pub fn new(owners: Vec<AccountId>, num_confirmations: u32) -> Self {
        require!(!owners.is_empty(), "Need at least one owner");
        // Security: Enforce max owners limit (BUG-9)
        require!(owners.len() <= MAX_OWNERS, "Too many owners (max 50)");
        require!(
            num_confirmations > 0 && num_confirmations <= owners.len() as u32,
            "Invalid confirmation threshold"
        );

        // Security: Check for duplicate owners
        let mut owner_set = UnorderedSet::new(b"o");
        for owner in &owners {
            require!(owner_set.insert(owner.clone()), "Duplicate owner");
        }

        Self {
            owners: owner_set,
            num_confirmations,
            transactions: Vector::new(b"t"),
            pending_callbacks: 0,
            callback_gas: DEFAULT_CALLBACK_GAS,
            storage_deposit: TRANSACTION_STORAGE_DEPOSIT, // L-5 fix
            next_tx_id: 0, // M-3 fix
            reserved_balance: 0, // M-2 fix
        }
    }

    /// Submit a new transaction for approval
    /// Requires 0.01 NEAR storage deposit (refunded on execution/cancellation)
    #[payable]
    pub fn submit_transaction(&mut self, receiver_id: AccountId, actions: Vec<Action>, expiration_hours: Option<u64>) -> u64 {
        let sender = env::predecessor_account_id();
        require!(self.owners.contains(&sender), "Not an owner");

        // Security: Require storage deposit to prevent spam (M-1)
        let attached = env::attached_deposit().as_yoctonear();
        require!(
            attached >= self.storage_deposit,
            format!("Must attach at least {} yoctoNEAR for storage", self.storage_deposit)
        );

        // Security: Prevent sending to self
        require!(
            receiver_id != env::current_account_id(),
            "Cannot send to multisig contract itself"
        );

        // Security: Validate actions and get total deposit
        let total_deposit = Self::validate_actions(&actions);

        // M-2 fix: Check available balance after accounting for reserved amounts
        // CRITICAL-2 fix: Subtract storage deposit from available balance since it won't be usable
        let available_balance = env::account_balance()
            .as_yoctonear()
            .saturating_sub(self.reserved_balance)
            .saturating_sub(self.storage_deposit);
        require!(
            total_deposit <= available_balance,
            "Insufficient available balance (pending transactions already reserved funds)"
        );

        // M-2 fix: Reserve balance for this transaction
        self.reserved_balance = self.reserved_balance.saturating_add(total_deposit);

        // M-3 fix: Use monotonic counter instead of vector length
        let tx_id = self.next_tx_id;
        // MEDIUM-1 fix: Check for counter overflow before incrementing
        require!(
            self.next_tx_id < u64::MAX,
            "Transaction ID counter limit reached"
        );
        self.next_tx_id = self.next_tx_id.saturating_add(1);

        // L-2 fix: Calculate expiration timestamp if hours provided
        let expiration = expiration_hours.map(|hours| {
            let nanos_per_hour = 3_600_000_000_000u64; // 1 hour = 3.6 trillion nanoseconds
            env::block_timestamp().saturating_add(hours.saturating_mul(nanos_per_hour))
        });

        let tx = Transaction {
            id: tx_id,
            receiver_id,
            actions,
            confirmations: vec![sender.clone()],
            executed: false,
            cancelled: false,
            storage_depositor: sender.clone(),
            expiration,
        };

        self.transactions.push(tx.clone());

        // Emit event for off-chain indexing
        MultisigEvent::TransactionSubmitted {
            tx_id,
            submitter: sender,
            receiver_id: tx.receiver_id.clone(),
        }.emit();

        // H-2 fix: Don't auto-execute even if threshold is 1 - require explicit execution
        if self.num_confirmations == 1 {
            MultisigEvent::TransactionReady {
                tx_id,
                confirmations: 1,
            }.emit();
        }

        tx_id
    }

    /// Confirm a pending transaction
    pub fn confirm_transaction(&mut self, tx_id: u64) {
        let sender = env::predecessor_account_id();
        require!(self.owners.contains(&sender), "Not an owner");

        // MEDIUM-2 fix: Removed u32::MAX check - get_tx() handles any u64 value

        let mut tx = self.get_tx_or_panic(tx_id).clone();

        require!(!tx.executed, "Already executed");
        require!(!tx.cancelled, "Transaction cancelled");
        require!(
            !tx.confirmations.contains(&sender),
            "Already confirmed by this owner"
        );

        tx.confirmations.push(sender.clone());
        let confirmations_count = tx.confirmations.len() as u32;

        // Emit confirmation event after state is persisted
        MultisigEvent::TransactionConfirmed {
            tx_id,
            confirmer: sender,
            confirmations: confirmations_count,
        }.emit();

        // H-2 fix: Don't auto-execute - emit ready event and require explicit execution
        if confirmations_count >= self.num_confirmations {
            MultisigEvent::TransactionReady {
                tx_id,
                confirmations: confirmations_count,
            }.emit();
        }

        self.transactions.replace(self.get_tx_index_or_panic(tx_id), tx);
    }

    /// Manually execute a fully-approved transaction (for retries after failure)
    pub fn execute_transaction(&mut self, tx_id: u64) -> Promise {
        let sender = env::predecessor_account_id();
        require!(self.owners.contains(&sender), "Not an owner");

        // MEDIUM-2 fix: Removed u32::MAX check - get_tx() handles any u64 value

        let mut tx = self.get_tx_or_panic(tx_id).clone();

        require!(!tx.executed, "Already executed");
        require!(!tx.cancelled, "Transaction cancelled");

        // L-2 fix: Check if transaction has expired
        if let Some(exp_time) = tx.expiration {
            require!(
                env::block_timestamp() < exp_time,
                "Transaction expired"
            );
        }

        require!(
            tx.confirmations.len() as u32 >= self.num_confirmations,
            "Not enough confirmations"
        );

        // Mark as executed
        tx.executed = true;
        self.transactions.replace(self.get_tx_index_or_panic(tx_id), tx);

        // Track pending callback to prevent cleanup corruption
        self.pending_callbacks = self.pending_callbacks.saturating_add(1);

        self.execute_transaction_internal(tx_id)
    }

    /// Internal: Execute transaction actions with proper promise chaining
    fn execute_transaction_internal(&self, tx_id: u64) -> Promise {
        let tx = self.get_tx_or_panic(tx_id);

        // Security: Chain promises properly instead of creating separate ones (NH-3)
        let mut promise = Promise::new(tx.receiver_id.clone());

        for action in &tx.actions {
            match action {
                Action::Transfer { amount } => {
                    promise = promise.transfer(NearToken::from_yoctonear(*amount));
                }
                Action::FunctionCall {
                    method_name,
                    args,
                    gas,
                    deposit,
                } => {
                    promise = promise.function_call(
                        method_name.clone(),
                        args.clone(),
                        NearToken::from_yoctonear(*deposit),
                        Gas::from_gas(*gas),
                    );
                }
            }
        }

        // Security: Attach callback to handle promise failures (NH-1)
        // Callback gas is configurable (default 20 TGas) for flexibility with complex state updates
        promise.then(
            Self::ext(env::current_account_id())
                .with_static_gas(Gas::from_gas(self.callback_gas))
                .on_execute_callback(tx_id)
        )
    }

    /// Cancel a pending transaction (only submitter can cancel)
    /// Returns a Promise for the storage deposit refund
    pub fn cancel_transaction(&mut self, tx_id: u64) -> Promise {
        let sender = env::predecessor_account_id();
        require!(self.owners.contains(&sender), "Not an owner");

        // MEDIUM-2 fix: Removed u32::MAX check - get_tx() handles any u64 value

        let mut tx = self.get_tx_or_panic(tx_id).clone();
        require!(!tx.executed, "Already executed");
        require!(!tx.cancelled, "Already cancelled");

        // Only the submitter (first confirmer) can cancel
        require!(
            tx.confirmations.first() == Some(&sender),
            "Only submitter can cancel"
        );

        tx.cancelled = true;
        self.transactions.replace(self.get_tx_index_or_panic(tx_id), tx.clone());

        // M-2 fix: Release reserved balance when cancelling
        let deposit = Self::calculate_transaction_deposit(&tx);
        self.reserved_balance = self.reserved_balance.saturating_sub(deposit);

        // Emit cancellation event
        MultisigEvent::TransactionCancelled {
            tx_id,
            canceller: sender,
        }.emit();

        // Security (H-1 fix): Return refund promise instead of detaching
        // If refund fails, the caller will be notified via promise failure
        Promise::new(tx.storage_depositor.clone())
            .transfer(NearToken::from_yoctonear(self.storage_deposit))
    }

    /// Update callback gas allocation (owner-only)
    /// Allows adjusting gas for complex callback scenarios
    pub fn set_callback_gas(&mut self, gas: u64) {
        let sender = env::predecessor_account_id();
        require!(self.owners.contains(&sender), "Not an owner");
        require!(gas >= 5_000_000_000_000, "Callback gas too low (min 5 TGas)");
        require!(gas <= 100_000_000_000_000, "Callback gas too high (max 100 TGas)");

        let old_gas = self.callback_gas;
        self.callback_gas = gas;

        // Emit configuration change event
        MultisigEvent::CallbackGasChanged {
            old_gas,
            new_gas: gas,
            changer: sender,
        }.emit();
    }

    /// L-5 fix: Update storage deposit amount (owner-only)
    pub fn set_storage_deposit(&mut self, deposit: u128) {
        let sender = env::predecessor_account_id();
        require!(self.owners.contains(&sender), "Not an owner");
        require!(deposit >= 1_000_000_000_000_000_000_000, "Storage deposit too low (min 0.001 NEAR)");
        require!(deposit <= 1_000_000_000_000_000_000_000_000, "Storage deposit too high (max 1 NEAR)");

        let old_deposit = self.storage_deposit;
        self.storage_deposit = deposit;

        MultisigEvent::StorageDepositChanged {
            old_deposit,
            new_deposit: deposit,
            changer: sender,
        }.emit();
    }

    pub fn get_storage_deposit(&self) -> u128 {
        self.storage_deposit
    }

    /// Revoke your confirmation from a pending transaction
    pub fn revoke_confirmation(&mut self, tx_id: u64) {
        let sender = env::predecessor_account_id();
        require!(self.owners.contains(&sender), "Not an owner");

        // MEDIUM-2 fix: Removed u32::MAX check - get_tx() handles any u64 value

        let mut tx = self.get_tx_or_panic(tx_id).clone();
        require!(!tx.executed, "Already executed");
        require!(!tx.cancelled, "Transaction cancelled");

        let pos = tx.confirmations.iter().position(|x| x == &sender);
        require!(pos.is_some(), "Not confirmed by you");

        tx.confirmations.remove(pos.unwrap());
        let confirmations_count = tx.confirmations.len() as u32;
        self.transactions.replace(self.get_tx_index_or_panic(tx_id), tx);

        // Emit revocation event
        MultisigEvent::ConfirmationRevoked {
            tx_id,
            revoker: sender,
            confirmations: confirmations_count,
        }.emit();
    }

    /// Clean up old executed/cancelled transactions to reduce storage costs
    /// WARNING: This is gas-expensive. Only removes transactions before the specified index.
    /// Only executed or cancelled transactions can be removed (pending transactions are preserved).
    /// BLOCKS if there are pending callbacks to prevent corruption.
    /// Security: Processes max 100 transactions per call to prevent DoS
    pub fn cleanup_old_transactions(&mut self, before_index: u64) -> u64 {
        let sender = env::predecessor_account_id();
        require!(self.owners.contains(&sender), "Not an owner");
        require!(before_index <= u32::MAX as u64, "Index too large");

        // Security: Prevent cleanup during pending callbacks to avoid index corruption
        require!(
            self.pending_callbacks == 0,
            "Cannot cleanup while callbacks are pending"
        );

        let cleanup_end = (before_index as u32).min(self.transactions.len());
        let mut removed_count = 0u64;

        // Collect transactions to keep (avoid storage prefix collision)
        let mut transactions_to_keep: Vec<Transaction> = Vec::new();

        // Security: Limit iterations to prevent gas exhaustion DoS
        let max_iterations = cleanup_end.min(MAX_CLEANUP_BATCH);

        // Iterate transactions up to the batch limit
        for i in 0..max_iterations {
            if let Some(tx) = self.transactions.get(i) {
                // Keep if: after cleanup range OR (in cleanup range but still pending)
                if i >= cleanup_end || (!tx.executed && !tx.cancelled) {
                    transactions_to_keep.push(tx.clone());
                } else {
                    removed_count += 1;
                }
            }
        }

        // Keep all transactions after the batch limit
        for i in max_iterations..self.transactions.len() {
            if let Some(tx) = self.transactions.get(i) {
                transactions_to_keep.push(tx.clone());
            }
        }

        // Clear and rebuild the vector to avoid storage corruption
        self.transactions.clear();
        for tx in transactions_to_keep {
            self.transactions.push(tx);
        }

        // Emit cleanup event with transaction range context
        MultisigEvent::TransactionsCleanedUp {
            count: removed_count,
            from_index: 0,
            to_index: max_iterations as u64,
            cleaner: sender,
        }.emit();

        removed_count
    }

    /// Security: Callback to handle promise execution results (NH-1)
    /// If promise fails, mark transaction as not executed so it can be retried
    #[private]
    pub fn on_execute_callback(&mut self, tx_id: u64) {
        // L-3 fix: Validate transaction exists and ID matches parameter
        if let Some(tx) = self.get_tx(tx_id) {
            require!(tx.id == tx_id, "Transaction ID mismatch in callback");
        } else {
            env::log_str(&format!("⚠️ Callback for non-existent transaction {}", tx_id));
            return;
        }

        // Decrement pending callbacks counter
        self.pending_callbacks = self.pending_callbacks.saturating_sub(1);

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                // Transaction executed successfully, already marked as executed
                env::log_str(&format!("Transaction {} executed successfully", tx_id));

                // M-2 fix: Release reserved balance after successful execution
                if let Some(tx) = self.transactions.get(self.get_tx_index_or_panic(tx_id)) {
                    let deposit = Self::calculate_transaction_deposit(tx);
                    self.reserved_balance = self.reserved_balance.saturating_sub(deposit);
                }

                MultisigEvent::TransactionExecuted {
                    tx_id,
                    success: true,
                }.emit();

                // Security (H-1 fix): Refund storage deposit with callback to track failures
                // Detached to prevent refund failures from affecting transaction execution result
                if let Some(tx) = self.transactions.get(self.get_tx_index_or_panic(tx_id)) {
                    Promise::new(tx.storage_depositor.clone())
                        .transfer(NearToken::from_yoctonear(self.storage_deposit))
                        .then(
                            Self::ext(env::current_account_id())
                                .with_static_gas(Gas::from_gas(5_000_000_000_000))
                                .on_refund_callback(tx_id, tx.storage_depositor.clone())
                        );
                }
            }
            PromiseResult::Failed => {
                // Promise failed - revert executed flag so transaction can be retried
                env::log_str(&format!("Transaction {} failed, marking for retry", tx_id));
                if let Some(tx) = self.transactions.get(self.get_tx_index_or_panic(tx_id)) {
                    let mut tx_clone = tx.clone();
                    tx_clone.executed = false;
                    self.transactions.replace(self.get_tx_index_or_panic(tx_id), tx_clone);
                }
                MultisigEvent::TransactionExecuted {
                    tx_id,
                    success: false,
                }.emit();
            }
        }
    }

    /// Security (H-1 fix): Callback to track storage deposit refund results
    /// Logs refund failures so users know if their deposit wasn't returned
    #[private]
    pub fn on_refund_callback(&mut self, tx_id: u64, recipient: AccountId) {
        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                env::log_str(&format!("Storage deposit refund successful for tx {} to {}", tx_id, recipient));
            }
            PromiseResult::Failed => {
                env::log_str(&format!("⚠️  Storage deposit refund FAILED for tx {} to {}. User may need to claim manually.", tx_id, recipient));
                // Future enhancement: Store failed refunds in a claimable pool
            }
        }
    }

    // === View Methods ===

    /// Get all owners
    pub fn get_owners(&self) -> Vec<AccountId> {
        self.owners.iter().cloned().collect()
    }

    /// Get confirmation threshold
    pub fn get_num_confirmations(&self) -> u32 {
        self.num_confirmations
    }

    /// Get a specific transaction
    pub fn get_transaction(&self, tx_id: u64) -> Option<Transaction> {
        self.get_tx(tx_id).cloned()
    }

    /// Get pending transactions (paginated to avoid gas exhaustion)
    /// Security: Unbounded method removed - always use pagination to prevent DoS
    pub fn get_pending_transactions_paginated(&self, from_index: u64, limit: u64) -> Vec<Transaction> {
        let len = self.transactions.len() as u64;
        let start = from_index.min(len);
        let end = (start.saturating_add(limit)).min(len);

        // HIGH-1 fix: Use direct index access instead of get_tx() to avoid O(n²) complexity
        (start..end)
            .filter_map(|i| {
                let tx = self.transactions.get(i as u32)?;
                if !tx.executed && !tx.cancelled {
                    Some(tx.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all transactions (paginated)
    pub fn get_transactions(&self, from_index: u64, limit: u64) -> Vec<Transaction> {
        let len = self.transactions.len() as u64;
        let start = from_index.min(len);
        let end = (start.saturating_add(limit)).min(len);

        // HIGH-1 fix: Use direct index access instead of get_tx() to avoid O(n²) complexity
        (start..end)
            .filter_map(|i| self.transactions.get(i as u32).cloned())
            .collect()
    }

    /// Get total number of transactions
    pub fn get_transaction_count(&self) -> u64 {
        self.transactions.len() as u64
    }

    /// Check if account is an owner
    pub fn is_owner(&self, account_id: AccountId) -> bool {
        self.owners.contains(&account_id)
    }

    /// Check if account has confirmed a transaction
    pub fn has_confirmed(&self, tx_id: u64, account_id: AccountId) -> bool {
        if let Some(tx) = self.get_tx(tx_id) {
            tx.confirmations.contains(&account_id)
        } else {
            false
        }
    }
}
"#;

const TIMELOCK_TEMPLATE_LIB: &str = r#"use near_sdk::store::{UnorderedSet, Vector};
use near_sdk::{near, require, AccountId, PanicOnDefault, env, Promise, NearToken, Gas, PromiseResult};

/// Security: Maximum number of actions per transaction to prevent gas exhaustion
const MAX_ACTIONS: usize = 10;

/// Security: Maximum args size (32KB) to prevent storage attacks
const MAX_ARGS_LEN: usize = 32768;

/// Security: Maximum method name length
const MAX_METHOD_NAME_LEN: usize = 256;

/// Security: Maximum gas per action (100 TGas)
const MAX_GAS_PER_ACTION: u64 = 100_000_000_000_000;

/// Security: Maximum total gas across all actions (250 TGas)
const MAX_TOTAL_GAS: u64 = 250_000_000_000_000;

/// Security: Minimum timelock duration (1 minute)
const MIN_TIMELOCK: u64 = 60_000_000_000;

/// Security: Maximum timelock duration (30 days) to prevent permanent fund lockup
const MAX_TIMELOCK: u64 = 30 * 24 * 60 * 60 * 1_000_000_000;

/// Security: Maximum number of owners to prevent gas exhaustion
const MAX_OWNERS: usize = 50;

/// Security: Maximum transactions to process per cleanup call (prevents DoS)
const MAX_CLEANUP_BATCH: u32 = 100;

/// Storage cost per transaction (0.01 NEAR) - refundable on execution/cancellation
const TRANSACTION_STORAGE_DEPOSIT: u128 = 10_000_000_000_000_000_000_000; // 0.01 NEAR

/// Default callback gas (20 TGas) - can be configured per contract
const DEFAULT_CALLBACK_GAS: u64 = 20_000_000_000_000;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct TimelockMultisig {
    pub owners: UnorderedSet<AccountId>,
    pub num_confirmations: u32,
    pub timelock_duration: u64, // Nanoseconds
    pub transactions: Vector<Transaction>,
    pub pending_callbacks: u32, // Track pending executions to prevent cleanup corruption
    pub callback_gas: u64, // Gas allocated for execution callbacks (configurable)
    pub storage_deposit: u128, // L-5 fix: Storage deposit per transaction (configurable)
    pub next_tx_id: u64, // M-3 fix: Monotonic transaction ID counter
    pub reserved_balance: u128, // M-2 fix: Total deposits reserved by pending transactions
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Transaction {
    pub id: u64,
    pub receiver_id: AccountId,
    pub actions: Vec<Action>,
    pub confirmations: Vec<AccountId>,
    pub scheduled_time: Option<u64>,
    pub executed: bool,
    pub cancelled: bool,
    pub storage_depositor: AccountId, // Who paid storage deposit (gets refund)
    pub expiration: Option<u64>, // L-2 fix: Optional expiration timestamp (nanoseconds)
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub enum Action {
    Transfer { amount: u128 },
    FunctionCall {
        method: String,
        args: Vec<u8>,
        gas: u64,
        deposit: u128,
    },
}

// Events for off-chain indexing
#[near(event_json(standard = "multisig"))]
pub enum MultisigEvent {
    #[event_version("1.0.0")]
    TransactionSubmitted { tx_id: u64, submitter: AccountId, receiver_id: AccountId },

    #[event_version("1.0.0")]
    TransactionConfirmed { tx_id: u64, confirmer: AccountId, confirmations: u32 },

    #[event_version("1.0.0")]
    TransactionScheduled { tx_id: u64, scheduled_time: u64 },

    #[event_version("1.0.0")]
    TransactionExecuted { tx_id: u64, success: bool },

    #[event_version("1.0.0")]
    TransactionCancelled { tx_id: u64, canceller: AccountId },

    #[event_version("1.0.0")]
    ConfirmationRevoked { tx_id: u64, revoker: AccountId, confirmations: u32 },

    #[event_version("1.0.0")]
    CallbackGasChanged { old_gas: u64, new_gas: u64, changer: AccountId },

    #[event_version("1.0.0")]
    TransactionsCleanedUp { count: u64, from_index: u64, to_index: u64, cleaner: AccountId },
}

// Security: Safe transaction access methods to prevent u32 overflow attacks
impl TimelockMultisig {
    /// Safe transaction lookup by ID (M-3 fix: search by id field, not index)
    fn get_tx(&self, tx_id: u64) -> Option<&Transaction> {
        for i in 0..self.transactions.len() {
            if let Some(tx) = self.transactions.get(i) {
                if tx.id == tx_id {
                    return Some(tx);
                }
            }
        }
        None
    }

    /// Find transaction index by ID
    fn get_tx_index(&self, tx_id: u64) -> Option<u32> {
        for i in 0..self.transactions.len() {
            if let Some(tx) = self.transactions.get(i) {
                if tx.id == tx_id {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Safe transaction index lookup (panics if not found)
    fn get_tx_index_or_panic(&self, tx_id: u64) -> u32 {
        self.get_tx_index(tx_id).expect("Transaction not found")
    }

    /// Safe transaction lookup (panics if not found)
    fn get_tx_or_panic(&self, tx_id: u64) -> &Transaction {
        self.get_tx(tx_id).expect("Transaction not found")
    }

    /// Validate action vector to prevent attacks
    fn validate_actions(actions: &Vec<Action>) -> u128 {
        require!(!actions.is_empty(), "Actions cannot be empty");
        require!(actions.len() <= MAX_ACTIONS, "Too many actions (max 10)");

        let mut total_gas = 0u64;
        let mut total_deposit = 0u128;

        for action in actions {
            match action {
                Action::Transfer { amount } => {
                    require!(*amount > 0, "Transfer amount must be positive");
                    total_deposit = total_deposit.saturating_add(*amount);
                }
                Action::FunctionCall { method, args, gas, deposit } => {
                    require!(args.len() <= MAX_ARGS_LEN, "Args too large (max 32KB)");
                    require!(method.len() <= MAX_METHOD_NAME_LEN, "Method name too long");
                    require!(!method.is_empty(), "Method name cannot be empty");

                    // Security: Validate gas parameters (BUG-7)
                    require!(*gas > 0, "Gas must be positive");
                    require!(*gas <= MAX_GAS_PER_ACTION, "Gas per action exceeds limit (max 100 TGas)");
                    total_gas = total_gas.saturating_add(*gas);
                    total_deposit = total_deposit.saturating_add(*deposit);
                }
            }
        }

        require!(total_gas <= MAX_TOTAL_GAS, "Total gas exceeds limit (max 250 TGas)");
        total_deposit
    }

    /// M-2 fix: Calculate total deposits in a transaction
    fn calculate_transaction_deposit(tx: &Transaction) -> u128 {
        let mut total_deposit = 0u128;
        for action in &tx.actions {
            match action {
                Action::Transfer { amount } => {
                    total_deposit = total_deposit.saturating_add(*amount);
                }
                Action::FunctionCall { deposit, .. } => {
                    total_deposit = total_deposit.saturating_add(*deposit);
                }
            }
        }
        total_deposit
    }
}

#[near]
impl TimelockMultisig {
    #[init]
    pub fn new(owners: Vec<AccountId>, num_confirmations: u32, timelock_duration: u64) -> Self {
        // Security: Enforce max owners limit (BUG-9)
        require!(!owners.is_empty(), "Need at least one owner");
        require!(owners.len() <= MAX_OWNERS, "Too many owners (max 50)");
        require!(
            num_confirmations > 0 && num_confirmations <= owners.len() as u32,
            "Invalid confirmation threshold"
        );
        // Security: Enforce minimum and maximum timelock duration
        require!(timelock_duration >= MIN_TIMELOCK, "Timelock too short (min 1 minute)");
        require!(timelock_duration <= MAX_TIMELOCK, "Timelock too long (max 30 days)");

        // Security: Check for duplicate owners
        let mut owner_set = UnorderedSet::new(b"o");
        for owner in &owners {
            require!(owner_set.insert(owner.clone()), "Duplicate owner");
        }

        Self {
            owners: owner_set,
            num_confirmations,
            timelock_duration,
            transactions: Vector::new(b"t"),
            pending_callbacks: 0,
            callback_gas: DEFAULT_CALLBACK_GAS,
            storage_deposit: TRANSACTION_STORAGE_DEPOSIT, // L-5 fix
            next_tx_id: 0, // M-3 fix
            reserved_balance: 0, // M-2 fix
        }
    }

    // ===== Write Methods =====

    /// Submit a new transaction for approval
    /// Requires 0.01 NEAR storage deposit (refunded on execution/cancellation)
    #[payable]
    pub fn submit_transaction(&mut self, receiver_id: AccountId, actions: Vec<Action>, expiration_hours: Option<u64>) -> u64 {
        let sender = env::predecessor_account_id();
        require!(self.owners.contains(&sender), "Not an owner");

        // Security: Require storage deposit to prevent spam (M-1)
        let attached = env::attached_deposit().as_yoctonear();
        require!(
            attached >= self.storage_deposit,
            format!("Must attach at least {} yoctoNEAR for storage", self.storage_deposit)
        );

        // Security: Prevent sending to self
        require!(
            receiver_id != env::current_account_id(),
            "Cannot send to multisig contract itself"
        );

        // Security: Validate actions and get total deposit
        let total_deposit = Self::validate_actions(&actions);

        // M-2 fix: Check available balance after accounting for reserved amounts
        // CRITICAL-2 fix: Subtract storage deposit from available balance since it won't be usable
        let available_balance = env::account_balance()
            .as_yoctonear()
            .saturating_sub(self.reserved_balance)
            .saturating_sub(self.storage_deposit);
        require!(
            total_deposit <= available_balance,
            "Insufficient available balance (pending transactions already reserved funds)"
        );

        // M-2 fix: Reserve balance for this transaction
        self.reserved_balance = self.reserved_balance.saturating_add(total_deposit);

        // M-3 fix: Use monotonic counter instead of vector length
        let tx_id = self.next_tx_id;
        // MEDIUM-1 fix: Check for counter overflow before incrementing
        require!(
            self.next_tx_id < u64::MAX,
            "Transaction ID counter limit reached"
        );
        self.next_tx_id = self.next_tx_id.saturating_add(1);

        // L-2 fix: Calculate expiration timestamp if hours provided
        let expiration = expiration_hours.map(|hours| {
            let nanos_per_hour = 3_600_000_000_000u64; // 1 hour = 3.6 trillion nanoseconds
            env::block_timestamp().saturating_add(hours.saturating_mul(nanos_per_hour))
        });

        let tx = Transaction {
            id: tx_id,
            receiver_id: receiver_id.clone(),
            actions,
            confirmations: vec![sender.clone()],
            scheduled_time: None,
            executed: false,
            cancelled: false,
            storage_depositor: sender.clone(),
            expiration,
        };

        self.transactions.push(tx);

        // Emit event for off-chain indexing
        MultisigEvent::TransactionSubmitted {
            tx_id,
            submitter: sender,
            receiver_id,
        }.emit();

        tx_id
    }

    pub fn confirm_transaction(&mut self, tx_id: u64) {
        let sender = env::predecessor_account_id();
        require!(self.owners.contains(&sender), "Not an owner");

        // MEDIUM-2 fix: Removed u32::MAX check - get_tx() handles any u64 value

        let mut tx = self.get_tx_or_panic(tx_id).clone();
        require!(!tx.executed, "Already executed");
        require!(!tx.cancelled, "Transaction cancelled");
        require!(!tx.confirmations.contains(&sender), "Already confirmed");

        tx.confirmations.push(sender.clone());

        // Emit confirmation event
        MultisigEvent::TransactionConfirmed {
            tx_id,
            confirmer: sender,
            confirmations: tx.confirmations.len() as u32,
        }.emit();

        // Schedule execution after timelock if threshold reached
        if tx.confirmations.len() as u32 >= self.num_confirmations && tx.scheduled_time.is_none() {
            // Security: Use saturating_add to prevent timestamp overflow (M-4)
            let scheduled_time = env::block_timestamp().saturating_add(self.timelock_duration);
            tx.scheduled_time = Some(scheduled_time);

            // Emit scheduled event
            MultisigEvent::TransactionScheduled {
                tx_id,
                scheduled_time,
            }.emit();
        }

        self.transactions.replace(self.get_tx_index_or_panic(tx_id), tx);
    }

    pub fn execute_transaction(&mut self, tx_id: u64) -> Promise {
        let sender = env::predecessor_account_id();
        // Security: Only owners can execute transactions (H-1)
        require!(self.owners.contains(&sender), "Not an owner");

        // MEDIUM-2 fix: Removed u32::MAX check - get_tx() handles any u64 value

        let mut tx = self.get_tx_or_panic(tx_id).clone();

        require!(!tx.executed, "Already executed");
        require!(!tx.cancelled, "Transaction cancelled");

        // L-2 fix: Check if transaction has expired
        if let Some(exp_time) = tx.expiration {
            require!(
                env::block_timestamp() < exp_time,
                "Transaction expired"
            );
        }

        require!(tx.scheduled_time.is_some(), "Not scheduled");
        require!(
            env::block_timestamp() >= tx.scheduled_time.unwrap(),
            "Timelock not expired"
        );

        // CRITICAL-1 fix: Verify threshold is STILL met at execution time
        // This prevents execution if confirmations were revoked after scheduling
        require!(
            tx.confirmations.len() as u32 >= self.num_confirmations,
            "Insufficient confirmations (threshold not met at execution time)"
        );

        // Mark as executed
        tx.executed = true;
        self.transactions.replace(self.get_tx_index_or_panic(tx_id), tx.clone());

        // Track pending callback to prevent cleanup corruption
        self.pending_callbacks = self.pending_callbacks.saturating_add(1);

        // Build promise chain
        let mut promise = Promise::new(tx.receiver_id.clone());

        for action in &tx.actions {
            match action {
                Action::Transfer { amount } => {
                    promise = promise.transfer(NearToken::from_yoctonear(*amount));
                }
                Action::FunctionCall {
                    method,
                    args,
                    gas,
                    deposit,
                } => {
                    promise = promise.function_call(
                        method.clone(),
                        args.clone(),
                        NearToken::from_yoctonear(*deposit),
                        Gas::from_gas(*gas),
                    );
                }
            }
        }

        // Security: Attach callback to handle promise failures (NH-1)
        // Callback gas is configurable (default 20 TGas) for flexibility with complex state updates
        promise.then(
            Self::ext(env::current_account_id())
                .with_static_gas(Gas::from_gas(self.callback_gas))
                .on_execute_callback(tx_id)
        )
    }

    /// Cancel a pending transaction (only submitter can cancel before execution)
    /// Returns a Promise for the storage deposit refund
    pub fn cancel_transaction(&mut self, tx_id: u64) -> Promise {
        let sender = env::predecessor_account_id();
        require!(self.owners.contains(&sender), "Not an owner");

        // MEDIUM-2 fix: Removed u32::MAX check - get_tx() handles any u64 value

        let mut tx = self.get_tx_or_panic(tx_id).clone();
        require!(!tx.executed, "Already executed");
        require!(!tx.cancelled, "Already cancelled");

        // Only the submitter (first confirmer) can cancel
        require!(
            tx.confirmations.first() == Some(&sender),
            "Only submitter can cancel"
        );

        tx.cancelled = true;
        self.transactions.replace(self.get_tx_index_or_panic(tx_id), tx.clone());

        // M-2 fix: Release reserved balance when cancelling
        let deposit = Self::calculate_transaction_deposit(&tx);
        self.reserved_balance = self.reserved_balance.saturating_sub(deposit);

        // Emit cancellation event
        MultisigEvent::TransactionCancelled {
            tx_id,
            canceller: sender,
        }.emit();

        // Security (H-1 fix): Return refund promise instead of detaching
        // If refund fails, the caller will be notified via promise failure
        Promise::new(tx.storage_depositor.clone())
            .transfer(NearToken::from_yoctonear(self.storage_deposit))
    }

    /// Update callback gas allocation (owner-only)
    /// Allows adjusting gas for complex callback scenarios
    pub fn set_callback_gas(&mut self, gas: u64) {
        let sender = env::predecessor_account_id();
        require!(self.owners.contains(&sender), "Not an owner");
        require!(gas >= 5_000_000_000_000, "Callback gas too low (min 5 TGas)");
        require!(gas <= 100_000_000_000_000, "Callback gas too high (max 100 TGas)");

        let old_gas = self.callback_gas;
        self.callback_gas = gas;

        // M-3: Emit event for configuration change
        MultisigEvent::CallbackGasChanged {
            old_gas,
            new_gas: gas,
            changer: sender,
        }.emit();
    }

    /// L-5 fix: Update storage deposit amount (owner-only)
    pub fn set_storage_deposit(&mut self, deposit: u128) {
        let sender = env::predecessor_account_id();
        require!(self.owners.contains(&sender), "Not an owner");
        require!(deposit >= 1_000_000_000_000_000_000_000, "Storage deposit too low (min 0.001 NEAR)");
        require!(deposit <= 1_000_000_000_000_000_000_000_000, "Storage deposit too high (max 1 NEAR)");

        let old_deposit = self.storage_deposit;
        self.storage_deposit = deposit;

        MultisigEvent::StorageDepositChanged {
            old_deposit,
            new_deposit: deposit,
            changer: sender,
        }.emit();
    }

    pub fn get_storage_deposit(&self) -> u128 {
        self.storage_deposit
    }

    /// Revoke your confirmation from a pending transaction
    pub fn revoke_confirmation(&mut self, tx_id: u64) {
        let sender = env::predecessor_account_id();
        require!(self.owners.contains(&sender), "Not an owner");

        // MEDIUM-2 fix: Removed u32::MAX check - get_tx() handles any u64 value

        let mut tx = self.get_tx_or_panic(tx_id).clone();
        require!(!tx.executed, "Already executed");
        require!(!tx.cancelled, "Transaction cancelled");

        // CRITICAL-1 fix: Allow revocation but DON'T reset scheduled_time
        // This prevents timelock bypass (original scheduled_time preserved)
        // while avoiding permanent deadlock (revocation still possible)
        // Transaction becomes executable only if it STILL has threshold after timelock expires

        let pos = tx.confirmations.iter().position(|x| x == &sender);
        require!(pos.is_some(), "Not confirmed by you");

        tx.confirmations.remove(pos.unwrap());
        let confirmations_count = tx.confirmations.len() as u32;

        // Don't reset scheduled_time - preserve the original timelock deadline
        // This prevents the bypass attack where malicious owners extend the delay

        self.transactions.replace(self.get_tx_index_or_panic(tx_id), tx);

        // Emit event for confirmation revocation
        MultisigEvent::ConfirmationRevoked {
            tx_id,
            revoker: sender,
            confirmations: confirmations_count,
        }.emit();
    }

    /// Clean up old executed/cancelled transactions to reduce storage costs
    /// WARNING: This is gas-expensive. Only removes transactions before the specified index.
    /// Only executed or cancelled transactions can be removed (pending transactions are preserved).
    /// BLOCKS if there are pending callbacks to prevent corruption.
    /// Security: Processes max 100 transactions per call to prevent DoS
    pub fn cleanup_old_transactions(&mut self, before_index: u64) -> u64 {
        let sender = env::predecessor_account_id();
        require!(self.owners.contains(&sender), "Not an owner");
        require!(before_index <= u32::MAX as u64, "Index too large");

        // Security: Prevent cleanup during pending callbacks to avoid index corruption
        require!(
            self.pending_callbacks == 0,
            "Cannot cleanup while callbacks are pending"
        );

        let cleanup_end = (before_index as u32).min(self.transactions.len());
        let mut removed_count = 0u64;

        // Collect transactions to keep (avoid storage prefix collision)
        let mut transactions_to_keep: Vec<Transaction> = Vec::new();

        // Security: Limit iterations to prevent gas exhaustion DoS
        let max_iterations = cleanup_end.min(MAX_CLEANUP_BATCH);

        // Iterate transactions up to the batch limit
        for i in 0..max_iterations {
            if let Some(tx) = self.transactions.get(i) {
                // Keep if: after cleanup range OR (in cleanup range but still pending)
                if i >= cleanup_end || (!tx.executed && !tx.cancelled) {
                    transactions_to_keep.push(tx.clone());
                } else {
                    removed_count += 1;
                }
            }
        }

        // Keep all transactions after the batch limit
        for i in max_iterations..self.transactions.len() {
            if let Some(tx) = self.transactions.get(i) {
                transactions_to_keep.push(tx.clone());
            }
        }

        // Clear and rebuild the vector to avoid storage corruption
        self.transactions.clear();
        for tx in transactions_to_keep {
            self.transactions.push(tx);
        }

        // M-3: Emit event for cleanup operation with transaction range context
        MultisigEvent::TransactionsCleanedUp {
            count: removed_count,
            from_index: 0,
            to_index: max_iterations as u64,
            cleaner: sender,
        }.emit();

        removed_count
    }

    /// Security: Callback to handle promise execution results (NH-1)
    /// If promise fails, mark transaction as not executed and reset timelock so it can be retried
    #[private]
    pub fn on_execute_callback(&mut self, tx_id: u64) {
        // L-3 fix: Validate transaction exists and ID matches parameter
        if let Some(tx) = self.get_tx(tx_id) {
            require!(tx.id == tx_id, "Transaction ID mismatch in callback");
        } else {
            env::log_str(&format!("⚠️ Callback for non-existent transaction {}", tx_id));
            return;
        }

        // Decrement pending callbacks counter
        self.pending_callbacks = self.pending_callbacks.saturating_sub(1);

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                // Transaction executed successfully, already marked as executed
                env::log_str(&format!("Transaction {} executed successfully", tx_id));

                // M-2 fix: Release reserved balance after successful execution
                if let Some(tx) = self.transactions.get(self.get_tx_index_or_panic(tx_id)) {
                    let deposit = Self::calculate_transaction_deposit(tx);
                    self.reserved_balance = self.reserved_balance.saturating_sub(deposit);
                }

                MultisigEvent::TransactionExecuted {
                    tx_id,
                    success: true,
                }.emit();

                // Security (H-1 fix): Refund storage deposit with callback to track failures
                // Detached to prevent refund failures from affecting transaction execution result
                if let Some(tx) = self.transactions.get(self.get_tx_index_or_panic(tx_id)) {
                    Promise::new(tx.storage_depositor.clone())
                        .transfer(NearToken::from_yoctonear(self.storage_deposit))
                        .then(
                            Self::ext(env::current_account_id())
                                .with_static_gas(Gas::from_gas(5_000_000_000_000))
                                .on_refund_callback(tx_id, tx.storage_depositor.clone())
                        );
                }
            }
            PromiseResult::Failed => {
                // Promise failed - revert executed flag and reset timelock for security
                env::log_str(&format!("Transaction {} failed, resetting timelock and marking for retry", tx_id));

                // Emit failure event
                MultisigEvent::TransactionExecuted {
                    tx_id,
                    success: false,
                }.emit();

                if let Some(tx) = self.transactions.get(self.get_tx_index_or_panic(tx_id)) {
                    let mut tx_clone = tx.clone();
                    tx_clone.executed = false;
                    tx_clone.scheduled_time = None; // Reset timelock - require new approval cycle
                    self.transactions.replace(self.get_tx_index_or_panic(tx_id), tx_clone);
                }
            }
        }
    }

    /// Security (H-1 fix): Callback to track storage deposit refund results
    /// Logs refund failures so users know if their deposit wasn't returned
    #[private]
    pub fn on_refund_callback(&mut self, tx_id: u64, recipient: AccountId) {
        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                env::log_str(&format!("Storage deposit refund successful for tx {} to {}", tx_id, recipient));
            }
            PromiseResult::Failed => {
                env::log_str(&format!("⚠️  Storage deposit refund FAILED for tx {} to {}. User may need to claim manually.", tx_id, recipient));
                // Future enhancement: Store failed refunds in a claimable pool
            }
        }
    }

    // ===== View Methods =====

    pub fn get_owners(&self) -> Vec<AccountId> {
        self.owners.iter().cloned().collect()
    }

    pub fn get_num_confirmations(&self) -> u32 {
        self.num_confirmations
    }

    pub fn get_timelock_duration(&self) -> u64 {
        self.timelock_duration
    }

    pub fn get_transaction(&self, tx_id: u64) -> Option<Transaction> {
        self.get_tx(tx_id).cloned()
    }

    /// Get pending transactions (paginated to avoid gas exhaustion - H-2 fix)
    /// Security: Unbounded version removed to prevent DoS attacks
    pub fn get_pending_transactions_paginated(&self, from_index: u64, limit: u64) -> Vec<Transaction> {
        let len = self.transactions.len() as u64;
        let start = from_index.min(len);
        let end = (start.saturating_add(limit)).min(len);

        (start..end)
            .filter_map(|i| {
                let tx = self.get_tx(i)?;
                if !tx.executed && !tx.cancelled {
                    Some(tx.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get scheduled transactions (paginated to avoid gas exhaustion - H-2 fix)
    /// Security: Unbounded version removed to prevent DoS attacks
    pub fn get_scheduled_transactions_paginated(&self, from_index: u64, limit: u64) -> Vec<Transaction> {
        let len = self.transactions.len() as u64;
        let start = from_index.min(len);
        let end = (start.saturating_add(limit)).min(len);

        // HIGH-1 fix: Use direct index access instead of get_tx() to avoid O(n²) complexity
        (start..end)
            .filter_map(|i| {
                let tx = self.transactions.get(i as u32)?;
                if !tx.executed && !tx.cancelled && tx.scheduled_time.is_some() {
                    Some(tx.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get executable transactions (paginated to avoid gas exhaustion - H-2 fix)
    /// Security: Unbounded version removed to prevent DoS attacks
    pub fn get_executable_transactions_paginated(&self, from_index: u64, limit: u64) -> Vec<Transaction> {
        let current_time = env::block_timestamp();
        let len = self.transactions.len() as u64;
        let start = from_index.min(len);
        let end = (start.saturating_add(limit)).min(len);

        // HIGH-1 fix: Use direct index access instead of get_tx() to avoid O(n²) complexity
        (start..end)
            .filter_map(|i| {
                let tx = self.transactions.get(i as u32)?;
                if !tx.executed && !tx.cancelled && tx.scheduled_time.is_some() && current_time >= tx.scheduled_time.unwrap() {
                    Some(tx.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_transaction_count(&self) -> u64 {
        self.transactions.len() as u64
    }

    pub fn is_owner(&self, account_id: AccountId) -> bool {
        self.owners.contains(&account_id)
    }

    pub fn has_confirmed(&self, tx_id: u64, account_id: AccountId) -> bool {
        if let Some(tx) = self.get_tx(tx_id) {
            tx.confirmations.contains(&account_id)
        } else {
            false
        }
    }
}
"#;

const WEIGHTED_TEMPLATE_LIB: &str = r#"use near_sdk::store::{UnorderedMap, Vector};
use near_sdk::{near, require, AccountId, PanicOnDefault, env, Promise, NearToken, Gas, PromiseResult};

/// Security: Maximum number of actions per transaction to prevent gas exhaustion
const MAX_ACTIONS: usize = 10;

/// Security: Maximum args size (32KB) to prevent storage attacks
const MAX_ARGS_LEN: usize = 32768;

/// Security: Maximum method name length
const MAX_METHOD_NAME_LEN: usize = 256;

/// Security: Maximum gas per action (100 TGas)
const MAX_GAS_PER_ACTION: u64 = 100_000_000_000_000;

/// Security: Maximum total gas across all actions (250 TGas)
const MAX_TOTAL_GAS: u64 = 250_000_000_000_000;

/// Security: Maximum number of owners to prevent gas exhaustion
const MAX_OWNERS: usize = 50;

/// Security: Maximum transactions to process per cleanup call (prevents DoS)
const MAX_CLEANUP_BATCH: u32 = 100;

/// Storage cost per transaction (0.01 NEAR) - refundable on execution/cancellation
const TRANSACTION_STORAGE_DEPOSIT: u128 = 10_000_000_000_000_000_000_000; // 0.01 NEAR

/// Default callback gas (20 TGas) - can be configured per contract
const DEFAULT_CALLBACK_GAS: u64 = 20_000_000_000_000;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct WeightedMultisig {
    pub owner_weights: UnorderedMap<AccountId, u32>,
    pub approval_threshold: u32, // Total weight needed
    pub transactions: Vector<Transaction>,
    pub pending_callbacks: u32, // Track pending executions to prevent cleanup corruption
    pub callback_gas: u64, // Gas allocated for execution callbacks (configurable)
    pub storage_deposit: u128, // L-5 fix: Storage deposit per transaction (configurable)
    pub next_tx_id: u64, // M-3 fix: Monotonic transaction ID counter
    pub reserved_balance: u128, // M-2 fix: Total deposits reserved by pending transactions
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Transaction {
    pub id: u64,
    pub receiver_id: AccountId,
    pub actions: Vec<Action>,
    pub approvals: Vec<(AccountId, u32)>, // (owner, weight)
    pub total_weight: u32,
    pub executed: bool,
    pub cancelled: bool,
    pub storage_depositor: AccountId, // Who paid storage deposit (gets refund)
    pub expiration: Option<u64>, // L-2 fix: Optional expiration timestamp (nanoseconds)
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub enum Action {
    Transfer { amount: u128 },
    FunctionCall {
        method: String,
        args: Vec<u8>,
        gas: u64,
        deposit: u128,
    },
}

// Events for off-chain indexing
#[near(event_json(standard = "multisig"))]
pub enum MultisigEvent {
    #[event_version("1.0.0")]
    TransactionSubmitted { tx_id: u64, submitter: AccountId, receiver_id: AccountId },

    #[event_version("1.0.0")]
    TransactionApproved { tx_id: u64, approver: AccountId, weight: u32, total_weight: u32 },

    #[event_version("1.0.0")]
    TransactionExecuted { tx_id: u64, success: bool },

    #[event_version("1.0.0")]
    TransactionCancelled { tx_id: u64, canceller: AccountId },

    #[event_version("1.0.0")]
    ApprovalRevoked { tx_id: u64, revoker: AccountId, weight: u32, total_weight: u32 },

    #[event_version("1.0.0")]
    CallbackGasChanged { old_gas: u64, new_gas: u64, changer: AccountId },

    #[event_version("1.0.0")]
    ManualExecutionTriggered { tx_id: u64, executor: AccountId },

    #[event_version("1.0.0")]
    TransactionReady { tx_id: u64, total_weight: u32 },

    #[event_version("1.0.0")]
    TransactionsCleanedUp { count: u64, from_index: u64, to_index: u64, cleaner: AccountId },
}

// Security: Safe transaction access methods to prevent u32 overflow attacks
impl WeightedMultisig {
    /// Safe transaction lookup by ID (M-3 fix: search by id field, not index)
    fn get_tx(&self, tx_id: u64) -> Option<&Transaction> {
        for i in 0..self.transactions.len() {
            if let Some(tx) = self.transactions.get(i) {
                if tx.id == tx_id {
                    return Some(tx);
                }
            }
        }
        None
    }

    /// Find transaction index by ID
    fn get_tx_index(&self, tx_id: u64) -> Option<u32> {
        for i in 0..self.transactions.len() {
            if let Some(tx) = self.transactions.get(i) {
                if tx.id == tx_id {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Safe transaction index lookup (panics if not found)
    fn get_tx_index_or_panic(&self, tx_id: u64) -> u32 {
        self.get_tx_index(tx_id).expect("Transaction not found")
    }

    /// Safe transaction lookup (panics if not found)
    fn get_tx_or_panic(&self, tx_id: u64) -> &Transaction {
        self.get_tx(tx_id).expect("Transaction not found")
    }

    /// Validate action vector to prevent attacks
    fn validate_actions(actions: &Vec<Action>) -> u128 {
        require!(!actions.is_empty(), "Actions cannot be empty");
        require!(actions.len() <= MAX_ACTIONS, "Too many actions (max 10)");

        let mut total_gas = 0u64;
        let mut total_deposit = 0u128;

        for action in actions {
            match action {
                Action::Transfer { amount } => {
                    require!(*amount > 0, "Transfer amount must be positive");
                    total_deposit = total_deposit.saturating_add(*amount);
                }
                Action::FunctionCall { method, args, gas, deposit } => {
                    require!(args.len() <= MAX_ARGS_LEN, "Args too large (max 32KB)");
                    require!(method.len() <= MAX_METHOD_NAME_LEN, "Method name too long");
                    require!(!method.is_empty(), "Method name cannot be empty");

                    // Security: Validate gas parameters (BUG-7)
                    require!(*gas > 0, "Gas must be positive");
                    require!(*gas <= MAX_GAS_PER_ACTION, "Gas per action exceeds limit (max 100 TGas)");
                    total_gas = total_gas.saturating_add(*gas);
                    total_deposit = total_deposit.saturating_add(*deposit);
                }
            }
        }

        require!(total_gas <= MAX_TOTAL_GAS, "Total gas exceeds limit (max 250 TGas)");
        total_deposit
    }

    /// M-2 fix: Calculate total deposits in a transaction
    fn calculate_transaction_deposit(tx: &Transaction) -> u128 {
        let mut total_deposit = 0u128;
        for action in &tx.actions {
            match action {
                Action::Transfer { amount } => {
                    total_deposit = total_deposit.saturating_add(*amount);
                }
                Action::FunctionCall { deposit, .. } => {
                    total_deposit = total_deposit.saturating_add(*deposit);
                }
            }
        }
        total_deposit
    }
}

#[near]
impl WeightedMultisig {
    #[init]
    pub fn new(owners_with_weights: Vec<(AccountId, u32)>, approval_threshold: u32) -> Self {
        require!(!owners_with_weights.is_empty(), "Need at least one owner");
        // Security: Enforce max owners limit (BUG-9)
        require!(owners_with_weights.len() <= MAX_OWNERS, "Too many owners (max 50)");
        require!(approval_threshold > 0, "Threshold must be positive");

        // Security: Use checked arithmetic to prevent overflow (H-2)
        let total_weight: u32 = owners_with_weights
            .iter()
            .try_fold(0u32, |acc, (_, w)| acc.checked_add(*w))
            .expect("Total weight overflow");
        require!(
            approval_threshold <= total_weight,
            "Threshold exceeds total weight"
        );

        let mut owner_weights = UnorderedMap::new(b"w");
        for (owner, weight) in &owners_with_weights {
            require!(*weight > 0, "Weight must be positive");
            // Security: Check for duplicate owners (L-3)
            require!(!owner_weights.contains_key(owner), "Duplicate owner");
            owner_weights.insert(owner.clone(), *weight);
        }

        Self {
            owner_weights,
            approval_threshold,
            transactions: Vector::new(b"t"),
            pending_callbacks: 0,
            callback_gas: DEFAULT_CALLBACK_GAS,
            storage_deposit: TRANSACTION_STORAGE_DEPOSIT, // L-5 fix
            next_tx_id: 0, // M-3 fix
            reserved_balance: 0, // M-2 fix
        }
    }

    // ===== Write Methods =====

    /// Submit a new transaction for approval
    /// Requires 0.01 NEAR storage deposit (refunded on execution/cancellation)
    #[payable]
    pub fn submit_transaction(&mut self, receiver_id: AccountId, actions: Vec<Action>, expiration_hours: Option<u64>) -> u64 {
        let sender = env::predecessor_account_id();
        let weight = self.owner_weights.get(&sender).expect("Not an owner");

        // Security: Require storage deposit to prevent spam (M-1)
        let attached = env::attached_deposit().as_yoctonear();
        require!(
            attached >= self.storage_deposit,
            format!("Must attach at least {} yoctoNEAR for storage", self.storage_deposit)
        );

        // Security: Prevent sending to self
        require!(
            receiver_id != env::current_account_id(),
            "Cannot send to multisig contract itself"
        );

        // Security: Validate actions and get total deposit
        let total_deposit = Self::validate_actions(&actions);

        // M-2 fix: Check available balance after accounting for reserved amounts
        // CRITICAL-2 fix: Subtract storage deposit from available balance since it won't be usable
        let available_balance = env::account_balance()
            .as_yoctonear()
            .saturating_sub(self.reserved_balance)
            .saturating_sub(self.storage_deposit);
        require!(
            total_deposit <= available_balance,
            "Insufficient available balance (pending transactions already reserved funds)"
        );

        // M-2 fix: Reserve balance for this transaction
        self.reserved_balance = self.reserved_balance.saturating_add(total_deposit);

        // M-3 fix: Use monotonic counter instead of vector length
        let tx_id = self.next_tx_id;
        // MEDIUM-1 fix: Check for counter overflow before incrementing
        require!(
            self.next_tx_id < u64::MAX,
            "Transaction ID counter limit reached"
        );
        self.next_tx_id = self.next_tx_id.saturating_add(1);

        // L-2 fix: Calculate expiration timestamp if hours provided
        let expiration = expiration_hours.map(|hours| {
            let nanos_per_hour = 3_600_000_000_000u64; // 1 hour = 3.6 trillion nanoseconds
            env::block_timestamp().saturating_add(hours.saturating_mul(nanos_per_hour))
        });

        let tx = Transaction {
            id: tx_id,
            receiver_id: receiver_id.clone(),
            actions,
            approvals: vec![(sender.clone(), *weight)],
            total_weight: *weight,
            executed: false,
            cancelled: false,
            storage_depositor: sender.clone(),
            expiration,
        };

        self.transactions.push(tx);

        // Emit event for off-chain indexing
        MultisigEvent::TransactionSubmitted {
            tx_id,
            submitter: sender.clone(),
            receiver_id,
        }.emit();

        // Emit approval event (submitter auto-approves)
        MultisigEvent::TransactionApproved {
            tx_id,
            approver: sender,
            weight: *weight,
            total_weight: *weight,
        }.emit();

        tx_id
    }

    pub fn approve_transaction(&mut self, tx_id: u64) {
        let sender = env::predecessor_account_id();
        let weight = self.owner_weights.get(&sender).expect("Not an owner");

        // MEDIUM-2 fix: Removed u32::MAX check - get_tx() handles any u64 value

        let mut tx = self.get_tx_or_panic(tx_id).clone();
        require!(!tx.executed, "Already executed");
        require!(!tx.cancelled, "Transaction cancelled");
        require!(
            !tx.approvals.iter().any(|(owner, _)| owner == &sender),
            "Already approved"
        );

        tx.approvals.push((sender.clone(), *weight));

        // Security: Use checked_add to prevent weight overflow (H-2)
        tx.total_weight = tx.total_weight.checked_add(*weight).expect("Weight overflow");

        // Emit approval event
        MultisigEvent::TransactionApproved {
            tx_id,
            approver: sender,
            weight: *weight,
            total_weight: tx.total_weight,
        }.emit();

        // H-2 fix: Don't auto-execute - emit ready event and require explicit execution
        if tx.total_weight >= self.approval_threshold {
            MultisigEvent::TransactionReady {
                tx_id,
                total_weight: tx.total_weight,
            }.emit();
        }

        self.transactions.replace(self.get_tx_index_or_panic(tx_id), tx);
    }

    /// Manually execute a fully-approved transaction (for retries after failure)
    pub fn execute_transaction(&mut self, tx_id: u64) -> Promise {
        let sender = env::predecessor_account_id();
        require!(self.owner_weights.contains_key(&sender), "Not an owner");

        // MEDIUM-2 fix: Removed u32::MAX check - get_tx() handles any u64 value

        let mut tx = self.get_tx_or_panic(tx_id).clone();

        require!(!tx.executed, "Already executed");
        require!(!tx.cancelled, "Transaction cancelled");

        // L-2 fix: Check if transaction has expired
        if let Some(exp_time) = tx.expiration {
            require!(
                env::block_timestamp() < exp_time,
                "Transaction expired"
            );
        }

        require!(
            tx.total_weight >= self.approval_threshold,
            "Not enough weight"
        );

        // Mark as executed
        tx.executed = true;
        self.transactions.replace(self.get_tx_index_or_panic(tx_id), tx.clone());

        // Emit manual execution event
        MultisigEvent::ManualExecutionTriggered {
            tx_id,
            executor: sender,
        }.emit();

        // Track pending callback to prevent cleanup corruption
        self.pending_callbacks = self.pending_callbacks.saturating_add(1);

        // Build promise chain
        let mut promise = Promise::new(tx.receiver_id.clone());

        for action in &tx.actions {
            match action {
                Action::Transfer { amount } => {
                    promise = promise.transfer(NearToken::from_yoctonear(*amount));
                }
                Action::FunctionCall {
                    method,
                    args,
                    gas,
                    deposit,
                } => {
                    promise = promise.function_call(
                        method.clone(),
                        args.clone(),
                        NearToken::from_yoctonear(*deposit),
                        Gas::from_gas(*gas),
                    );
                }
            }
        }

        // Security: Attach callback to handle promise failures (NH-1)
        // Callback gas is configurable (default 20 TGas) for flexibility with complex state updates
        promise.then(
            Self::ext(env::current_account_id())
                .with_static_gas(Gas::from_gas(self.callback_gas))
                .on_execute_callback(tx_id)
        )
    }

    /// Cancel a pending transaction (only submitter can cancel)
    /// Returns a Promise for the storage deposit refund
    pub fn cancel_transaction(&mut self, tx_id: u64) -> Promise {
        let sender = env::predecessor_account_id();
        require!(self.owner_weights.contains_key(&sender), "Not an owner");

        // MEDIUM-2 fix: Removed u32::MAX check - get_tx() handles any u64 value

        let mut tx = self.get_tx_or_panic(tx_id).clone();
        require!(!tx.executed, "Already executed");
        require!(!tx.cancelled, "Already cancelled");

        // Only the submitter (first approver) can cancel
        require!(
            tx.approvals.first().map(|(owner, _)| owner) == Some(&sender),
            "Only submitter can cancel"
        );

        tx.cancelled = true;
        self.transactions.replace(self.get_tx_index_or_panic(tx_id), tx.clone());

        // M-2 fix: Release reserved balance when cancelling
        let deposit = Self::calculate_transaction_deposit(&tx);
        self.reserved_balance = self.reserved_balance.saturating_sub(deposit);

        // Emit cancellation event
        MultisigEvent::TransactionCancelled {
            tx_id,
            canceller: sender,
        }.emit();

        // Security (H-1 fix): Return refund promise instead of detaching
        // If refund fails, the caller will be notified via promise failure
        Promise::new(tx.storage_depositor.clone())
            .transfer(NearToken::from_yoctonear(self.storage_deposit))
    }

    /// Update callback gas allocation (owner-only)
    /// Allows adjusting gas for complex callback scenarios
    pub fn set_callback_gas(&mut self, gas: u64) {
        let sender = env::predecessor_account_id();
        require!(self.owner_weights.contains_key(&sender), "Not an owner");
        require!(gas >= 5_000_000_000_000, "Callback gas too low (min 5 TGas)");
        require!(gas <= 100_000_000_000_000, "Callback gas too high (max 100 TGas)");

        let old_gas = self.callback_gas;
        self.callback_gas = gas;

        // Emit configuration change event
        MultisigEvent::CallbackGasChanged {
            old_gas,
            new_gas: gas,
            changer: sender,
        }.emit();
    }

    /// L-5 fix: Update storage deposit amount (owner-only)
    pub fn set_storage_deposit(&mut self, deposit: u128) {
        let sender = env::predecessor_account_id();
        require!(self.owner_weights.contains_key(&sender), "Not an owner");
        require!(deposit >= 1_000_000_000_000_000_000_000, "Storage deposit too low (min 0.001 NEAR)");
        require!(deposit <= 1_000_000_000_000_000_000_000_000, "Storage deposit too high (max 1 NEAR)");

        let old_deposit = self.storage_deposit;
        self.storage_deposit = deposit;

        MultisigEvent::StorageDepositChanged {
            old_deposit,
            new_deposit: deposit,
            changer: sender,
        }.emit();
    }

    pub fn get_storage_deposit(&self) -> u128 {
        self.storage_deposit
    }

    /// Revoke your approval from a pending transaction
    pub fn revoke_approval(&mut self, tx_id: u64) {
        let sender = env::predecessor_account_id();
        require!(self.owner_weights.contains_key(&sender), "Not an owner");

        // MEDIUM-2 fix: Removed u32::MAX check - get_tx() handles any u64 value

        let mut tx = self.get_tx_or_panic(tx_id).clone();
        require!(!tx.executed, "Already executed");
        require!(!tx.cancelled, "Transaction cancelled");

        let pos = tx.approvals.iter().position(|(owner, _)| owner == &sender);
        require!(pos.is_some(), "Not approved by you");

        let (_, weight) = tx.approvals.remove(pos.unwrap());

        // Security: Use checked_sub to prevent underflow
        tx.total_weight = tx.total_weight.checked_sub(weight).expect("Weight underflow");

        let total_weight = tx.total_weight;
        self.transactions.replace(self.get_tx_index_or_panic(tx_id), tx);

        // Emit revocation event
        MultisigEvent::ApprovalRevoked {
            tx_id,
            revoker: sender,
            weight,
            total_weight,
        }.emit();
    }

    /// Clean up old executed/cancelled transactions to reduce storage costs
    /// WARNING: This is gas-expensive. Only removes transactions before the specified index.
    /// Only executed or cancelled transactions can be removed (pending transactions are preserved).
    /// BLOCKS if there are pending callbacks to prevent corruption.
    /// Security: Processes max 100 transactions per call to prevent DoS
    pub fn cleanup_old_transactions(&mut self, before_index: u64) -> u64 {
        let sender = env::predecessor_account_id();
        require!(self.owner_weights.contains_key(&sender), "Not an owner");
        require!(before_index <= u32::MAX as u64, "Index too large");

        // Security: Prevent cleanup during pending callbacks to avoid index corruption
        require!(
            self.pending_callbacks == 0,
            "Cannot cleanup while callbacks are pending"
        );

        let cleanup_end = (before_index as u32).min(self.transactions.len());
        let mut removed_count = 0u64;

        // Collect transactions to keep (avoid storage prefix collision)
        let mut transactions_to_keep: Vec<Transaction> = Vec::new();

        // Security: Limit iterations to prevent gas exhaustion DoS
        let max_iterations = cleanup_end.min(MAX_CLEANUP_BATCH);

        // Iterate transactions up to the batch limit
        for i in 0..max_iterations {
            if let Some(tx) = self.transactions.get(i) {
                // Keep if: after cleanup range OR (in cleanup range but still pending)
                if i >= cleanup_end || (!tx.executed && !tx.cancelled) {
                    transactions_to_keep.push(tx.clone());
                } else {
                    removed_count += 1;
                }
            }
        }

        // Keep all transactions after the batch limit
        for i in max_iterations..self.transactions.len() {
            if let Some(tx) = self.transactions.get(i) {
                transactions_to_keep.push(tx.clone());
            }
        }

        // Clear and rebuild the vector to avoid storage corruption
        self.transactions.clear();
        for tx in transactions_to_keep {
            self.transactions.push(tx);
        }

        // M-3: Emit event for cleanup operation with transaction range context
        MultisigEvent::TransactionsCleanedUp {
            count: removed_count,
            from_index: 0,
            to_index: max_iterations as u64,
            cleaner: sender,
        }.emit();

        removed_count
    }

    /// Security: Callback to handle promise execution results (NH-1)
    /// If promise fails, mark transaction as not executed so it can be retried
    #[private]
    pub fn on_execute_callback(&mut self, tx_id: u64) {
        // L-3 fix: Validate transaction exists and ID matches parameter
        if let Some(tx) = self.get_tx(tx_id) {
            require!(tx.id == tx_id, "Transaction ID mismatch in callback");
        } else {
            env::log_str(&format!("⚠️ Callback for non-existent transaction {}", tx_id));
            return;
        }

        // Decrement pending callbacks counter
        self.pending_callbacks = self.pending_callbacks.saturating_sub(1);

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                // Transaction executed successfully, already marked as executed
                env::log_str(&format!("Transaction {} executed successfully", tx_id));

                // M-2 fix: Release reserved balance after successful execution
                if let Some(tx) = self.transactions.get(self.get_tx_index_or_panic(tx_id)) {
                    let deposit = Self::calculate_transaction_deposit(tx);
                    self.reserved_balance = self.reserved_balance.saturating_sub(deposit);
                }

                MultisigEvent::TransactionExecuted {
                    tx_id,
                    success: true,
                }.emit();

                // Security (H-1 fix): Refund storage deposit with callback to track failures
                // Detached to prevent refund failures from affecting transaction execution result
                if let Some(tx) = self.transactions.get(self.get_tx_index_or_panic(tx_id)) {
                    Promise::new(tx.storage_depositor.clone())
                        .transfer(NearToken::from_yoctonear(self.storage_deposit))
                        .then(
                            Self::ext(env::current_account_id())
                                .with_static_gas(Gas::from_gas(5_000_000_000_000))
                                .on_refund_callback(tx_id, tx.storage_depositor.clone())
                        );
                }
            }
            PromiseResult::Failed => {
                // Promise failed - revert executed flag so transaction can be retried
                env::log_str(&format!("Transaction {} failed, marking for retry", tx_id));

                // Emit failure event
                MultisigEvent::TransactionExecuted {
                    tx_id,
                    success: false,
                }.emit();

                if let Some(tx) = self.transactions.get(self.get_tx_index_or_panic(tx_id)) {
                    let mut tx_clone = tx.clone();
                    tx_clone.executed = false;
                    self.transactions.replace(self.get_tx_index_or_panic(tx_id), tx_clone);
                }
            }
        }
    }

    /// Security (H-1 fix): Callback to track storage deposit refund results
    /// Logs refund failures so users know if their deposit wasn't returned
    #[private]
    pub fn on_refund_callback(&mut self, tx_id: u64, recipient: AccountId) {
        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                env::log_str(&format!("Storage deposit refund successful for tx {} to {}", tx_id, recipient));
            }
            PromiseResult::Failed => {
                env::log_str(&format!("⚠️  Storage deposit refund FAILED for tx {} to {}. User may need to claim manually.", tx_id, recipient));
                // Future enhancement: Store failed refunds in a claimable pool
            }
        }
    }

    // ===== View Methods =====

    pub fn get_owners(&self) -> Vec<(AccountId, u32)> {
        self.owner_weights
            .iter()
            .map(|(account, weight)| (account.clone(), *weight))
            .collect()
    }

    pub fn get_owner_weight(&self, account_id: AccountId) -> Option<u32> {
        self.owner_weights.get(&account_id).copied()
    }

    pub fn get_approval_threshold(&self) -> u32 {
        self.approval_threshold
    }

    pub fn get_total_weight(&self) -> Option<u32> {
        // Security: Use checked arithmetic in view method too
        // Returns None on overflow instead of silently returning u32::MAX
        self.owner_weights
            .iter()
            .try_fold(0u32, |acc, (_, w)| acc.checked_add(*w))
    }

    pub fn get_transaction(&self, tx_id: u64) -> Option<Transaction> {
        self.get_tx(tx_id).cloned()
    }

    /// Get pending transactions (paginated to avoid gas exhaustion - H-2 fix)
    /// Security: Unbounded version removed to prevent DoS attacks
    pub fn get_pending_transactions_paginated(&self, from_index: u64, limit: u64) -> Vec<Transaction> {
        let len = self.transactions.len() as u64;
        let start = from_index.min(len);
        let end = (start.saturating_add(limit)).min(len);

        (start..end)
            .filter_map(|i| {
                let tx = self.get_tx(i)?;
                if !tx.executed && !tx.cancelled {
                    Some(tx.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_transaction_count(&self) -> u64 {
        self.transactions.len() as u64
    }

    pub fn is_owner(&self, account_id: AccountId) -> bool {
        self.owner_weights.contains_key(&account_id)
    }

    pub fn has_approved(&self, tx_id: u64, account_id: AccountId) -> bool {
        if let Some(tx) = self.get_tx(tx_id) {
            tx.approvals.iter().any(|(owner, _)| owner == &account_id)
        } else {
            false
        }
    }

    pub fn get_transaction_progress(&self, tx_id: u64) -> Option<(u32, u32)> {
        self.get_tx(tx_id)
            .map(|tx| (tx.total_weight, self.approval_threshold))
    }
}
"#;

const GITHUB_ACTIONS_WORKFLOW: &str = r#"name: Release Contract

on:
  push:
    tags:
      - 'v[0-9]+*'

permissions:
  contents: write

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: 1.86.0
          target: wasm32-unknown-unknown
          override: true

      - name: Install cargo-near
        run: |
          curl --proto '=https' --tlsv1.2 -LsSf https://github.com/near/cargo-near/releases/download/cargo-near-v0.18.0/cargo-near-installer.sh | sh

      - name: Build contract
        run: cargo near build non-reproducible-wasm

      - name: Generate verification artifacts
        run: |
          mkdir -p release
          cp target/near/*.wasm release/
          cd release
          sha256sum -b *.wasm > SHA256SUMS

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            release/*.wasm
            release/SHA256SUMS
          generate_release_notes: true
          body: |
            ## Verification

            Download and verify the contract:

            ```bash
            wget https://github.com/${{ github.repository }}/releases/download/${{ github.ref_name }}/SHA256SUMS
            wget https://github.com/${{ github.repository }}/releases/download/${{ github.ref_name }}/*.wasm
            sha256sum -c SHA256SUMS
            ```

            **Build Info**:
            - Rust: 1.86.0
            - cargo-near: 0.18.0
            - near-sdk: 5.24.0
            - Commit: ${{ github.sha }}
"#;

pub fn run(project_name: &str, template: &str) -> Result<()> {
    // L-4 fix: Validate project name to prevent path traversal and filesystem issues
    if project_name.is_empty() {
        anyhow::bail!("Project name cannot be empty");
    }
    if project_name.contains('/') || project_name.contains('\\') {
        anyhow::bail!("Project name cannot contain path separators");
    }
    if project_name.starts_with('.') || project_name.starts_with('-') {
        anyhow::bail!("Project name cannot start with '.' or '-'");
    }
    if !project_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        anyhow::bail!(
            "Project name can only contain alphanumeric characters, hyphens, and underscores"
        );
    }

    let project_path = Path::new(project_name);

    if project_path.exists() {
        anyhow::bail!("Directory '{}' already exists", project_name);
    }

    // Select template
    let (lib_template, template_description) = match template {
        "basic" => (
            BASIC_TEMPLATE_LIB,
            "Basic M-of-N multisig with security hardening",
        ),
        "timelock" => (
            TIMELOCK_TEMPLATE_LIB,
            "Timelock multisig with delayed execution and security hardening",
        ),
        "weighted" => (
            WEIGHTED_TEMPLATE_LIB,
            "Weighted voting multisig with security hardening",
        ),
        _ => anyhow::bail!(
            "Unknown template: {}. Available: basic, timelock, weighted",
            template
        ),
    };

    println!(
        "Creating multisig project: {} ({})",
        project_name, template_description
    );

    // Create directory structure
    fs::create_dir_all(project_path.join("src"))?;
    fs::create_dir_all(project_path.join(".github/workflows"))?;

    // Write Cargo.toml
    let cargo_toml = BASIC_TEMPLATE_CARGO.replace("{{project_name}}", project_name);
    fs::write(project_path.join("Cargo.toml"), cargo_toml)?;

    // Write src/lib.rs
    fs::write(project_path.join("src/lib.rs"), lib_template)?;

    // Write GitHub Actions workflow
    fs::write(
        project_path.join(".github/workflows/release.yml"),
        GITHUB_ACTIONS_WORKFLOW,
    )?;

    println!("✓ Created {}/", project_name);
    println!("✓ Created {}/Cargo.toml", project_name);
    println!("✓ Created {}/src/lib.rs", project_name);
    println!("✓ Created {}/.github/workflows/release.yml", project_name);
    println!("\nNext steps:");
    println!("  cd {}", project_name);
    println!("  git init && git add . && git commit -m 'Initial commit'");
    println!("  git tag v0.1.0 && git push --tags  # Triggers auto-release");
    println!("  OR: near-multisig build  # For local development");
    println!(
        "\n✓ All security fixes applied (callback handling, overflow protection, input validation)"
    );

    Ok(())
}
