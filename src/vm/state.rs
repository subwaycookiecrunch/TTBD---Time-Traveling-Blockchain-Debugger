use crate::core::BlockContext;
use crate::journal::Journal;
use crate::vm::{CallFrame, Memory, Stack, Storage};

#[derive(Clone)]
pub struct VmState {
    pub stack: Stack,
    pub memory: Memory,
    pub storage: Storage,
    pub pc: usize,
    pub gas: u64,
    pub call_depth: usize,
    pub return_data: Vec<u8>,
}

impl VmState {
    pub fn new(gas: u64) -> Self {
        Self {
            stack: Stack::new(),
            memory: Memory::new(),
            storage: Storage::new(),
            pc: 0,
            gas,
            call_depth: 0,
            return_data: Vec::new(),
        }
    }
}

pub struct Vm {
    pub(crate) state: VmState,
    pub(crate) bytecode: Vec<u8>,
    pub(crate) journal: Journal,
    pub(crate) context: BlockContext,
    pub(crate) jump_dests: Vec<bool>, // cached JUMPDEST positions
    pub(crate) call_stack: Vec<CallFrame>,
}

impl Vm {
    pub fn new(bytecode: Vec<u8>, gas: u64, context: BlockContext) -> Self {
        let jump_dests = Self::analyze_jump_dests(&bytecode);
        Self {
            state: VmState::new(gas),
            bytecode,
            journal: Journal::new(1000, 10_000_000),
            context,
            jump_dests,
            call_stack: Vec::new(),
        }
    }

    pub fn state(&self) -> &VmState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut VmState {
        &mut self.state
    }

    pub fn journal(&self) -> &Journal {
        &self.journal
    }

    pub fn context(&self) -> &BlockContext {
        &self.context
    }

    pub fn bytecode(&self) -> &[u8] {
        &self.bytecode
    }

    pub fn is_valid_jump(&self, dest: usize) -> bool {
        self.jump_dests.get(dest).copied().unwrap_or(false)
    }

    fn analyze_jump_dests(bytecode: &[u8]) -> Vec<bool> {
        let mut result = vec![false; bytecode.len()];
        let mut i = 0;

        while i < bytecode.len() {
            let opcode = bytecode[i];
            if opcode == 0x5B {
                // JUMPDEST
                result[i] = true;
            }
            // skip PUSH immediate bytes
            if opcode >= 0x60 && opcode <= 0x7F {
                let push_size = (opcode - 0x5F) as usize;
                i += push_size;
            }
            i += 1;
        }

        result
    }

    // quick-and-dirty state hash for determinism checks
    pub fn compute_state_hash(&self) -> [u8; 32] {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.state.pc.hash(&mut hasher);
        self.state.gas.hash(&mut hasher);
        for val in self.state.stack.as_slice() {
            val.0.hash(&mut hasher);
        }
        // just hash the size, not contents (perf)
        self.state.memory.size().hash(&mut hasher);

        let hash = hasher.finish();
        let mut result = [0u8; 32];
        result[..8].copy_from_slice(&hash.to_le_bytes());
        result
    }

    pub fn reset(&mut self, gas: u64) {
        self.state = VmState::new(gas);
        self.journal.clear();
        self.call_stack.clear();
    }
}

impl Clone for Vm {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            bytecode: self.bytecode.clone(),
            journal: self.journal.clone(),
            context: self.context.clone(),
            jump_dests: self.jump_dests.clone(),
            call_stack: self.call_stack.clone(),
        }
    }
}
