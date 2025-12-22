use crate::core::U256;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Checkpoint {
    pub instruction_index: usize,
    pub state_snapshot: StateSnapshot,
}

#[derive(Clone, Debug)]
pub struct StateSnapshot {
    pub stack: Vec<U256>,
    pub memory: Vec<u8>,
    pub storage: HashMap<U256, U256>,
    pub pc: usize,
    pub gas: u64,
    pub call_depth: usize,
    pub return_data: Vec<u8>,
}

impl StateSnapshot {
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

    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.stack.len() * std::mem::size_of::<U256>()
            + self.memory.len()
            + self.storage.len() * (std::mem::size_of::<U256>() * 2)
            + self.return_data.len()
    }
}

impl Checkpoint {
    pub fn new(instruction_index: usize, state: StateSnapshot) -> Self {
        Self {
            instruction_index,
            state_snapshot: state,
        }
    }
}
