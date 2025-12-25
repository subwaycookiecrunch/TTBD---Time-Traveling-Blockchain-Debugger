//! Journal entry types for instruction-level reversibility

use crate::core::U256;
use crate::vm::CallFrameSnapshot;

/// A single state mutation that can be reversed.
#[derive(Clone, Debug)]
pub enum JournalEntry {
    /// Value pushed to stack (reverse: pop)
    StackPush { value: U256 },
    
    /// Value popped from stack (reverse: push)
    StackPop { value: U256 },
    
    /// Memory write (reverse: restore old_data)
    MemoryWrite {
        offset: usize,
        old_data: Vec<u8>,
        new_data: Vec<u8>,
    },
    
    /// Storage write (reverse: restore old_value)
    StorageWrite {
        key: U256,
        old_value: U256,
        new_value: U256,
    },
    
    /// Program counter change (reverse: restore old_pc)
    PcChange {
        old_pc: usize,
        new_pc: usize,
    },
    
    /// Gas change (reverse: restore old_gas)
    GasChange {
        old_gas: u64,
        new_gas: u64,
    },
    
    /// Entering a call (reverse: pop frame)
    CallEnter {
        caller_frame: CallFrameSnapshot,
    },
    
    /// Exiting a call (reverse: push frame, clear return data)
    CallExit {
        callee_frame: CallFrameSnapshot,
        return_data: Vec<u8>,
    },
    
    /// Return data set (reverse: restore old return data)
    ReturnDataSet {
        old_data: Vec<u8>,
        new_data: Vec<u8>,
    },
    
    /// Memory size expansion (for accurate gas accounting on rewind)
    MemoryExpansion {
        old_size: usize,
        new_size: usize,
    },
}

impl JournalEntry {
    /// Estimate memory usage of this entry
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() + match self {
            Self::MemoryWrite { old_data, new_data, .. } => {
                old_data.len() + new_data.len()
            }
            Self::CallEnter { .. } | Self::CallExit { .. } => {
                std::mem::size_of::<CallFrameSnapshot>()
            }
            Self::ReturnDataSet { old_data, new_data } => {
                old_data.len() + new_data.len()
            }
            _ => 0,
        }
    }
}

/// Complete journal for a single instruction execution.
#[derive(Clone, Debug)]
pub struct InstructionJournal {
    /// PC at start of instruction
    pub pc: usize,
    /// Opcode executed
    pub opcode: u8,
    /// All state mutations
    pub entries: Vec<JournalEntry>,
    /// State hash after instruction (for verification)
    pub state_hash: [u8; 32],
    /// Gas before instruction
    pub gas_before: u64,
    /// Gas after instruction
    pub gas_after: u64,
}

impl InstructionJournal {
    /// Create a new instruction journal
    pub fn new(pc: usize, opcode: u8, gas_before: u64) -> Self {
        Self {
            pc,
            opcode,
            entries: Vec::new(),
            state_hash: [0u8; 32],
            gas_before,
            gas_after: gas_before,
        }
    }

    /// Add an entry
    pub fn push(&mut self, entry: JournalEntry) {
        self.entries.push(entry);
    }

    /// Total memory usage of this journal
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() 
            + self.entries.iter().map(|e| e.memory_usage()).sum::<usize>()
    }
}
