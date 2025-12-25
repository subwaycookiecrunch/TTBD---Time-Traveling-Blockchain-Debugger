//! Opcode definitions and metadata

/// VM opcodes with forward and reverse semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    // ============ Stop and Arithmetic (0x00 - 0x0F) ============
    Stop = 0x00,
    Add = 0x01,
    Mul = 0x02,
    Sub = 0x03,
    Div = 0x04,
    SDiv = 0x05,
    Mod = 0x06,
    SMod = 0x07,
    AddMod = 0x08,
    MulMod = 0x09,
    Exp = 0x0A,
    SignExtend = 0x0B,

    // ============ Comparison & Bitwise (0x10 - 0x1F) ============
    Lt = 0x10,
    Gt = 0x11,
    Slt = 0x12,
    Sgt = 0x13,
    Eq = 0x14,
    IsZero = 0x15,
    And = 0x16,
    Or = 0x17,
    Xor = 0x18,
    Not = 0x19,
    Byte = 0x1A,
    Shl = 0x1B,
    Shr = 0x1C,
    Sar = 0x1D,

    // ============ Keccak256 (0x20) ============
    Keccak256 = 0x20,

    // ============ Environmental (0x30 - 0x3F) ============
    Address = 0x30,
    Balance = 0x31,
    Origin = 0x32,
    Caller = 0x33,
    CallValue = 0x34,
    CallDataLoad = 0x35,
    CallDataSize = 0x36,
    CallDataCopy = 0x37,
    CodeSize = 0x38,
    CodeCopy = 0x39,
    GasPrice = 0x3A,
    ExtCodeSize = 0x3B,
    ExtCodeCopy = 0x3C,
    ReturnDataSize = 0x3D,
    ReturnDataCopy = 0x3E,
    ExtCodeHash = 0x3F,

    // ============ Block Information (0x40 - 0x4F) ============
    BlockHash = 0x40,
    Coinbase = 0x41,
    Timestamp = 0x42,
    Number = 0x43,
    Difficulty = 0x44,
    GasLimit = 0x45,
    ChainId = 0x46,
    SelfBalance = 0x47,
    BaseFee = 0x48,

    // ============ Stack, Memory, Storage (0x50 - 0x5F) ============
    Pop = 0x50,
    MLoad = 0x51,
    MStore = 0x52,
    MStore8 = 0x53,
    SLoad = 0x54,
    SStore = 0x55,
    Jump = 0x56,
    JumpI = 0x57,
    Pc = 0x58,
    MSize = 0x59,
    Gas = 0x5A,
    JumpDest = 0x5B,

    // ============ Push (0x60 - 0x7F) ============
    Push1 = 0x60,
    Push2 = 0x61,
    Push3 = 0x62,
    Push4 = 0x63,
    Push5 = 0x64,
    Push6 = 0x65,
    Push7 = 0x66,
    Push8 = 0x67,
    Push9 = 0x68,
    Push10 = 0x69,
    Push11 = 0x6A,
    Push12 = 0x6B,
    Push13 = 0x6C,
    Push14 = 0x6D,
    Push15 = 0x6E,
    Push16 = 0x6F,
    Push17 = 0x70,
    Push18 = 0x71,
    Push19 = 0x72,
    Push20 = 0x73,
    Push21 = 0x74,
    Push22 = 0x75,
    Push23 = 0x76,
    Push24 = 0x77,
    Push25 = 0x78,
    Push26 = 0x79,
    Push27 = 0x7A,
    Push28 = 0x7B,
    Push29 = 0x7C,
    Push30 = 0x7D,
    Push31 = 0x7E,
    Push32 = 0x7F,

    // ============ Dup (0x80 - 0x8F) ============
    Dup1 = 0x80,
    Dup2 = 0x81,
    Dup3 = 0x82,
    Dup4 = 0x83,
    Dup5 = 0x84,
    Dup6 = 0x85,
    Dup7 = 0x86,
    Dup8 = 0x87,
    Dup9 = 0x88,
    Dup10 = 0x89,
    Dup11 = 0x8A,
    Dup12 = 0x8B,
    Dup13 = 0x8C,
    Dup14 = 0x8D,
    Dup15 = 0x8E,
    Dup16 = 0x8F,

    // ============ Swap (0x90 - 0x9F) ============
    Swap1 = 0x90,
    Swap2 = 0x91,
    Swap3 = 0x92,
    Swap4 = 0x93,
    Swap5 = 0x94,
    Swap6 = 0x95,
    Swap7 = 0x96,
    Swap8 = 0x97,
    Swap9 = 0x98,
    Swap10 = 0x99,
    Swap11 = 0x9A,
    Swap12 = 0x9B,
    Swap13 = 0x9C,
    Swap14 = 0x9D,
    Swap15 = 0x9E,
    Swap16 = 0x9F,

    // ============ Log (0xA0 - 0xA4) ============
    Log0 = 0xA0,
    Log1 = 0xA1,
    Log2 = 0xA2,
    Log3 = 0xA3,
    Log4 = 0xA4,

    // ============ System (0xF0 - 0xFF) ============
    Create = 0xF0,
    Call = 0xF1,
    CallCode = 0xF2,
    Return = 0xF3,
    DelegateCall = 0xF4,
    Create2 = 0xF5,
    StaticCall = 0xFA,
    Revert = 0xFD,
    Invalid = 0xFE,
    SelfDestruct = 0xFF,
}

impl Opcode {
    /// Check if this is a PUSH opcode
    #[inline]
    pub fn is_push(&self) -> bool {
        let b = *self as u8;
        b >= 0x60 && b <= 0x7F
    }

    /// Check if this is a DUP opcode
    #[inline]
    pub fn is_dup(&self) -> bool {
        let b = *self as u8;
        b >= 0x80 && b <= 0x8F
    }

    /// Check if this is a SWAP opcode
    #[inline]
    pub fn is_swap(&self) -> bool {
        let b = *self as u8;
        b >= 0x90 && b <= 0x9F
    }

    /// Check if this is a LOG opcode
    #[inline]
    pub fn is_log(&self) -> bool {
        let b = *self as u8;
        b >= 0xA0 && b <= 0xA4
    }

    /// Parse opcode from byte
    pub fn from_u8(byte: u8) -> Option<Self> {
        // All valid opcodes can be transmuted safely from their byte representation
        match byte {
            0x00..=0x0B => Some(unsafe { std::mem::transmute(byte) }),
            0x10..=0x1D => Some(unsafe { std::mem::transmute(byte) }),
            0x20 => Some(Self::Keccak256),
            0x30..=0x3F => Some(unsafe { std::mem::transmute(byte) }),
            0x40..=0x48 => Some(unsafe { std::mem::transmute(byte) }),
            0x50..=0x5B => Some(unsafe { std::mem::transmute(byte) }),
            0x60..=0x7F => Some(unsafe { std::mem::transmute(byte) }),
            0x80..=0x8F => Some(unsafe { std::mem::transmute(byte) }),
            0x90..=0x9F => Some(unsafe { std::mem::transmute(byte) }),
            0xA0..=0xA4 => Some(unsafe { std::mem::transmute(byte) }),
            0xF0..=0xF5 => Some(unsafe { std::mem::transmute(byte) }),
            0xFA => Some(Self::StaticCall),
            0xFD => Some(Self::Revert),
            0xFE => Some(Self::Invalid),
            0xFF => Some(Self::SelfDestruct),
            _ => None,
        }
    }

    /// Number of stack inputs required
    pub fn stack_inputs(&self) -> usize {
        let byte = *self as u8;
        
        // Handle ranges first using byte value
        if self.is_push() {
            return 0;
        }
        if self.is_dup() {
            return (byte - 0x80 + 1) as usize;
        }
        if self.is_swap() {
            return (byte - 0x90 + 2) as usize;
        }
        
        match self {
            Self::Stop | Self::JumpDest | Self::Invalid => 0,
            Self::Address | Self::Origin | Self::Caller | Self::CallValue 
            | Self::CallDataSize | Self::CodeSize | Self::GasPrice 
            | Self::ReturnDataSize | Self::Coinbase | Self::Timestamp
            | Self::Number | Self::Difficulty | Self::GasLimit 
            | Self::ChainId | Self::SelfBalance | Self::BaseFee
            | Self::Pc | Self::MSize | Self::Gas => 0,
            Self::IsZero | Self::Not | Self::Pop | Self::MLoad | Self::SLoad
            | Self::Jump | Self::Balance | Self::ExtCodeSize | Self::ExtCodeHash
            | Self::BlockHash | Self::CallDataLoad => 1,
            Self::Add | Self::Mul | Self::Sub | Self::Div | Self::SDiv
            | Self::Mod | Self::SMod | Self::Exp | Self::SignExtend
            | Self::Lt | Self::Gt | Self::Slt | Self::Sgt | Self::Eq
            | Self::And | Self::Or | Self::Xor | Self::Byte
            | Self::Shl | Self::Shr | Self::Sar
            | Self::MStore | Self::MStore8 | Self::SStore | Self::JumpI
            | Self::Return | Self::Revert => 2,
            Self::AddMod | Self::MulMod | Self::CallDataCopy | Self::CodeCopy
            | Self::ReturnDataCopy | Self::Keccak256 | Self::Log0 => 3,
            Self::ExtCodeCopy | Self::Log1 | Self::Create => 4,
            Self::Log2 | Self::Create2 => 5,
            Self::Log3 | Self::Call | Self::CallCode | Self::DelegateCall => 6,
            Self::Log4 | Self::StaticCall => 7,
            Self::SelfDestruct => 1,
            _ => 0, // PUSH/DUP/SWAP handled above
        }
    }

    /// Number of stack outputs produced
    pub fn stack_outputs(&self) -> usize {
        let byte = *self as u8;
        
        if self.is_dup() {
            return (byte - 0x80 + 2) as usize;
        }
        if self.is_swap() {
            return (byte - 0x90 + 2) as usize;
        }
        
        match self {
            Self::Stop | Self::JumpDest | Self::Invalid | Self::Pop
            | Self::MStore | Self::MStore8 | Self::SStore | Self::Jump
            | Self::JumpI | Self::Return | Self::Revert | Self::SelfDestruct
            | Self::Log0 | Self::Log1 | Self::Log2 | Self::Log3 | Self::Log4
            | Self::CallDataCopy | Self::CodeCopy | Self::ExtCodeCopy
            | Self::ReturnDataCopy => 0,
            _ => 1,
        }
    }

    /// Base gas cost
    pub fn base_gas(&self) -> u64 {
        if self.is_push() || self.is_dup() || self.is_swap() {
            return 3;
        }

        match self {
            Self::Stop | Self::Invalid | Self::Return | Self::Revert => 0,
            Self::JumpDest => 1,
            Self::Add | Self::Sub | Self::Not | Self::Lt | Self::Gt
            | Self::Slt | Self::Sgt | Self::Eq | Self::IsZero
            | Self::And | Self::Or | Self::Xor | Self::Byte
            | Self::Shl | Self::Shr | Self::Sar | Self::Pop
            | Self::Pc | Self::MSize | Self::Gas => 3,
            Self::Mul | Self::Div | Self::SDiv | Self::Mod | Self::SMod
            | Self::SignExtend => 5,
            Self::AddMod | Self::MulMod => 8,
            Self::Jump => 8,
            Self::JumpI => 10,
            Self::Keccak256 => 30,
            Self::Address | Self::Origin | Self::Caller | Self::CallValue
            | Self::CallDataSize | Self::CodeSize | Self::GasPrice
            | Self::Coinbase | Self::Timestamp | Self::Number
            | Self::Difficulty | Self::GasLimit | Self::ChainId
            | Self::SelfBalance | Self::BaseFee | Self::ReturnDataSize => 2,
            Self::CallDataLoad | Self::MLoad | Self::MStore | Self::MStore8 => 3,
            Self::SLoad => 100,
            Self::SStore => 100,
            Self::Balance | Self::ExtCodeHash => 100,
            Self::ExtCodeSize => 100,
            Self::CallDataCopy | Self::CodeCopy | Self::ReturnDataCopy => 3,
            Self::ExtCodeCopy => 100,
            Self::BlockHash => 20,
            Self::Log0 => 375,
            Self::Log1 => 750,
            Self::Log2 => 1125,
            Self::Log3 => 1500,
            Self::Log4 => 1875,
            Self::Exp => 10,
            Self::Create => 32000,
            Self::Create2 => 32000,
            Self::Call | Self::CallCode | Self::DelegateCall | Self::StaticCall => 100,
            Self::SelfDestruct => 5000,
            _ => 3,
        }
    }

    /// Size of immediate data following opcode
    pub fn immediate_size(&self) -> usize {
        if self.is_push() {
            (*self as u8 - 0x60 + 1) as usize
        } else {
            0
        }
    }
}
