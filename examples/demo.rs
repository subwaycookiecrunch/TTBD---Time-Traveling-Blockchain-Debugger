//! Time-Traveling Blockchain Debugger Example
//!
//! This example demonstrates the core time-travel debugging capabilities:
//! - Forward execution with full journaling
//! - Backward execution (rewind)
//! - State inspection at any point

use ttbd::core::BlockContext;
use ttbd::vm::Vm;
use ttbd::debugger::TimeTravel;
use ttbd::executor::StepResult;
use ttbd::bytecode::disassemble;

fn main() {
    println!("=== Time-Traveling Blockchain Debugger Demo ===\n");

    // Sample bytecode: PUSH1 10, PUSH1 20, ADD, PUSH1 0, MSTORE, STOP
    let bytecode = vec![
        0x60, 0x0A, // PUSH1 10
        0x60, 0x14, // PUSH1 20
        0x01,       // ADD
        0x60, 0x00, // PUSH1 0
        0x52,       // MSTORE
        0x00,       // STOP
    ];

    println!("Bytecode Disassembly:");
    for insn in disassemble(&bytecode) {
        println!("  {:04x}: {}", insn.offset, insn.mnemonic);
    }
    println!();

    // Create VM with 100k gas
    let vm = Vm::new(bytecode, 100_000, BlockContext::default());
    let mut debugger = TimeTravel::new(vm);

    println!("Step-by-step execution:\n");

    // Execute forward step by step
    let mut step = 0;
    loop {
        let pc = debugger.inspect_pc();
        let gas = debugger.inspect_gas();
        let stack = debugger.inspect_stack();

        println!("Step {}: PC={:04x} Gas={} Stack={:?}", 
            step, pc, gas, 
            stack.iter().map(|v| v.as_u64()).collect::<Vec<_>>()
        );

        match debugger.step_forward() {
            Ok(StepResult::Halted { reason }) => {
                println!("  -> HALTED: {:?}\n", reason);
                break;
            }
            Ok(StepResult::Executed { opcode, gas_used }) => {
                println!("  -> Executed {:?}, cost {} gas", opcode, gas_used);
                step += 1;
            }
            Err(e) => {
                println!("  -> ERROR: {:?}\n", e);
                break;
            }
            _ => {}
        }
    }

    println!("Total steps recorded in journal: {}", debugger.history_len());
    println!("\n=== Now rewinding... ===\n");

    // Rewind back to the beginning
    while debugger.history_len() > 0 {
        let pc_before = debugger.inspect_pc();
        debugger.step_backward().unwrap();
        let pc_after = debugger.inspect_pc();
        println!("Rewound: PC {:04x} -> {:04x}", pc_before, pc_after);
    }

    println!("\n=== State restored to beginning ===");
    println!("PC: {:04x}", debugger.inspect_pc());
    println!("Stack: {:?}", debugger.inspect_stack());
    println!("Gas: {}", debugger.inspect_gas());

    println!("\n=== Re-executing forward ===\n");
    
    // Execute again to show determinism
    loop {
        match debugger.step_forward() {
            Ok(StepResult::Halted { .. }) => break,
            Ok(StepResult::Executed { opcode, .. }) => {
                println!("Executed: {:?}", opcode);
            }
            _ => break,
        }
    }

    println!("\nFinal state:");
    println!("Stack: {:?}", debugger.inspect_stack().iter().map(|v| v.as_u64()).collect::<Vec<_>>());
    println!("Memory at 0: {:?}", debugger.inspect_memory(0, 32));
    
    println!("\n=== Demo Complete ===");
}
