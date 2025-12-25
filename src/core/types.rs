//! Primitive types for the TTBD virtual machine

/// 256-bit unsigned integer for stack/storage values.
/// 
/// Stored as 4 x u64 in little-endian limb order (limb 0 is least significant).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct U256(pub [u64; 4]);

impl U256 {
    pub const ZERO: Self = Self([0; 4]);
    pub const ONE: Self = Self([1, 0, 0, 0]);
    pub const MAX: Self = Self([u64::MAX; 4]);

    /// Create from big-endian bytes
    pub fn from_be_bytes(bytes: [u8; 32]) -> Self {
        let mut limbs = [0u64; 4];
        for (i, limb) in limbs.iter_mut().rev().enumerate() {
            let offset = i * 8;
            *limb = u64::from_be_bytes([
                bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
                bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
            ]);
        }
        Self(limbs)
    }

    /// Convert to big-endian bytes
    pub fn to_be_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        for (i, limb) in self.0.iter().rev().enumerate() {
            let be = limb.to_be_bytes();
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&be);
        }
        bytes
    }

    /// Check if value is zero
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0[0] == 0 && self.0[1] == 0 && self.0[2] == 0 && self.0[3] == 0
    }

    /// Wrapping addition
    pub fn wrapping_add(self, rhs: Self) -> Self {
        let mut result = [0u64; 4];
        let mut carry = 0u64;
        for i in 0..4 {
            let (sum1, c1) = self.0[i].overflowing_add(rhs.0[i]);
            let (sum2, c2) = sum1.overflowing_add(carry);
            result[i] = sum2;
            carry = (c1 as u64) + (c2 as u64);
        }
        Self(result)
    }

    /// Wrapping subtraction
    pub fn wrapping_sub(self, rhs: Self) -> Self {
        let mut result = [0u64; 4];
        let mut borrow = 0u64;
        for i in 0..4 {
            let (diff1, b1) = self.0[i].overflowing_sub(rhs.0[i]);
            let (diff2, b2) = diff1.overflowing_sub(borrow);
            result[i] = diff2;
            borrow = (b1 as u64) + (b2 as u64);
        }
        Self(result)
    }

    /// Convert to usize (truncating)
    #[inline]
    pub fn as_usize(&self) -> usize {
        self.0[0] as usize
    }

    /// Convert to u64 (truncating)
    #[inline]
    pub fn as_u64(&self) -> u64 {
        self.0[0]
    }
}

impl From<u64> for U256 {
    fn from(v: u64) -> Self {
        Self([v, 0, 0, 0])
    }
}

impl From<usize> for U256 {
    fn from(v: usize) -> Self {
        Self([v as u64, 0, 0, 0])
    }
}

/// 20-byte Ethereum-style address
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Address(pub [u8; 20]);

impl Address {
    pub const ZERO: Self = Self([0u8; 20]);

    pub fn from_slice(slice: &[u8]) -> Self {
        let mut addr = [0u8; 20];
        let len = slice.len().min(20);
        addr[20 - len..].copy_from_slice(&slice[..len]);
        Self(addr)
    }
}

/// Block context providing deterministic environmental inputs.
/// 
/// All fields are explicitly provided rather than queried from the system,
/// ensuring deterministic execution.
#[derive(Clone, Debug)]
pub struct BlockContext {
    /// Block number
    pub number: u64,
    /// Block timestamp (Unix seconds)
    pub timestamp: u64,
    /// Block gas limit
    pub gas_limit: u64,
    /// Block coinbase address (miner/validator)
    pub coinbase: Address,
    /// Block difficulty (PoW) or prevrandao (PoS)
    pub difficulty: U256,
    /// Chain ID for replay protection
    pub chain_id: u64,
    /// Base fee per gas (EIP-1559)
    pub base_fee: U256,
}

impl Default for BlockContext {
    fn default() -> Self {
        Self {
            number: 0,
            timestamp: 0,
            gas_limit: 30_000_000,
            coinbase: Address::ZERO,
            difficulty: U256::ZERO,
            chain_id: 1,
            base_fee: U256::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u256_add() {
        let a = U256::from(100u64);
        let b = U256::from(200u64);
        let c = a.wrapping_add(b);
        assert_eq!(c.as_u64(), 300);
    }

    #[test]
    fn test_u256_sub() {
        let a = U256::from(300u64);
        let b = U256::from(100u64);
        let c = a.wrapping_sub(b);
        assert_eq!(c.as_u64(), 200);
    }

    #[test]
    fn test_u256_bytes_roundtrip() {
        let original = U256([0x1234_5678_9abc_def0, 0xfedcba9876543210, 0, 0]);
        let bytes = original.to_be_bytes();
        let recovered = U256::from_be_bytes(bytes);
        assert_eq!(original, recovered);
    }
}
