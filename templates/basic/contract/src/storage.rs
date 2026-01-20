use crate::types::Transaction;
use near_sdk::store::{LookupMap, Vector};

/// Transaction storage abstraction
///
/// This trait defines operations for the multisig's optimized storage pattern.
/// The contract uses three complementary data structures for efficiency:
///
/// 1. `tx_ids: Vector<u64>` - Ordered list of transaction IDs (for iteration)
/// 2. `tx_by_id: LookupMap<u64, Transaction>` - Fast O(1) lookup by ID
/// 3. `tx_index: LookupMap<u64, u32>` - Reverse index (ID → position in vector)
///
/// This pattern enables:
/// - O(1) transaction lookup by ID
/// - O(1) position lookup (no linear scanning)
/// - Efficient iteration over all transactions
/// - 50% storage savings vs storing full transactions in vector
#[allow(dead_code)] // Trait defined for future extensibility
pub trait TransactionStorage {
    /// Retrieves a transaction by ID in O(1) time
    fn get_transaction(&self, tx_id: u64) -> Option<&Transaction>;

    /// Retrieves a transaction by ID, panicking if not found
    fn get_transaction_or_panic(&self, tx_id: u64) -> &Transaction;

    /// Gets the vector position of a transaction by ID in O(1) time
    fn get_transaction_index(&self, tx_id: u64) -> Option<u32>;

    /// Inserts or updates a transaction, maintaining consistency across all structures
    fn upsert_transaction(&mut self, tx: Transaction);

    /// Removes a transaction by ID
    /// Note: Only removes from tx_by_id; caller must handle vector/index cleanup
    fn remove_transaction(&mut self, tx_id: u64);

    /// Returns the total number of transactions
    fn transaction_count(&self) -> u32;

    /// Rebuilds the vector and index structures from a new set of transaction IDs
    /// Used during cleanup operations to maintain consistency
    fn rebuild_storage(&mut self, tx_ids_to_keep: Vec<u64>);
}

/// Default storage implementation for the multisig contract
///
/// Holds references to the three storage structures that work together
/// to provide efficient transaction management.
pub struct MultisigStorage<'a> {
    pub tx_ids: &'a mut Vector<u64>,
    pub tx_by_id: &'a mut LookupMap<u64, Transaction>,
    pub tx_index: &'a mut LookupMap<u64, u32>,
}

impl TransactionStorage for MultisigStorage<'_> {
    #[inline]
    fn get_transaction(&self, tx_id: u64) -> Option<&Transaction> {
        self.tx_by_id.get(&tx_id)
    }

    #[inline]
    fn get_transaction_or_panic(&self, tx_id: u64) -> &Transaction {
        self.get_transaction(tx_id).expect("Transaction not found")
    }

    #[inline]
    fn get_transaction_index(&self, tx_id: u64) -> Option<u32> {
        self.tx_index.get(&tx_id).copied()
    }

    fn upsert_transaction(&mut self, tx: Transaction) {
        let tx_id = tx.id;

        // For new transactions, add to vector and create reverse index entry
        if !self.tx_index.contains_key(&tx_id) {
            let position = self.tx_ids.len();
            self.tx_ids.push(tx_id);
            self.tx_index.insert(tx_id, position);
        }

        // Store or update the full transaction data
        self.tx_by_id.insert(tx_id, tx);
    }

    fn remove_transaction(&mut self, tx_id: u64) {
        // Remove the transaction data
        self.tx_by_id.remove(&tx_id);
        // Note: Vector and index cleanup happens via rebuild_storage() during batch operations
    }

    #[inline]
    fn transaction_count(&self) -> u32 {
        self.tx_ids.len()
    }

    fn rebuild_storage(&mut self, tx_ids_to_keep: Vec<u64>) {
        // Clear the ordered list and reverse index
        self.tx_ids.clear();
        *self.tx_index = LookupMap::new(b"i");

        // Rebuild with new ordering, maintaining consistency
        for (new_index, &tx_id) in tx_ids_to_keep.iter().enumerate() {
            self.tx_ids.push(tx_id);
            self.tx_index.insert(tx_id, new_index as u32);
        }
    }
}
