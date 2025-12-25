//! Bytecode decoding and disassembly

use crate::executor::Opcode;

/// Decoded instruction with metadata
#[derive(Clone, Debug)]
pub struct DecodedInstruction {
    pub offset: usize,
    pub opcode: Opcode,
    pub immediate: Option<Vec<u8>>,
    pub mnemonic: String,
}

/// Decode a single instruction at offset
pub fn decode_instruction(bytecode: &[u8], offset: usize) -> Option<DecodedInstruction> {
    if offset >= bytecode.len() {
        return None;
    }

    let opcode_byte = bytecode[offset];
    let opcode = Opcode::from_u8(opcode_byte)?;
    let imm_size = opcode.immediate_size();
    
    let immediate = if imm_size > 0 {
        let start = offset + 1;
        let end = (start + imm_size).min(bytecode.len());
        Some(bytecode[start..end].to_vec())
    } else {
        None
    };

    let mnemonic = format_mnemonic(opcode, &immediate);

    Some(DecodedInstruction {
        offset,
        opcode,
        immediate,
        mnemonic,
    })
}

/// Format opcode as mnemonic string
fn format_mnemonic(opcode: Opcode, immediate: &Option<Vec<u8>>) -> String {
    let byte = opcode as u8;
    
    // Handle PUSH opcodes
    if opcode.is_push() {
        let n = byte - 0x60 + 1;
        if let Some(imm) = immediate {
            let hex = imm.iter().map(|b| format!("{:02x}", b)).collect::<String>();
            return format!("PUSH{} 0x{}", n, hex);
        }
        return format!("PUSH{}", n);
    }
    
    // Handle DUP opcodes
    if opcode.is_dup() {
        let n = byte - 0x80 + 1;
        return format!("DUP{}", n);
    }
    
    // Handle SWAP opcodes
    if opcode.is_swap() {
        let n = byte - 0x90 + 1;
        return format!("SWAP{}", n);
    }
    
    // Handle LOG opcodes
    if opcode.is_log() {
        let n = byte - 0xA0;
        return format!("LOG{}", n);
    }

    let name = match opcode {
        Opcode::Stop => "STOP",
        Opcode::Add => "ADD",
        Opcode::Mul => "MUL",
        Opcode::Sub => "SUB",
        Opcode::Div => "DIV",
        Opcode::SDiv => "SDIV",
        Opcode::Mod => "MOD",
        Opcode::SMod => "SMOD",
        Opcode::AddMod => "ADDMOD",
        Opcode::MulMod => "MULMOD",
        Opcode::Exp => "EXP",
        Opcode::SignExtend => "SIGNEXTEND",
        Opcode::Lt => "LT",
        Opcode::Gt => "GT",
        Opcode::Slt => "SLT",
        Opcode::Sgt => "SGT",
        Opcode::Eq => "EQ",
        Opcode::IsZero => "ISZERO",
        Opcode::And => "AND",
        Opcode::Or => "OR",
        Opcode::Xor => "XOR",
        Opcode::Not => "NOT",
        Opcode::Byte => "BYTE",
        Opcode::Shl => "SHL",
        Opcode::Shr => "SHR",
        Opcode::Sar => "SAR",
        Opcode::Keccak256 => "KECCAK256",
        Opcode::Pop => "POP",
        Opcode::MLoad => "MLOAD",
        Opcode::MStore => "MSTORE",
        Opcode::MStore8 => "MSTORE8",
        Opcode::SLoad => "SLOAD",
        Opcode::SStore => "SSTORE",
        Opcode::Jump => "JUMP",
        Opcode::JumpI => "JUMPI",
        Opcode::Pc => "PC",
        Opcode::MSize => "MSIZE",
        Opcode::Gas => "GAS",
        Opcode::JumpDest => "JUMPDEST",
        Opcode::Return => "RETURN",
        Opcode::Revert => "REVERT",
        Opcode::Invalid => "INVALID",
        Opcode::Call => "CALL",
        Opcode::CallCode => "CALLCODE",
        Opcode::DelegateCall => "DELEGATECALL",
        Opcode::StaticCall => "STATICCALL",
        Opcode::Create => "CREATE",
        Opcode::Create2 => "CREATE2",
        Opcode::SelfDestruct => "SELFDESTRUCT",
        _ => "UNKNOWN",
    };

    name.to_string()
}

/// Disassemble bytecode into list of instructions
pub fn disassemble(bytecode: &[u8]) -> Vec<DecodedInstruction> {
    let mut instructions = Vec::new();
    let mut offset = 0;

    while offset < bytecode.len() {
        if let Some(insn) = decode_instruction(bytecode, offset) {
            let next_offset = offset + 1 + insn.opcode.immediate_size();
            instructions.push(insn);
            offset = next_offset;
        } else {
            offset += 1;
        }
    }

    instructions
}

/// Print disassembly to string
pub fn disassemble_to_string(bytecode: &[u8]) -> String {
    let instructions = disassemble(bytecode);
    let mut output = String::new();
    
    for insn in instructions {
        output.push_str(&format!("{:04x}: {}\n", insn.offset, insn.mnemonic));
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassemble() {
        let bytecode = vec![0x60, 0x42, 0x60, 0x00, 0x52, 0x00];
        let instructions = disassemble(&bytecode);
        
        assert_eq!(instructions.len(), 4);
        assert_eq!(instructions[0].mnemonic, "PUSH1 0x42");
        assert_eq!(instructions[1].mnemonic, "PUSH1 0x00");
        assert_eq!(instructions[2].mnemonic, "MSTORE");
        assert_eq!(instructions[3].mnemonic, "STOP");
    }
}
