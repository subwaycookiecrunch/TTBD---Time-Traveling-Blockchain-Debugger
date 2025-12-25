//! Error types for the TTBD virtual machine

use std::fmt;

/// Errors that can occur during VM execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmError {
    /// Stack underflow - not enough values on stack
    StackUnderflow {
        required: usize,
        available: usize,
    },
    /// Stack overflow - exceeded maximum stack size
    StackOverflow {
        max: usize,
    },
    /// Out of gas - operation requires more gas than available
    OutOfGas {
        required: u64,
        available: u64,
    },
    /// Invalid jump destination
    InvalidJump {
        destination: usize,
    },
    /// Invalid opcode encountered
    InvalidOpcode {
        opcode: u8,
    },
    /// Memory access out of bounds
    OutOfBoundsMemory {
        offset: usize,
        size: usize,
    },
    /// Attempted to write to read-only storage
    WriteProtectedStorage,
    /// Maximum call depth exceeded
    CallDepthExceeded {
        max: usize,
    },
    /// Journal exhausted - cannot rewind further
    JournalExhausted,
    /// Checkpoint not found
    CheckpointNotFound {
        index: usize,
    },
    /// Execution halted
    Halted {
        reason: HaltReason,
    },
}

/// Reasons for execution halt
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HaltReason {
    /// Normal stop (STOP opcode)
    Stop,
    /// Successful return with data
    Return(Vec<u8>),
    /// Revert with data
    Revert(Vec<u8>),
    /// Ran out of gas
    OutOfGas,
    /// Invalid opcode
    InvalidOpcode(u8),
    /// Invalid jump
    InvalidJump,
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StackUnderflow { required, available } => {
                write!(f, "stack underflow: need {required}, have {available}")
            }
            Self::StackOverflow { max } => {
                write!(f, "stack overflow: max size is {max}")
            }
            Self::OutOfGas { required, available } => {
                write!(f, "out of gas: need {required}, have {available}")
            }
            Self::InvalidJump { destination } => {
                write!(f, "invalid jump to {destination:#x}")
            }
            Self::InvalidOpcode { opcode } => {
                write!(f, "invalid opcode: {opcode:#04x}")
            }
            Self::OutOfBoundsMemory { offset, size } => {
                write!(f, "memory access out of bounds: offset={offset}, size={size}")
            }
            Self::WriteProtectedStorage => {
                write!(f, "write to protected storage")
            }
            Self::CallDepthExceeded { max } => {
                write!(f, "call depth exceeded: max is {max}")
            }
            Self::JournalExhausted => {
                write!(f, "journal exhausted: cannot rewind further")
            }
            Self::CheckpointNotFound { index } => {
                write!(f, "checkpoint not found at index {index}")
            }
            Self::Halted { reason } => {
                write!(f, "execution halted: {reason:?}")
            }
        }
    }
}

impl std::error::Error for VmError {}

/// Result type alias for VM operations
pub type VmResult<T> = Result<T, VmError>;
