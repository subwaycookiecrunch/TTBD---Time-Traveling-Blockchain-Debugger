# TTBD

Reversible VM for debugging smart contract bytecode. You can step through execution forward and backward, inspect any historical state, and it's all deterministic.

I built this because I got tired of staring at static transaction traces trying to figure out what went wrong. Existing tools show you a log of what happened — this actually lets you scrub back and forth through execution like a video player.

## how it works

The VM journals every state mutation (stack pushes/pops, memory writes, storage writes, PC changes, gas) as delta entries instead of taking full snapshots. Each opcode has an inverse operation, so rewinding just means applying the journal entries in reverse. Periodic checkpoints give you O(√N) jumps to distant states without blowing up memory.

It's not EVM-compatible (uses its own bytecode format), and it's ~100x slower than a real VM. This is a debugging tool, not a production runtime.

## what's in here

- `core/` — U256 type, addresses, block context, error types
- `vm/` — stack, memory (page-based), persistent storage, call frames, main VM struct
- `journal/` — delta entries, instruction journals, checkpoints
- `executor/` — forward interpreter with journaling, reverse execution, opcode definitions
- `debugger/` — time-travel API, breakpoints, state inspection
- `bytecode/` — decoder and disassembler

## building

```bash
cmake.. just kidding, it's rust

cargo build
cargo test
```

Needs a recent nightly or stable with edition 2021 support.

## quick example

```rust
use ttbd::{Vm, TimeTravel, BlockContext};

let bytecode = vec![0x60, 0x42, 0x60, 0x01, 0x01, 0x00];
let mut vm = Vm::new(bytecode, 100_000, BlockContext::default());
let mut dbg = TimeTravel::new(vm);

// step forward a few times
dbg.step_forward().unwrap();
dbg.step_forward().unwrap();

// check the stack
println!("{:?}", dbg.inspect_stack());

// go back
dbg.step_backward().unwrap();
```

See `examples/demo.rs` for the full thing.

## known issues

- `is_lock_free()` returns false on MSVC for some atomic configs. Still works, just not technically lock-free there.
- Hash map is insert-only, no delete, no resize. Fine for order-id style lookups but annoying otherwise.
- The zero-copy arena is a bump allocator that panics when full. Need to reset it manually for long runs.
- U256 mul/div truncates to u64 internally. Good enough for testing but not for real math.

## todo

- hazard pointers or epoch reclamation for safe memory management
- hash map deletion
- multi-threaded matching
- better arena allocator

## license

MIT
