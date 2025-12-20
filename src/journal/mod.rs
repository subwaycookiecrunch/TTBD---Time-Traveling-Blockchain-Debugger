mod checkpoint;
mod entry;

pub use checkpoint::{Checkpoint, StateSnapshot};
pub use entry::{InstructionJournal, JournalEntry};

// Tracks per-instruction state deltas + periodic checkpoints.
// Single-step rewind is O(1), jumping to a distant point uses checkpoints.
#[derive(Clone)]
pub struct Journal {
    instructions: Vec<InstructionJournal>,
    checkpoints: Vec<Checkpoint>,
    checkpoint_interval: usize,
    max_size: usize,
}

impl Journal {
    pub fn new(checkpoint_interval: usize, max_size: usize) -> Self {
        Self {
            instructions: Vec::new(),
            checkpoints: Vec::new(),
            checkpoint_interval,
            max_size,
        }
    }

    pub fn record(&mut self, insn: InstructionJournal) {
        self.instructions.push(insn);

        if self.instructions.len() % self.checkpoint_interval == 0 {
            // actual checkpoint creation happens in the executor
        }

        // drop old entries if we're over budget
        if self.instructions.len() > self.max_size {
            let trim = self.max_size / 10;
            self.instructions.drain(0..trim);
            self.checkpoints.retain(|c| c.instruction_index >= trim);
            for c in &mut self.checkpoints {
                c.instruction_index -= trim;
            }
        }
    }

    pub fn pop(&mut self) -> Option<InstructionJournal> {
        self.instructions.pop()
    }

    pub fn peek(&self) -> Option<&InstructionJournal> {
        self.instructions.last()
    }

    pub fn get(&self, index: usize) -> Option<&InstructionJournal> {
        self.instructions.get(index)
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    pub fn clear(&mut self) {
        self.instructions.clear();
        self.checkpoints.clear();
    }

    pub fn add_checkpoint(&mut self, checkpoint: Checkpoint) {
        self.checkpoints.push(checkpoint);
    }

    pub fn find_checkpoint_before(&self, index: usize) -> Option<&Checkpoint> {
        self.checkpoints
            .iter()
            .rev()
            .find(|c| c.instruction_index < index)
    }

    pub fn checkpoints(&self) -> &[Checkpoint] {
        &self.checkpoints
    }

    pub fn should_checkpoint(&self) -> bool {
        self.instructions.len() % self.checkpoint_interval == 0
    }

    pub fn checkpoint_interval(&self) -> usize {
        self.checkpoint_interval
    }
}
