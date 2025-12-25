# TTBD: Time-Traveling Blockchain Debugger

> **⚠️ This is not for the faint of heart.**

A research-grade **reversible virtual machine** for debugging smart contract 
bytecode with **instruction-level time travel**. Execute forward, rewind 
backward, inspect any historical state—deterministically.

## Why This Exists

Existing debuggers show you *traces* corpses of past executions. TTBD gives 
you a **living VM** that you can scrub back and forth like a video player.

## Core Innovation

- **Δ-Journaling**: Log only state *changes*, not snapshots
- **Reversible Opcodes**: Every instruction has a mathematical inverse
- **Hybrid Checkpointing**: O(√N) rewind with bounded memory
- **Deterministic Replay**: Bit-exact execution, every time

## Quick Start

```rust
use ttbd::{Vm, TimeTravel, BlockContext};

// Create VM with bytecode
let bytecode = vec![0x60, 0x42, 0x60, 0x01, 0x01, 0x00]; // PUSH 0x42, PUSH 0x01, ADD, STOP
let mut vm = Vm::new(bytecode, 100_000, BlockContext::default());

// Wrap in debugger
let mut debugger = TimeTravel::new(vm);

// Step forward
debugger.step_forward().unwrap();
debugger.step_forward().unwrap();

// Inspect stack
let stack = debugger.inspect_stack();
println!("Stack: {:?}", stack);

// Rewind!
debugger.step_backward().unwrap();

// Stack is now restored to previous state
```

## Architecture

```
[Bytecode] → [Interpreter] → [Journal] → [Checkpoints]
                  ↓              ↑
             [Time-Travel Debugger API]
```

### Module Structure

- `core/` - Primitive types (U256, Address), error handling
- `vm/` - State containers (Stack, Memory, Storage, CallFrame)
- `journal/` - Mutation logging, delta encoding, checkpointing
- `executor/` - Forward/backward interpretation, opcodes
- `debugger/` - Breakpoints, inspection, run controls
- `bytecode/` - Decoding and disassembly

## Not For You If...

- You want EVM compatibility (we use custom bytecode)
- You need production performance (this is ~100x slower than native)
- You can't reason about `unsafe` Rust

## For You If...

- You debug smart contracts at the opcode level
- You need to understand *exactly* what happened
- You want to reverse execution, not just replay it

## Testing

```bash
cargo test
```

## Known Limitations

- **Single-threaded**: No parallel execution for determinism
- **No JIT**: Pure interpreter for reversibility
- **Memory pages remain allocated**: Rewind restores values but doesn't deallocate pages
- **Simplified 256-bit ops**: Some U256 operations use truncated u64 math

## Status

Research prototype. 13/14 tests passing. Breaks in interesting ways.

---

*"The best debugger is one that lets you undo your mistakes including running 
the wrong opcode."*
