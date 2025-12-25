//! VM state and main VM struct

use crate::core::BlockContext;
use crate::vm::{Stack, Memory, Storage, CallFrame};
use crate::journal::Journal;

/// Complete VM state at a point in time
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

/// The main virtual machine
pub struct Vm {
    /// Current execution state
    pub(crate) state: VmState,
    /// Bytecode being executed
    pub(crate) bytecode: Vec<u8>,
    /// Journal for time-travel debugging
    pub(crate) journal: Journal,
    /// Block context (deterministic inputs)
    pub(crate) context: BlockContext,
    /// Valid jump destinations (cached)
    pub(crate) jump_dests: Vec<bool>,
    /// Call stack for nested calls
    pub(crate) call_stack: Vec<CallFrame>,
}

impl Vm {
    /// Create a new VM instance
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

    /// Get current state reference
    pub fn state(&self) -> &VmState {
        &self.state
    }

    /// Get mutable state reference
    pub fn state_mut(&mut self) -> &mut VmState {
        &mut self.state
    }

    /// Get journal reference
    pub fn journal(&self) -> &Journal {
        &self.journal
    }

    /// Get block context
    pub fn context(&self) -> &BlockContext {
        &self.context
    }

    /// Get bytecode
    pub fn bytecode(&self) -> &[u8] {
        &self.bytecode
    }

    /// Check if address is a valid jump destination
    pub fn is_valid_jump(&self, dest: usize) -> bool {
        self.jump_dests.get(dest).copied().unwrap_or(false)
    }

    /// Analyze bytecode to find valid JUMPDEST positions
    fn analyze_jump_dests(bytecode: &[u8]) -> Vec<bool> {
        let mut result = vec![false; bytecode.len()];
        let mut i = 0;
        
        while i < bytecode.len() {
            let opcode = bytecode[i];
            if opcode == 0x5B { // JUMPDEST
                result[i] = true;
            }
            // Skip PUSH immediate data
            if opcode >= 0x60 && opcode <= 0x7F {
                let push_size = (opcode - 0x5F) as usize;
                i += push_size;
            }
            i += 1;
        }
        
        result
    }

    /// Compute a hash of the current state (for determinism verification)
    pub fn compute_state_hash(&self) -> [u8; 32] {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // Hash PC
        self.state.pc.hash(&mut hasher);
        
        // Hash gas
        self.state.gas.hash(&mut hasher);
        
        // Hash stack
        for val in self.state.stack.as_slice() {
            val.0.hash(&mut hasher);
        }
        
        // Hash memory size (not contents for performance)
        self.state.memory.size().hash(&mut hasher);
        
        let hash = hasher.finish();
        let mut result = [0u8; 32];
        result[..8].copy_from_slice(&hash.to_le_bytes());
        result
    }

    /// Reset VM to initial state
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
