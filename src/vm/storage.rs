use crate::core::U256;
use std::collections::HashMap;

// Persistent key-value storage. Tracks original values for gas refund calcs.
pub struct Storage {
    data: HashMap<U256, U256>,
    original: HashMap<U256, U256>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            original: HashMap::new(),
        }
    }

    pub fn with_state(state: HashMap<U256, U256>) -> Self {
        Self {
            original: state.clone(),
            data: state,
        }
    }

    #[inline]
    pub fn get(&self, key: &U256) -> U256 {
        self.data.get(key).copied().unwrap_or(U256::ZERO)
    }

    // returns old value for journaling
    pub fn insert(&mut self, key: U256, value: U256) -> U256 {
        let old = self.data.insert(key, value).unwrap_or(U256::ZERO);
        self.original.entry(key).or_insert(old);
        old
    }

    #[inline]
    pub fn contains(&self, key: &U256) -> bool {
        self.data.get(key).map(|v| !v.is_zero()).unwrap_or(false)
    }

    pub fn get_original(&self, key: &U256) -> U256 {
        self.original.get(key).copied().unwrap_or(U256::ZERO)
    }

    // EIP-2200 gas schedule (simplified)
    pub fn sstore_gas_cost(&self, key: &U256, new_value: &U256) -> u64 {
        let current = self.get(key);
        let original = self.get_original(key);

        if current == *new_value {
            100
        } else if current == original {
            if original.is_zero() {
                20000
            } else if new_value.is_zero() {
                5000
            } else {
                5000
            }
        } else {
            100
        }
    }

    pub fn sstore_refund(&self, key: &U256, new_value: &U256) -> i64 {
        let current = self.get(key);
        let original = self.get_original(key);

        if current == *new_value {
            return 0;
        }

        let mut refund = 0i64;

        if !current.is_zero() && new_value.is_zero() {
            refund += 4800;
        }

        if original != current && original == *new_value {
            if original.is_zero() {
                refund += 19900;
            } else {
                refund += 2800;
            }
        }

        refund
    }

    pub fn snapshot(&self) -> HashMap<U256, U256> {
        self.data.clone()
    }

    pub fn restore_from(&mut self, snapshot: HashMap<U256, U256>) {
        self.data = snapshot;
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.original.clear();
    }

    pub fn commit(&mut self) {
        self.original = self.data.clone();
    }

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

        // original should still be 0
        assert_eq!(storage.get_original(&key), U256::ZERO);
    }
}
