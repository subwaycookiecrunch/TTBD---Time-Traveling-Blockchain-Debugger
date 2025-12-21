use crate::core::{Address, U256};

#[derive(Clone)]
pub struct CallFrame {
    pub pc: usize,
    pub code: Vec<u8>,
    pub address: Address,
    pub caller: Address,
    pub value: U256,
    pub calldata: Vec<u8>,
    pub gas: u64,
    pub is_static: bool,
    pub return_offset: usize,
    pub return_size: usize,
}

impl CallFrame {
    pub fn new(
        code: Vec<u8>,
        address: Address,
        caller: Address,
        value: U256,
        calldata: Vec<u8>,
        gas: u64,
        is_static: bool,
    ) -> Self {
        Self {
            pc: 0,
            code,
            address,
            caller,
            value,
            calldata,
            gas,
            is_static,
            return_offset: 0,
            return_size: 0,
        }
    }

    pub fn snapshot(&self) -> CallFrameSnapshot {
        CallFrameSnapshot {
            pc: self.pc,
            gas: self.gas,
            address: self.address,
            caller: self.caller,
            value: self.value,
            is_static: self.is_static,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CallFrameSnapshot {
    pub pc: usize,
    pub gas: u64,
    pub address: Address,
    pub caller: Address,
    pub value: U256,
    pub is_static: bool,
}

pub const MAX_CALL_DEPTH: usize = 1024;
