//! Call frame management for the TTBD virtual machine

use crate::core::{U256, Address};

/// A call frame representing a single execution context
#[derive(Clone)]
pub struct CallFrame {
    /// Program counter
    pub pc: usize,
    /// Bytecode being executed
    pub code: Vec<u8>,
    /// Current contract address
    pub address: Address,
    /// Caller address
    pub caller: Address,
    /// Call value (in wei)
    pub value: U256,
    /// Call data (input)
    pub calldata: Vec<u8>,
    /// Available gas
    pub gas: u64,
    /// Whether this is a static call (read-only)
    pub is_static: bool,
    /// Return data offset in parent memory
    pub return_offset: usize,
    /// Return data size
    pub return_size: usize,
}

impl CallFrame {
    pub fn new(
        code: Vec<u8>,
        address: Address,
        caller: Address,
        value: U256,
        calldata: Vec<u8>,
        gas: u64,
        is_static: bool,
    ) -> Self {
        Self {
            pc: 0,
            code,
            address,
            caller,
            value,
            calldata,
            gas,
            is_static,
            return_offset: 0,
            return_size: 0,
        }
    }

    /// Create a snapshot for journaling
    pub fn snapshot(&self) -> CallFrameSnapshot {
        CallFrameSnapshot {
            pc: self.pc,
            gas: self.gas,
            address: self.address,
            caller: self.caller,
            value: self.value,
            is_static: self.is_static,
        }
    }
}

/// Minimal snapshot of a call frame for journaling
#[derive(Clone, Debug)]
pub struct CallFrameSnapshot {
    pub pc: usize,
    pub gas: u64,
    pub address: Address,
    pub caller: Address,
    pub value: U256,
    pub is_static: bool,
}

/// Maximum call depth
pub const MAX_CALL_DEPTH: usize = 1024;
