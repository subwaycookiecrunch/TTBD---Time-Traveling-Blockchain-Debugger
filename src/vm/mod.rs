//! VM state containers: stack, memory, storage, and call frames

mod stack;
mod memory;
mod storage;
mod frame;
mod state;

pub use stack::Stack;
pub use memory::Memory;
pub use storage::Storage;
pub use frame::{CallFrame, CallFrameSnapshot};
pub use state::{VmState, Vm};
