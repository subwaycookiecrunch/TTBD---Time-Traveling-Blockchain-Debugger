use crate::core::U256;
use crate::vm::CallFrameSnapshot;

// Each variant is a single reversible state mutation.
#[derive(Clone, Debug)]
pub enum JournalEntry {
    StackPush {
        value: U256,
    },
    StackPop {
        value: U256,
    },
    MemoryWrite {
        offset: usize,
        old_data: Vec<u8>,
        new_data: Vec<u8>,
    },
    StorageWrite {
        key: U256,
        old_value: U256,
        new_value: U256,
    },
    PcChange {
        old_pc: usize,
        new_pc: usize,
    },
    GasChange {
        old_gas: u64,
        new_gas: u64,
    },
    CallEnter {
        caller_frame: CallFrameSnapshot,
    },
    CallExit {
        callee_frame: CallFrameSnapshot,
        return_data: Vec<u8>,
    },
    ReturnDataSet {
        old_data: Vec<u8>,
        new_data: Vec<u8>,
    },
    MemoryExpansion {
        old_size: usize,
        new_size: usize,
    },
}

impl JournalEntry {
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>()
            + match self {
                Self::MemoryWrite {
                    old_data, new_data, ..
                } => old_data.len() + new_data.len(),
                Self::CallEnter { .. } | Self::CallExit { .. } => {
                    std::mem::size_of::<CallFrameSnapshot>()
                }
                Self::ReturnDataSet { old_data, new_data } => old_data.len() + new_data.len(),
                _ => 0,
            }
    }
}

// All the mutations from a single instruction, bundled together.
#[derive(Clone, Debug)]
pub struct InstructionJournal {
    pub pc: usize,
    pub opcode: u8,
    pub entries: Vec<JournalEntry>,
    pub state_hash: [u8; 32],
    pub gas_before: u64,
    pub gas_after: u64,
}

impl InstructionJournal {
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

    pub fn push(&mut self, entry: JournalEntry) {
        self.entries.push(entry);
    }

    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() + self.entries.iter().map(|e| e.memory_usage()).sum::<usize>()
    }
}
