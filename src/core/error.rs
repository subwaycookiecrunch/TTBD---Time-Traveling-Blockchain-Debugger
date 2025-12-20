use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmError {
    StackUnderflow { required: usize, available: usize },
    StackOverflow { max: usize },
    OutOfGas { required: u64, available: u64 },
    InvalidJump { destination: usize },
    InvalidOpcode { opcode: u8 },
    OutOfBoundsMemory { offset: usize, size: usize },
    WriteProtectedStorage,
    CallDepthExceeded { max: usize },
    JournalExhausted,
    CheckpointNotFound { index: usize },
    Halted { reason: HaltReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HaltReason {
    Stop,
    Return(Vec<u8>),
    Revert(Vec<u8>),
    OutOfGas,
    InvalidOpcode(u8),
    InvalidJump,
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StackUnderflow {
                required,
                available,
            } => {
                write!(f, "stack underflow: need {required}, have {available}")
            }
            Self::StackOverflow { max } => {
                write!(f, "stack overflow: max size is {max}")
            }
            Self::OutOfGas {
                required,
                available,
            } => {
                write!(f, "out of gas: need {required}, have {available}")
            }
            Self::InvalidJump { destination } => {
                write!(f, "invalid jump to {destination:#x}")
            }
            Self::InvalidOpcode { opcode } => {
                write!(f, "invalid opcode: {opcode:#04x}")
            }
            Self::OutOfBoundsMemory { offset, size } => {
                write!(
                    f,
                    "memory access out of bounds: offset={offset}, size={size}"
                )
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

pub type VmResult<T> = Result<T, VmError>;
