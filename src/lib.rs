//! # TTBD: Time-Traveling Blockchain Debugger
//!
//! A research-grade reversible virtual machine for debugging smart contract
//! bytecode with instruction-level time travel.
//!
//! ## Core Concepts
//!
//! - **Δ-Journaling**: Log only state changes, not snapshots
//! - **Reversible Opcodes**: Every instruction has a mathematical inverse
//! - **Hybrid Checkpointing**: O(√N) rewind with bounded memory
//! - **Deterministic Replay**: Bit-exact execution, every time

pub mod core;
pub mod vm;
pub mod journal;
pub mod executor;
pub mod debugger;
pub mod bytecode;

pub use crate::core::{U256, Address, BlockContext, VmError, VmResult};
pub use crate::debugger::TimeTravel;
pub use crate::vm::Vm;
