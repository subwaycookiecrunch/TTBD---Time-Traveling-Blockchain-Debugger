//! Forward execution interpreter with journaling

use crate::core::{U256, VmError, VmResult, HaltReason};
use crate::vm::Vm;
use crate::executor::Opcode;
use crate::journal::{JournalEntry, InstructionJournal, Checkpoint, StateSnapshot};

/// Result of a single step execution
#[derive(Clone, Debug)]
pub enum StepResult {
    Executed { opcode: Opcode, gas_used: u64 },
    Halted { reason: HaltReason },
    Rewound { steps: usize },
}

/// Final execution result
#[derive(Clone, Debug)]
pub enum ExecutionResult {
    Success { return_data: Vec<u8>, gas_used: u64 },
    Revert { return_data: Vec<u8>, gas_used: u64 },
    Halt { reason: HaltReason, gas_used: u64 },
}

impl Vm {
    /// Execute one instruction forward, journaling all state changes.
    pub fn step_forward(&mut self) -> VmResult<StepResult> {
        if self.state.pc >= self.bytecode.len() {
            return Ok(StepResult::Halted { reason: HaltReason::Stop });
        }

        let opcode_byte = self.bytecode[self.state.pc];
        let opcode = Opcode::from_u8(opcode_byte)
            .ok_or(VmError::InvalidOpcode { opcode: opcode_byte })?;

        let stack_len = self.state.stack.len();
        let required = opcode.stack_inputs();
        if stack_len < required {
            return Err(VmError::StackUnderflow { required, available: stack_len });
        }

        let gas_cost = opcode.base_gas();
        if self.state.gas < gas_cost {
            return Err(VmError::OutOfGas { required: gas_cost, available: self.state.gas });
        }

        let mut insn_journal = InstructionJournal::new(self.state.pc, opcode_byte, self.state.gas);
        let old_pc = self.state.pc;

        let halt = self.execute_opcode(opcode, &mut insn_journal)?;

        let old_gas = self.state.gas;
        self.state.gas -= gas_cost;
        insn_journal.push(JournalEntry::GasChange { old_gas, new_gas: self.state.gas });
        insn_journal.gas_after = self.state.gas;

        if self.state.pc == old_pc {
            let new_pc = old_pc + 1 + opcode.immediate_size();
            insn_journal.push(JournalEntry::PcChange { old_pc, new_pc });
            self.state.pc = new_pc;
        }

        insn_journal.state_hash = self.compute_state_hash();
        self.journal.record(insn_journal);

        if self.journal.should_checkpoint() {
            let snapshot = self.create_state_snapshot();
            let checkpoint = Checkpoint::new(self.journal.len(), snapshot);
            self.journal.add_checkpoint(checkpoint);
        }

        if let Some(reason) = halt {
            return Ok(StepResult::Halted { reason });
        }

        Ok(StepResult::Executed { opcode, gas_used: gas_cost })
    }

    fn execute_opcode(&mut self, opcode: Opcode, journal: &mut InstructionJournal) -> VmResult<Option<HaltReason>> {
        // Handle PUSH/DUP/SWAP first using helper methods
        if opcode.is_push() {
            return self.execute_push(opcode, journal);
        }
        if opcode.is_dup() {
            return self.execute_dup(opcode, journal);
        }
        if opcode.is_swap() {
            return self.execute_swap(opcode, journal);
        }

        match opcode {
            Opcode::Stop => return Ok(Some(HaltReason::Stop)),
            
            Opcode::Add => {
                let a = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: a });
                let b = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: b });
                let result = a.wrapping_add(b);
                self.state.stack.push(result)?;
                journal.push(JournalEntry::StackPush { value: result });
            }
            
            Opcode::Sub => {
                let a = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: a });
                let b = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: b });
                let result = a.wrapping_sub(b);
                self.state.stack.push(result)?;
                journal.push(JournalEntry::StackPush { value: result });
            }
            
            Opcode::Mul => {
                let a = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: a });
                let b = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: b });
                let result = U256::from(a.as_u64().wrapping_mul(b.as_u64()));
                self.state.stack.push(result)?;
                journal.push(JournalEntry::StackPush { value: result });
            }
            
            Opcode::Div => {
                let a = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: a });
                let b = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: b });
                let result = if b.is_zero() { U256::ZERO } else { U256::from(a.as_u64() / b.as_u64()) };
                self.state.stack.push(result)?;
                journal.push(JournalEntry::StackPush { value: result });
            }
            
            Opcode::IsZero => {
                let a = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: a });
                let result = if a.is_zero() { U256::ONE } else { U256::ZERO };
                self.state.stack.push(result)?;
                journal.push(JournalEntry::StackPush { value: result });
            }
            
            Opcode::Eq => {
                let a = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: a });
                let b = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: b });
                let result = if a == b { U256::ONE } else { U256::ZERO };
                self.state.stack.push(result)?;
                journal.push(JournalEntry::StackPush { value: result });
            }
            
            Opcode::Lt => {
                let a = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: a });
                let b = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: b });
                let result = if a.as_u64() < b.as_u64() { U256::ONE } else { U256::ZERO };
                self.state.stack.push(result)?;
                journal.push(JournalEntry::StackPush { value: result });
            }
            
            Opcode::Gt => {
                let a = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: a });
                let b = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: b });
                let result = if a.as_u64() > b.as_u64() { U256::ONE } else { U256::ZERO };
                self.state.stack.push(result)?;
                journal.push(JournalEntry::StackPush { value: result });
            }
            
            Opcode::And => {
                let a = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: a });
                let b = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: b });
                let result = U256([a.0[0] & b.0[0], a.0[1] & b.0[1], a.0[2] & b.0[2], a.0[3] & b.0[3]]);
                self.state.stack.push(result)?;
                journal.push(JournalEntry::StackPush { value: result });
            }
            
            Opcode::Or => {
                let a = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: a });
                let b = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: b });
                let result = U256([a.0[0] | b.0[0], a.0[1] | b.0[1], a.0[2] | b.0[2], a.0[3] | b.0[3]]);
                self.state.stack.push(result)?;
                journal.push(JournalEntry::StackPush { value: result });
            }
            
            Opcode::Xor => {
                let a = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: a });
                let b = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: b });
                let result = U256([a.0[0] ^ b.0[0], a.0[1] ^ b.0[1], a.0[2] ^ b.0[2], a.0[3] ^ b.0[3]]);
                self.state.stack.push(result)?;
                journal.push(JournalEntry::StackPush { value: result });
            }
            
            Opcode::Not => {
                let a = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: a });
                let result = U256([!a.0[0], !a.0[1], !a.0[2], !a.0[3]]);
                self.state.stack.push(result)?;
                journal.push(JournalEntry::StackPush { value: result });
            }
            
            Opcode::Pop => {
                let a = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: a });
            }
            
            Opcode::MLoad => {
                let offset = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: offset });
                let old_size = self.state.memory.size();
                let value = self.state.memory.load(offset.as_usize());
                let new_size = self.state.memory.size();
                if new_size > old_size {
                    journal.push(JournalEntry::MemoryExpansion { old_size, new_size });
                }
                self.state.stack.push(value)?;
                journal.push(JournalEntry::StackPush { value });
            }
            
            Opcode::MStore => {
                let offset = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: offset });
                let value = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value });
                let old_size = self.state.memory.size();
                let old_data = self.state.memory.store(offset.as_usize(), value);
                let new_size = self.state.memory.size();
                if new_size > old_size {
                    journal.push(JournalEntry::MemoryExpansion { old_size, new_size });
                }
                journal.push(JournalEntry::MemoryWrite {
                    offset: offset.as_usize(),
                    old_data,
                    new_data: value.to_be_bytes().to_vec(),
                });
            }
            
            Opcode::MStore8 => {
                let offset = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: offset });
                let value = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value });
                let byte = (value.0[0] & 0xFF) as u8;
                let old_byte = self.state.memory.store_byte(offset.as_usize(), byte);
                journal.push(JournalEntry::MemoryWrite {
                    offset: offset.as_usize(),
                    old_data: vec![old_byte],
                    new_data: vec![byte],
                });
            }
            
            Opcode::SLoad => {
                let key = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: key });
                let value = self.state.storage.get(&key);
                self.state.stack.push(value)?;
                journal.push(JournalEntry::StackPush { value });
            }
            
            Opcode::SStore => {
                let key = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: key });
                let value = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value });
                let old_value = self.state.storage.insert(key, value);
                journal.push(JournalEntry::StorageWrite { key, old_value, new_value: value });
            }
            
            Opcode::Jump => {
                let dest = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: dest });
                let dest_usize = dest.as_usize();
                if !self.is_valid_jump(dest_usize) {
                    return Err(VmError::InvalidJump { destination: dest_usize });
                }
                journal.push(JournalEntry::PcChange { old_pc: self.state.pc, new_pc: dest_usize });
                self.state.pc = dest_usize;
            }
            
            Opcode::JumpI => {
                let dest = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: dest });
                let cond = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: cond });
                if !cond.is_zero() {
                    let dest_usize = dest.as_usize();
                    if !self.is_valid_jump(dest_usize) {
                        return Err(VmError::InvalidJump { destination: dest_usize });
                    }
                    journal.push(JournalEntry::PcChange { old_pc: self.state.pc, new_pc: dest_usize });
                    self.state.pc = dest_usize;
                }
            }
            
            Opcode::Pc => {
                let value = U256::from(self.state.pc);
                self.state.stack.push(value)?;
                journal.push(JournalEntry::StackPush { value });
            }
            
            Opcode::MSize => {
                let value = U256::from(self.state.memory.size());
                self.state.stack.push(value)?;
                journal.push(JournalEntry::StackPush { value });
            }
            
            Opcode::Gas => {
                let value = U256::from(self.state.gas);
                self.state.stack.push(value)?;
                journal.push(JournalEntry::StackPush { value });
            }
            
            Opcode::JumpDest => {}
            
            Opcode::Return => {
                let offset = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: offset });
                let size = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: size });
                let mut return_data = vec![0u8; size.as_usize()];
                for i in 0..size.as_usize() {
                    return_data[i] = self.state.memory.load_byte(offset.as_usize() + i);
                }
                return Ok(Some(HaltReason::Return(return_data)));
            }
            
            Opcode::Revert => {
                let offset = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: offset });
                let size = self.state.stack.pop()?;
                journal.push(JournalEntry::StackPop { value: size });
                let mut return_data = vec![0u8; size.as_usize()];
                for i in 0..size.as_usize() {
                    return_data[i] = self.state.memory.load_byte(offset.as_usize() + i);
                }
                return Ok(Some(HaltReason::Revert(return_data)));
            }
            
            Opcode::Invalid => return Ok(Some(HaltReason::InvalidOpcode(opcode as u8))),
            
            _ => {} // Unimplemented opcodes - no-op
        }
        Ok(None)
    }

    fn execute_push(&mut self, opcode: Opcode, journal: &mut InstructionJournal) -> VmResult<Option<HaltReason>> {
        let size = opcode.immediate_size();
        let mut bytes = [0u8; 32];
        let code_len = self.bytecode.len();
        let start = self.state.pc + 1;
        for i in 0..size {
            if start + i < code_len {
                bytes[32 - size + i] = self.bytecode[start + i];
            }
        }
        let value = U256::from_be_bytes(bytes);
        self.state.stack.push(value)?;
        journal.push(JournalEntry::StackPush { value });
        Ok(None)
    }

    fn execute_dup(&mut self, opcode: Opcode, journal: &mut InstructionJournal) -> VmResult<Option<HaltReason>> {
        let depth = (opcode as u8 - 0x80) as usize;
        let value = self.state.stack.peek(depth)?;
        self.state.stack.push(value)?;
        journal.push(JournalEntry::StackPush { value });
        Ok(None)
    }

    fn execute_swap(&mut self, opcode: Opcode, journal: &mut InstructionJournal) -> VmResult<Option<HaltReason>> {
        let depth = (opcode as u8 - 0x90 + 1) as usize;
        let top = self.state.stack.peek(0)?;
        let other = self.state.stack.peek(depth)?;
        journal.push(JournalEntry::StackPop { value: top });
        journal.push(JournalEntry::StackPop { value: other });
        self.state.stack.swap(depth)?;
        journal.push(JournalEntry::StackPush { value: top });
        journal.push(JournalEntry::StackPush { value: other });
        Ok(None)
    }

    fn create_state_snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            stack: self.state.stack.to_vec(),
            memory: self.state.memory.snapshot(),
            storage: self.state.storage.snapshot(),
            pc: self.state.pc,
            gas: self.state.gas,
            call_depth: self.state.call_depth,
            return_data: self.state.return_data.clone(),
        }
    }

    pub fn run(&mut self) -> VmResult<ExecutionResult> {
        let initial_gas = self.state.gas;
        loop {
            match self.step_forward()? {
                StepResult::Halted { reason } => {
                    let gas_used = initial_gas - self.state.gas;
                    return Ok(match reason {
                        HaltReason::Stop => ExecutionResult::Success { return_data: Vec::new(), gas_used },
                        HaltReason::Return(data) => ExecutionResult::Success { return_data: data, gas_used },
                        HaltReason::Revert(data) => ExecutionResult::Revert { return_data: data, gas_used },
                        _ => ExecutionResult::Halt { reason, gas_used },
                    });
                }
                StepResult::Executed { .. } => continue,
                StepResult::Rewound { .. } => unreachable!(),
            }
        }
    }
}
