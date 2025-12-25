//! Journal system for time-travel debugging

mod entry;
mod checkpoint;

pub use entry::{JournalEntry, InstructionJournal};
pub use checkpoint::{Checkpoint, StateSnapshot};

/// Journal managing instruction-level state deltas and checkpoints.
/// 
/// The journal enables O(1) single-step rewind and O(√N) arbitrary rewind
/// through periodic checkpointing.
#[derive(Clone)]
pub struct Journal {
    /// Per-instruction journal entries
    instructions: Vec<InstructionJournal>,
    /// Periodic full-state checkpoints
    checkpoints: Vec<Checkpoint>,
    /// Interval between checkpoints
    checkpoint_interval: usize,
    /// Maximum journal size before truncation
    max_size: usize,
}

impl Journal {
    /// Create a new journal
    pub fn new(checkpoint_interval: usize, max_size: usize) -> Self {
        Self {
            instructions: Vec::new(),
            checkpoints: Vec::new(),
            checkpoint_interval,
            max_size,
        }
    }

    /// Record an instruction's effects
    pub fn record(&mut self, insn: InstructionJournal) {
        self.instructions.push(insn);
        
        // Create checkpoint at interval
        if self.instructions.len() % self.checkpoint_interval == 0 {
            // Checkpoint creation is deferred to executor
        }
        
        // Truncate old entries if over limit
        if self.instructions.len() > self.max_size {
            let trim = self.max_size / 10;
            self.instructions.drain(0..trim);
            // Adjust checkpoint indices
            self.checkpoints.retain(|c| c.instruction_index >= trim);
            for c in &mut self.checkpoints {
                c.instruction_index -= trim;
            }
        }
    }

    /// Pop the most recent instruction journal (for rewind)
    pub fn pop(&mut self) -> Option<InstructionJournal> {
        self.instructions.pop()
    }

    /// Peek at the most recent instruction journal
    pub fn peek(&self) -> Option<&InstructionJournal> {
        self.instructions.last()
    }

    /// Get instruction at index
    pub fn get(&self, index: usize) -> Option<&InstructionJournal> {
        self.instructions.get(index)
    }

    /// Number of recorded instructions
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    /// Check if journal is empty
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    /// Clear the journal
    pub fn clear(&mut self) {
        self.instructions.clear();
        self.checkpoints.clear();
    }

    /// Add a checkpoint
    pub fn add_checkpoint(&mut self, checkpoint: Checkpoint) {
        self.checkpoints.push(checkpoint);
    }

    /// Find nearest checkpoint before instruction index
    pub fn find_checkpoint_before(&self, index: usize) -> Option<&Checkpoint> {
        self.checkpoints
            .iter()
            .rev()
            .find(|c| c.instruction_index < index)
    }

    /// Get all checkpoints
    pub fn checkpoints(&self) -> &[Checkpoint] {
        &self.checkpoints
    }

    /// Check if checkpoint should be created
    pub fn should_checkpoint(&self) -> bool {
        self.instructions.len() % self.checkpoint_interval == 0
    }

    /// Get checkpoint interval
    pub fn checkpoint_interval(&self) -> usize {
        self.checkpoint_interval
    }
}
