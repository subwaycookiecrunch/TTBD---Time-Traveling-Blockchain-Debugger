use ttbd::bytecode::disassemble;
use ttbd::core::BlockContext;
use ttbd::debugger::TimeTravel;
use ttbd::executor::StepResult;
use ttbd::vm::Vm;

fn main() {
    println!("TTBD demo\n");

    // PUSH1 10, PUSH1 20, ADD, PUSH1 0, MSTORE, STOP
    let bytecode = vec![
        0x60, 0x0A, // PUSH1 10
        0x60, 0x14, // PUSH1 20
        0x01, // ADD
        0x60, 0x00, // PUSH1 0
        0x52, // MSTORE
        0x00, // STOP
    ];

    println!("disassembly:");
    for insn in disassemble(&bytecode) {
        println!("  {:04x}: {}", insn.offset, insn.mnemonic);
    }
    println!();

    let vm = Vm::new(bytecode, 100_000, BlockContext::default());
    let mut debugger = TimeTravel::new(vm);

    println!("stepping forward:\n");

    let mut step = 0;
    loop {
        let pc = debugger.inspect_pc();
        let gas = debugger.inspect_gas();
        let stack = debugger.inspect_stack();

        println!(
            "step {}: pc={:04x} gas={} stack={:?}",
            step,
            pc,
            gas,
            stack.iter().map(|v| v.as_u64()).collect::<Vec<_>>()
        );

        match debugger.step_forward() {
            Ok(StepResult::Halted { reason }) => {
                println!("  -> halted: {:?}\n", reason);
                break;
            }
            Ok(StepResult::Executed { opcode, gas_used }) => {
                println!("  -> {:?}, {} gas", opcode, gas_used);
                step += 1;
            }
            Err(e) => {
                println!("  -> error: {:?}\n", e);
                break;
            }
            _ => {}
        }
    }

    println!("journal has {} entries", debugger.history_len());
    println!("\nrewinding...\n");

    while debugger.history_len() > 0 {
        let pc_before = debugger.inspect_pc();
        debugger.step_backward().unwrap();
        let pc_after = debugger.inspect_pc();
        println!("  {:04x} -> {:04x}", pc_before, pc_after);
    }

    println!("\nback to start");
    println!("pc: {:04x}", debugger.inspect_pc());
    println!("stack: {:?}", debugger.inspect_stack());
    println!("gas: {}", debugger.inspect_gas());

    println!("\nre-executing...\n");

    loop {
        match debugger.step_forward() {
            Ok(StepResult::Halted { .. }) => break,
            Ok(StepResult::Executed { opcode, .. }) => {
                println!("  {:?}", opcode);
            }
            _ => break,
        }
    }

    println!("\nfinal state:");
    println!(
        "stack: {:?}",
        debugger
            .inspect_stack()
            .iter()
            .map(|v| v.as_u64())
            .collect::<Vec<_>>()
    );
    println!("mem[0..32]: {:?}", debugger.inspect_memory(0, 32));

    println!("\ndone.");
}
