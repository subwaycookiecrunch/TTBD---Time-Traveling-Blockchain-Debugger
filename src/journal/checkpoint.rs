//! Checkpoint structures for fast rewind to distant states

use crate::core::U256;
use std::collections::HashMap;

/// A full state snapshot at a point in execution.
#[derive(Clone, Debug)]
pub struct Checkpoint {
    /// Instruction index this checkpoint was taken at
    pub instruction_index: usize,
    /// Full state snapshot
    pub state_snapshot: StateSnapshot,
}

/// Complete snapshot of VM state.
#[derive(Clone, Debug)]
pub struct StateSnapshot {
    /// Stack contents
    pub stack: Vec<U256>,
    /// Memory contents (compressed)
    pub memory: Vec<u8>,
    /// Storage state
    pub storage: HashMap<U256, U256>,
    /// Program counter
    pub pc: usize,
    /// Remaining gas
    pub gas: u64,
    /// Call depth
    pub call_depth: usize,
    /// Return data
    pub return_data: Vec<u8>,
}

impl StateSnapshot {
    /// Create empty snapshot
    pub fn empty() -> Self {
        Self {
            stack: Vec::new(),
            memory: Vec::new(),
            storage: HashMap::new(),
            pc: 0,
            gas: 0,
            call_depth: 0,
            return_data: Vec::new(),
        }
    }

    /// Estimate memory usage
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.stack.len() * std::mem::size_of::<U256>()
            + self.memory.len()
            + self.storage.len() * (std::mem::size_of::<U256>() * 2)
            + self.return_data.len()
    }
}

impl Checkpoint {
    /// Create a new checkpoint
    pub fn new(instruction_index: usize, state: StateSnapshot) -> Self {
        Self {
            instruction_index,
            state_snapshot: state,
        }
    }
}
