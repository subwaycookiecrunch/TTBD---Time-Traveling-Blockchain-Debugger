mod interpreter;
mod opcodes;
mod reverse;

pub use interpreter::{ExecutionResult, StepResult};
pub use opcodes::Opcode;
pub use reverse::apply_inverse;
