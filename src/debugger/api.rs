//! Time-travel debugger API

use crate::core::{U256, VmResult, HaltReason};
use crate::vm::Vm;
use crate::executor::{StepResult, Opcode};

/// Unique identifier for a breakpoint
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BreakpointId(pub usize);

/// Breakpoint conditions
#[derive(Clone, Debug)]
pub enum Breakpoint {
    Address(usize),
    Opcode(u8),
    StorageAccess(U256),
    GasBelow(u64),
    MemoryAccess { start: usize, end: usize },
    AfterInstructions(usize),
}

/// Reason execution stopped
#[derive(Clone, Debug)]
pub enum StopReason {
    Breakpoint(BreakpointId),
    Halt(HaltReason),
    UserStop,
    ReachedBeginning,
}

/// Time-travel debugger wrapping a VM
pub struct TimeTravel {
    vm: Vm,
    breakpoints: Vec<(BreakpointId, Breakpoint)>,
    next_breakpoint_id: usize,
    instruction_count: usize,
}

impl TimeTravel {
    pub fn new(vm: Vm) -> Self {
        Self {
            vm,
            breakpoints: Vec::new(),
            next_breakpoint_id: 0,
            instruction_count: 0,
        }
    }

    pub fn step_forward(&mut self) -> VmResult<StepResult> {
        let result = self.vm.step_forward()?;
        if matches!(result, StepResult::Executed { .. }) {
            self.instruction_count += 1;
        }
        Ok(result)
    }

    pub fn step_backward(&mut self) -> VmResult<StepResult> {
        let result = self.vm.step_backward()?;
        if matches!(result, StepResult::Rewound { .. }) {
            self.instruction_count = self.instruction_count.saturating_sub(1);
        }
        Ok(result)
    }

    pub fn rewind(&mut self, n: usize) -> VmResult<usize> {
        let rewound = self.vm.rewind(n)?;
        self.instruction_count = self.instruction_count.saturating_sub(rewound);
        Ok(rewound)
    }

    pub fn run_forward(&mut self) -> VmResult<StopReason> {
        loop {
            if let Some(bp_id) = self.check_breakpoints() {
                return Ok(StopReason::Breakpoint(bp_id));
            }
            match self.vm.step_forward()? {
                StepResult::Halted { reason } => return Ok(StopReason::Halt(reason)),
                StepResult::Executed { .. } => self.instruction_count += 1,
                _ => {}
            }
        }
    }

    pub fn run_backward(&mut self) -> VmResult<StopReason> {
        loop {
            if self.vm.journal().is_empty() {
                return Ok(StopReason::ReachedBeginning);
            }
            if let Some(bp_id) = self.check_breakpoints() {
                return Ok(StopReason::Breakpoint(bp_id));
            }
            match self.vm.step_backward()? {
                StepResult::Rewound { .. } => {
                    self.instruction_count = self.instruction_count.saturating_sub(1);
                }
                _ => {}
            }
        }
    }

    pub fn step_n(&mut self, n: usize) -> VmResult<usize> {
        let mut stepped = 0;
        for _ in 0..n {
            match self.step_forward()? {
                StepResult::Halted { .. } => break,
                StepResult::Executed { .. } => stepped += 1,
                _ => {}
            }
        }
        Ok(stepped)
    }

    // ==================== Inspection ====================

    pub fn inspect_stack(&self) -> &[U256] {
        self.vm.state().stack.as_slice()
    }

    pub fn inspect_memory(&self, offset: usize, len: usize) -> Vec<u8> {
        // Create a mutable copy for reading
        let mut result = vec![0u8; len];
        let mem = &self.vm.state().memory;
        for i in 0..len {
            // Read without modifying - access internal state
            result[i] = mem.peek_byte(offset + i);
        }
        result
    }

    pub fn inspect_storage(&self, key: &U256) -> U256 {
        self.vm.state().storage.get(key)
    }

    pub fn inspect_pc(&self) -> usize {
        self.vm.state().pc
    }

    pub fn inspect_gas(&self) -> u64 {
        self.vm.state().gas
    }

    pub fn current_opcode(&self) -> Option<Opcode> {
        let pc = self.vm.state().pc;
        let bytecode = self.vm.bytecode();
        if pc < bytecode.len() {
            Opcode::from_u8(bytecode[pc])
        } else {
            None
        }
    }

    pub fn history_len(&self) -> usize {
        self.vm.journal().len()
    }

    pub fn instruction_count(&self) -> usize {
        self.instruction_count
    }

    pub fn memory_size(&self) -> usize {
        self.vm.state().memory.size()
    }

    pub fn call_depth(&self) -> usize {
        self.vm.state().call_depth
    }

    // ==================== Breakpoints ====================

    pub fn add_breakpoint(&mut self, bp: Breakpoint) -> BreakpointId {
        let id = BreakpointId(self.next_breakpoint_id);
        self.next_breakpoint_id += 1;
        self.breakpoints.push((id, bp));
        id
    }

    pub fn remove_breakpoint(&mut self, id: BreakpointId) -> bool {
        let len_before = self.breakpoints.len();
        self.breakpoints.retain(|(bp_id, _)| *bp_id != id);
        self.breakpoints.len() < len_before
    }

    pub fn list_breakpoints(&self) -> &[(BreakpointId, Breakpoint)] {
        &self.breakpoints
    }

    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
    }

    fn check_breakpoints(&self) -> Option<BreakpointId> {
        let pc = self.vm.state().pc;
        let gas = self.vm.state().gas;

        for (id, bp) in &self.breakpoints {
            let matches = match bp {
                Breakpoint::Address(addr) => pc == *addr,
                Breakpoint::Opcode(op) => self.vm.bytecode().get(pc).copied() == Some(*op),
                Breakpoint::GasBelow(threshold) => gas < *threshold,
                Breakpoint::AfterInstructions(n) => self.instruction_count >= *n,
                Breakpoint::StorageAccess(_) | Breakpoint::MemoryAccess { .. } => false,
            };
            if matches {
                return Some(*id);
            }
        }
        None
    }

    // ==================== Utilities ====================

    pub fn vm(&self) -> &Vm {
        &self.vm
    }

    pub fn vm_mut(&mut self) -> &mut Vm {
        &mut self.vm
    }

    pub fn reset(&mut self, gas: u64) {
        self.vm.reset(gas);
        self.instruction_count = 0;
    }

    pub fn state_hash(&self) -> [u8; 32] {
        self.vm.compute_state_hash()
    }
}

/// Debugger trait for custom implementations
pub trait Debugger {
    fn step_forward(&mut self) -> VmResult<StepResult>;
    fn step_backward(&mut self) -> VmResult<StepResult>;
    fn rewind(&mut self, n: usize) -> VmResult<usize>;
    fn inspect_stack(&self) -> &[U256];
    fn inspect_pc(&self) -> usize;
    fn inspect_gas(&self) -> u64;
    fn add_breakpoint(&mut self, bp: Breakpoint) -> BreakpointId;
    fn remove_breakpoint(&mut self, id: BreakpointId) -> bool;
}

impl Debugger for TimeTravel {
    fn step_forward(&mut self) -> VmResult<StepResult> { TimeTravel::step_forward(self) }
    fn step_backward(&mut self) -> VmResult<StepResult> { TimeTravel::step_backward(self) }
    fn rewind(&mut self, n: usize) -> VmResult<usize> { TimeTravel::rewind(self, n) }
    fn inspect_stack(&self) -> &[U256] { TimeTravel::inspect_stack(self) }
    fn inspect_pc(&self) -> usize { TimeTravel::inspect_pc(self) }
    fn inspect_gas(&self) -> u64 { TimeTravel::inspect_gas(self) }
    fn add_breakpoint(&mut self, bp: Breakpoint) -> BreakpointId { TimeTravel::add_breakpoint(self, bp) }
    fn remove_breakpoint(&mut self, id: BreakpointId) -> bool { TimeTravel::remove_breakpoint(self, id) }
}
