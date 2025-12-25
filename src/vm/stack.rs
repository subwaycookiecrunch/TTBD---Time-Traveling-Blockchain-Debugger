//! Operand stack for the TTBD virtual machine

use crate::core::{U256, VmError, VmResult};

/// Maximum stack depth (same as EVM)
pub const MAX_STACK_SIZE: usize = 1024;

/// Operand stack with bounded size.
pub struct Stack {
    data: [U256; MAX_STACK_SIZE],
    len: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            data: [U256::ZERO; MAX_STACK_SIZE],
            len: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, value: U256) -> VmResult<()> {
        if self.len >= MAX_STACK_SIZE {
            return Err(VmError::StackOverflow { max: MAX_STACK_SIZE });
        }
        self.data[self.len] = value;
        self.len += 1;
        Ok(())
    }

    #[inline]
    pub fn pop(&mut self) -> VmResult<U256> {
        if self.len == 0 {
            return Err(VmError::StackUnderflow { required: 1, available: 0 });
        }
        self.len -= 1;
        Ok(self.data[self.len])
    }

    #[inline]
    pub fn peek(&self, depth: usize) -> VmResult<U256> {
        if depth >= self.len {
            return Err(VmError::StackUnderflow {
                required: depth + 1,
                available: self.len,
            });
        }
        Ok(self.data[self.len - 1 - depth])
    }

    #[inline]
    pub fn swap(&mut self, depth: usize) -> VmResult<()> {
        if depth >= self.len {
            return Err(VmError::StackUnderflow {
                required: depth + 1,
                available: self.len,
            });
        }
        let top_idx = self.len - 1;
        let other_idx = self.len - 1 - depth;
        self.data.swap(top_idx, other_idx);
        Ok(())
    }

    #[inline]
    pub fn dup(&mut self, depth: usize) -> VmResult<()> {
        let value = self.peek(depth)?;
        self.push(value)
    }

    #[inline]
    pub fn as_slice(&self) -> &[U256] {
        &self.data[..self.len]
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn to_vec(&self) -> Vec<U256> {
        self.data[..self.len].to_vec()
    }

    pub fn restore_from(&mut self, snapshot: &[U256]) {
        let len = snapshot.len().min(MAX_STACK_SIZE);
        self.data[..len].copy_from_slice(&snapshot[..len]);
        self.len = len;
    }

    // === Unsafe hot-path methods ===

    /// Pop without bounds checking.
    /// # Safety: Caller must ensure stack has at least 1 element.
    #[inline(always)]
    pub unsafe fn pop_unchecked(&mut self) -> U256 {
        self.len -= 1;
        unsafe { *self.data.get_unchecked(self.len) }
    }

    /// Pop two values without bounds checking.
    /// # Safety: Caller must ensure stack has at least 2 elements.
    #[inline(always)]
    pub unsafe fn pop2_unchecked(&mut self) -> (U256, U256) {
        let a = unsafe { *self.data.get_unchecked(self.len - 1) };
        let b = unsafe { *self.data.get_unchecked(self.len - 2) };
        self.len -= 2;
        (a, b)
    }

    /// Push without bounds checking.
    /// # Safety: Caller must ensure stack has room.
    #[inline(always)]
    pub unsafe fn push_unchecked(&mut self, value: U256) {
        unsafe { *self.data.get_unchecked_mut(self.len) = value };
        self.len += 1;
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Stack {
    fn clone(&self) -> Self {
        let mut new_stack = Self::new();
        new_stack.data[..self.len].copy_from_slice(&self.data[..self.len]);
        new_stack.len = self.len;
        new_stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut stack = Stack::new();
        stack.push(U256::from(42u64)).unwrap();
        stack.push(U256::from(100u64)).unwrap();
        
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.pop().unwrap(), U256::from(100u64));
        assert_eq!(stack.pop().unwrap(), U256::from(42u64));
        assert!(stack.is_empty());
    }

    #[test]
    fn test_overflow() {
        let mut stack = Stack::new();
        for i in 0..MAX_STACK_SIZE {
            stack.push(U256::from(i as u64)).unwrap();
        }
        assert!(stack.push(U256::ONE).is_err());
    }
}
