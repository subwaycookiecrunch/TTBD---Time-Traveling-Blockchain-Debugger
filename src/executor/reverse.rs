//! Reverse execution - applying inverse operations

use crate::core::{VmError, VmResult};
use crate::vm::Vm;
use crate::journal::JournalEntry;
use crate::executor::StepResult;

/// Apply the inverse of a journal entry to restore previous state.
pub fn apply_inverse(vm: &mut Vm, entry: JournalEntry) -> VmResult<()> {
    match entry {
        JournalEntry::StackPush { value: _ } => {
            vm.state.stack.pop()?;
        }
        JournalEntry::StackPop { value } => {
            vm.state.stack.push(value)?;
        }
        JournalEntry::MemoryWrite { offset, old_data, .. } => {
            vm.state.memory.restore_bytes(offset, &old_data);
        }
        JournalEntry::StorageWrite { key, old_value, .. } => {
            vm.state.storage.insert(key, old_value);
        }
        JournalEntry::PcChange { old_pc, .. } => {
            vm.state.pc = old_pc;
        }
        JournalEntry::GasChange { old_gas, .. } => {
            vm.state.gas = old_gas;
        }
        JournalEntry::CallEnter { caller_frame: _ } => {
            vm.call_stack.pop();
            vm.state.call_depth = vm.state.call_depth.saturating_sub(1);
        }
        JournalEntry::CallExit { callee_frame: _, return_data: _ } => {
            vm.state.call_depth += 1;
        }
        JournalEntry::ReturnDataSet { old_data, .. } => {
            vm.state.return_data = old_data;
        }
        JournalEntry::MemoryExpansion { old_size: _, .. } => {
            // Memory pages remain allocated - this is a known limitation
        }
    }
    Ok(())
}

impl Vm {
    /// Execute one instruction backward, restoring previous state.
    pub fn step_backward(&mut self) -> VmResult<StepResult> {
        let insn = self.journal.pop()
            .ok_or(VmError::JournalExhausted)?;

        // Apply inverse operations in reverse order
        for entry in insn.entries.into_iter().rev() {
            apply_inverse(self, entry)?;
        }

        Ok(StepResult::Rewound { steps: 1 })
    }

    /// Rewind N steps backward
    pub fn rewind(&mut self, n: usize) -> VmResult<usize> {
        let mut rewound = 0;
        for _ in 0..n {
            if self.journal.is_empty() {
                break;
            }
            self.step_backward()?;
            rewound += 1;
        }
        Ok(rewound)
    }

    /// Rewind to a specific instruction index
    pub fn rewind_to(&mut self, target_index: usize) -> VmResult<()> {
        let current = self.journal.len();
        
        if target_index >= current {
            return Ok(());
        }

        // For now, simple step-by-step rewind
        // A more efficient implementation would use checkpoints
        let steps = current - target_index;
        self.rewind(steps)?;
        
        Ok(())
    }

    /// Restore VM state from a snapshot
    pub fn restore_from_snapshot(&mut self, snapshot: &crate::journal::StateSnapshot) {
        self.state.stack.restore_from(&snapshot.stack);
        self.state.memory.restore_from(&snapshot.memory);
        self.state.storage.restore_from(snapshot.storage.clone());
        self.state.pc = snapshot.pc;
        self.state.gas = snapshot.gas;
        self.state.call_depth = snapshot.call_depth;
        self.state.return_data = snapshot.return_data.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::BlockContext;

    #[test]
    fn test_forward_backward_equivalence() {
        // Simple bytecode: PUSH1 0x42, PUSH1 0x01, ADD, STOP
        let bytecode = vec![
            0x60, 0x42, // PUSH1 0x42
            0x60, 0x01, // PUSH1 0x01
            0x01,       // ADD
            0x00,       // STOP
        ];
        
        let mut vm = Vm::new(bytecode, 100_000, BlockContext::default());
        
        // Capture initial state
        let initial_pc = vm.state.pc;
        let initial_gas = vm.state.gas;
        let initial_stack_len = vm.state.stack.len();
        
        // Execute forward until halt
        loop {
            match vm.step_forward().unwrap() {
                StepResult::Halted { .. } => break,
                StepResult::Executed { .. } => continue,
                _ => unreachable!(),
            }
        }
        
        // Get steps from journal (this is accurate)
        let steps = vm.journal.len();
        assert!(steps > 0, "Should have executed some steps");
        
        // Rewind all steps using journal
        while !vm.journal.is_empty() {
            vm.step_backward().unwrap();
        }
        
        // Verify state is restored
        assert_eq!(vm.state.pc, initial_pc, "PC should be restored");
        assert_eq!(vm.state.gas, initial_gas, "Gas should be restored");
        assert_eq!(vm.state.stack.len(), initial_stack_len, "Stack should be empty again");
    }

    #[test]
    fn test_arithmetic_rewind() {
        // PUSH1 10, PUSH1 20, ADD, STOP
        let bytecode = vec![
            0x60, 0x0A, // PUSH1 10
            0x60, 0x14, // PUSH1 20
            0x01,       // ADD
            0x00,       // STOP
        ];
        
        let mut vm = Vm::new(bytecode, 100_000, BlockContext::default());
        
        // Step 1: PUSH1 10
        vm.step_forward().unwrap();
        assert_eq!(vm.state.stack.len(), 1);
        
        // Step 2: PUSH1 20
        vm.step_forward().unwrap();
        assert_eq!(vm.state.stack.len(), 2);
        
        // Step 3: ADD -> result 30
        vm.step_forward().unwrap();
        assert_eq!(vm.state.stack.len(), 1);
        let result = vm.state.stack.peek(0).unwrap();
        assert_eq!(result.as_u64(), 30);
        
        // Rewind ADD
        vm.step_backward().unwrap();
        assert_eq!(vm.state.stack.len(), 2);
        assert_eq!(vm.state.stack.peek(0).unwrap().as_u64(), 20);
        assert_eq!(vm.state.stack.peek(1).unwrap().as_u64(), 10);
        
        // Rewind PUSH 20
        vm.step_backward().unwrap();
        assert_eq!(vm.state.stack.len(), 1);
        assert_eq!(vm.state.stack.peek(0).unwrap().as_u64(), 10);
        
        // Rewind PUSH 10
        vm.step_backward().unwrap();
        assert_eq!(vm.state.stack.len(), 0);
    }

    #[test]
    fn test_storage_rewind() {
        // PUSH1 42, PUSH1 1, SSTORE, STOP
        let bytecode = vec![
            0x60, 0x2A, // PUSH1 42 (value)
            0x60, 0x01, // PUSH1 1 (key)
            0x55,       // SSTORE
            0x00,       // STOP
        ];
        
        let mut vm = Vm::new(bytecode, 100_000, BlockContext::default());
        
        // Execute all
        vm.step_forward().unwrap(); // PUSH 42
        vm.step_forward().unwrap(); // PUSH 1
        vm.step_forward().unwrap(); // SSTORE
        
        // Check storage has value
        use crate::core::U256;
        let key = U256::from(1u64);
        assert_eq!(vm.state.storage.get(&key).as_u64(), 42);
        
        // Rewind SSTORE
        vm.step_backward().unwrap();
        
        // Storage should be back to 0
        assert_eq!(vm.state.storage.get(&key).as_u64(), 0);
    }
}
