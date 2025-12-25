//! Persistent key-value storage for the TTBD virtual machine

use std::collections::HashMap;
use crate::core::U256;

/// Persistent storage (survives across calls within a transaction).
/// 
/// Each storage write is journaled for reversibility.
pub struct Storage {
    /// Current storage state
    data: HashMap<U256, U256>,
    /// Original values (for gas calculation and journaling)
    original: HashMap<U256, U256>,
}

impl Storage {
    /// Create new empty storage
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            original: HashMap::new(),
        }
    }

    /// Create storage with initial state
    pub fn with_state(state: HashMap<U256, U256>) -> Self {
        Self {
            original: state.clone(),
            data: state,
        }
    }

    /// Load value from storage (0 if not set)
    #[inline]
    pub fn get(&self, key: &U256) -> U256 {
        self.data.get(key).copied().unwrap_or(U256::ZERO)
    }

    /// Store value and return previous value (for journaling)
    pub fn insert(&mut self, key: U256, value: U256) -> U256 {
        let old = self.data.insert(key, value).unwrap_or(U256::ZERO);
        // Track original value for gas refunds
        self.original.entry(key).or_insert(old);
        old
    }

    /// Check if key exists with non-zero value
    #[inline]
    pub fn contains(&self, key: &U256) -> bool {
        self.data.get(key).map(|v| !v.is_zero()).unwrap_or(false)
    }

    /// Get original value (before any writes in current tx)
    pub fn get_original(&self, key: &U256) -> U256 {
        self.original.get(key).copied().unwrap_or(U256::ZERO)
    }

    /// Calculate gas cost for SSTORE operation
    pub fn sstore_gas_cost(&self, key: &U256, new_value: &U256) -> u64 {
        let current = self.get(key);
        let original = self.get_original(key);

        if current == *new_value {
            // No-op
            100
        } else if current == original {
            if original.is_zero() {
                // 0 -> non-zero
                20000
            } else if new_value.is_zero() {
                // non-zero -> 0 (with refund)
                5000
            } else {
                // non-zero -> non-zero (different)
                5000
            }
        } else {
            // Already modified in this tx
            100
        }
    }

    /// Calculate gas refund for SSTORE
    pub fn sstore_refund(&self, key: &U256, new_value: &U256) -> i64 {
        let current = self.get(key);
        let original = self.get_original(key);

        if current == *new_value {
            return 0;
        }

        let mut refund = 0i64;

        if !current.is_zero() && new_value.is_zero() {
            refund += 4800; // SSTORE_CLEARS_SCHEDULE
        }

        if original != current && original == *new_value {
            if original.is_zero() {
                refund += 19900; // SSTORE_SET_GAS - SLOAD_GAS
            } else {
                refund += 2800; // SSTORE_RESET_GAS - SLOAD_GAS
            }
        }

        refund
    }

    /// Snapshot for checkpointing
    pub fn snapshot(&self) -> HashMap<U256, U256> {
        self.data.clone()
    }

    /// Restore from snapshot
    pub fn restore_from(&mut self, snapshot: HashMap<U256, U256>) {
        self.data = snapshot;
    }

    /// Clear storage
    pub fn clear(&mut self) {
        self.data.clear();
        self.original.clear();
    }

    /// Commit storage (make current state the new original)
    pub fn commit(&mut self) {
        self.original = self.data.clone();
    }

    /// Iterate over all key-value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&U256, &U256)> {
        self.data.iter()
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Storage {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            original: self.original.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_storage() {
        let mut storage = Storage::new();
        let key = U256::from(1u64);
        let value = U256::from(42u64);
        
        assert_eq!(storage.get(&key), U256::ZERO);
        storage.insert(key, value);
        assert_eq!(storage.get(&key), value);
    }

    #[test]
    fn test_insert_returns_old() {
        let mut storage = Storage::new();
        let key = U256::from(1u64);
        
        let old1 = storage.insert(key, U256::from(10u64));
        assert_eq!(old1, U256::ZERO);
        
        let old2 = storage.insert(key, U256::from(20u64));
        assert_eq!(old2, U256::from(10u64));
    }

    #[test]
    fn test_original_tracking() {
        let mut storage = Storage::new();
        let key = U256::from(1u64);
        
        storage.insert(key, U256::from(10u64));
        storage.insert(key, U256::from(20u64));
        storage.insert(key, U256::from(30u64));
        
        // Original should still be 0 (the value before first write)
        assert_eq!(storage.get_original(&key), U256::ZERO);
    }
}
