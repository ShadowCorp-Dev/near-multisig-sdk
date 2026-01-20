use crate::types::Action;
use near_sdk::require;

// ==================== Security Limits ====================
// These constants protect the contract from abuse and ensure reliable operation

/// Maximum number of actions per transaction
/// Prevents gas exhaustion from overly complex transactions
pub const MAX_ACTIONS: usize = 10;

/// Maximum size of function call arguments (32KB)
/// Prevents storage bloat and excessive data processing costs
pub const MAX_ARGS_LEN: usize = 32768;

/// Maximum length of method names
/// Prevents abuse while allowing reasonable naming conventions
pub const MAX_METHOD_NAME_LEN: usize = 256;

/// Maximum gas per individual action (100 TGas)
/// Ensures each action completes within reasonable bounds
pub const MAX_GAS_PER_ACTION: u64 = 100_000_000_000_000;

/// Maximum total gas across all actions in a transaction (250 TGas)
/// Prevents excessive total gas consumption
pub const MAX_TOTAL_GAS: u64 = 250_000_000_000_000;

/// Maximum number of multisig owners
/// Prevents iteration costs from becoming too expensive
pub const MAX_OWNERS: usize = 50;

/// Maximum transactions to clean up in a single call
/// Prevents cleanup operations from running out of gas
pub const MAX_CLEANUP_BATCH: u32 = 100;

/// Storage deposit required per transaction (0.01 NEAR)
/// Covers storage costs and prevents spam. Refunded when transaction completes.
pub const TRANSACTION_STORAGE_DEPOSIT: u128 = 10_000_000_000_000_000_000_000; // 0.01 NEAR

/// Default gas allocation for execution callbacks (20 TGas)
/// Can be adjusted by owners based on transaction complexity
pub const DEFAULT_CALLBACK_GAS: u64 = 20_000_000_000_000;

/// Minimum balance the contract must maintain (0.1 NEAR)
/// Ensures the contract can't be drained and remains operational
pub const MIN_CONTRACT_BALANCE: u128 = 100_000_000_000_000_000_000_000; // 0.1 NEAR

/// Maximum number of pending transactions
/// Prevents unbounded storage growth that could make the contract unusable
pub const MAX_TRANSACTIONS: u32 = 1000;

/// Current state version for migration tracking
/// Incremented when storage structure changes require migration
pub const STATE_VERSION: u32 = 2;

/// Validates a list of actions and calculates total deposit needed
///
/// Performs comprehensive validation to ensure:
/// - Action count is within limits
/// - Gas allocation is reasonable
/// - Arguments are appropriately sized
/// - All parameters are valid
///
/// Returns the total NEAR deposit required across all actions
pub fn validate_actions(actions: &Vec<Action>) -> u128 {
    require!(!actions.is_empty(), "Actions cannot be empty");
    require!(actions.len() <= MAX_ACTIONS, "Too many actions (max 10)");

    let mut total_gas = 0u64;
    let mut total_deposit = 0u128;

    for action in actions {
        match action {
            Action::Transfer { amount } => {
                require!(*amount > 0, "Transfer amount must be positive");
                // Use saturating_add to prevent overflow attacks
                total_deposit = total_deposit.saturating_add(*amount);
            }
            Action::FunctionCall {
                method_name,
                args,
                gas,
                deposit,
            } => {
                // Validate function call parameters
                require!(args.len() <= MAX_ARGS_LEN, "Args too large (max 32KB)");
                require!(
                    method_name.len() <= MAX_METHOD_NAME_LEN,
                    "Method name too long"
                );
                require!(!method_name.is_empty(), "Method name cannot be empty");

                // Ensure reasonable gas allocation
                require!(*gas > 0, "Gas must be positive");
                require!(
                    *gas <= MAX_GAS_PER_ACTION,
                    "Gas per action exceeds limit (max 100 TGas)"
                );
                total_gas = total_gas.saturating_add(*gas);

                // Accumulate all deposits
                total_deposit = total_deposit.saturating_add(*deposit);
            }
        }
    }

    // Final check: total gas must be within bounds
    require!(
        total_gas <= MAX_TOTAL_GAS,
        "Total gas exceeds limit (max 250 TGas)"
    );
    total_deposit
}

/// Calculates total NEAR deposit needed for a transaction
///
/// Sums all transfer amounts and function call deposits.
/// Used for balance validation before execution.
#[inline]
pub fn calculate_transaction_deposit(tx: &crate::types::Transaction) -> u128 {
    tx.actions.iter().fold(0u128, |acc, action| match action {
        Action::Transfer { amount } => acc.saturating_add(*amount),
        Action::FunctionCall { deposit, .. } => acc.saturating_add(*deposit),
    })
}
