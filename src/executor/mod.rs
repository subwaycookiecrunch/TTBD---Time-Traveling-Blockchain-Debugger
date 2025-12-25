//! Execution engine for the TTBD virtual machine

mod opcodes;
mod interpreter;
mod reverse;

pub use opcodes::Opcode;
pub use interpreter::{StepResult, ExecutionResult};
pub use reverse::apply_inverse;
