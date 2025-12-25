//! Linear memory for the TTBD virtual machine

use crate::core::U256;

/// Page size for memory allocation (4KB)
const PAGE_SIZE: usize = 4096;

/// Linear byte-addressable memory with lazy page allocation.
/// 
/// Memory grows on demand and uses copy-on-write semantics for efficient
/// snapshotting. All writes are journaled by the executor for reversibility.
pub struct Memory {
    /// Pages are allocated lazily; None means zero-filled page
    pages: Vec<Option<Box<[u8; PAGE_SIZE]>>>,
    /// Current logical size (high water mark)
    size: usize,
}

impl Memory {
    /// Create new empty memory
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            size: 0,
        }
    }

    /// Load a 256-bit word from memory
    pub fn load(&mut self, offset: usize) -> U256 {
        self.ensure_size(offset + 32);
        let mut bytes = [0u8; 32];
        self.read_slice(offset, &mut bytes);
        U256::from_be_bytes(bytes)
    }

    /// Load a single byte from memory
    pub fn load_byte(&mut self, offset: usize) -> u8 {
        self.ensure_size(offset + 1);
        self.get_byte(offset)
    }

    /// Store a 256-bit word to memory, returning old bytes for journaling
    pub fn store(&mut self, offset: usize, value: U256) -> Vec<u8> {
        self.ensure_size(offset + 32);
        let old = self.read_range(offset, 32);
        self.write_slice_internal(offset, &value.to_be_bytes());
        old
    }

    /// Store a single byte to memory, returning old byte
    pub fn store_byte(&mut self, offset: usize, value: u8) -> u8 {
        self.ensure_size(offset + 1);
        let old = self.get_byte(offset);
        self.set_byte(offset, value);
        old
    }

    /// Store arbitrary bytes, returning old bytes for journaling
    pub fn store_bytes(&mut self, offset: usize, data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }
        self.ensure_size(offset + data.len());
        let old = self.read_range(offset, data.len());
        self.write_slice_internal(offset, data);
        old
    }

    /// Restore bytes from journal (for rewinding)
    pub fn restore_bytes(&mut self, offset: usize, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        self.write_slice_internal(offset, data);
    }

    /// Current memory size in bytes
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Ensure memory is at least min_size bytes
    fn ensure_size(&mut self, min_size: usize) {
        if min_size <= self.size {
            return;
        }
        let pages_needed = (min_size + PAGE_SIZE - 1) / PAGE_SIZE;
        while self.pages.len() < pages_needed {
            self.pages.push(None); // Lazy zero-filled pages
        }
        self.size = min_size;
    }

    /// Get a byte at offset (internal, may not exist)
    fn get_byte(&self, offset: usize) -> u8 {
        let page_idx = offset / PAGE_SIZE;
        let page_offset = offset % PAGE_SIZE;
        match self.pages.get(page_idx) {
            Some(Some(page)) => page[page_offset],
            _ => 0, // Unallocated pages are zero-filled
        }
    }

    /// Peek a byte at offset without modifying memory state
    pub fn peek_byte(&self, offset: usize) -> u8 {
        self.get_byte(offset)
    }

    /// Set a byte at offset
    fn set_byte(&mut self, offset: usize, value: u8) {
        let page_idx = offset / PAGE_SIZE;
        let page_offset = offset % PAGE_SIZE;
        
        // Ensure page exists
        if page_idx >= self.pages.len() {
            self.pages.resize(page_idx + 1, None);
        }
        
        // Allocate page if needed
        if self.pages[page_idx].is_none() {
            self.pages[page_idx] = Some(Box::new([0u8; PAGE_SIZE]));
        }
        
        if let Some(ref mut page) = self.pages[page_idx] {
            page[page_offset] = value;
        }
    }

    /// Read a slice of bytes into dst
    fn read_slice(&self, offset: usize, dst: &mut [u8]) {
        for (i, byte) in dst.iter_mut().enumerate() {
            *byte = self.get_byte(offset + i);
        }
    }

    /// Read a range of bytes
    fn read_range(&self, offset: usize, len: usize) -> Vec<u8> {
        let mut result = vec![0u8; len];
        self.read_slice(offset, &mut result);
        result
    }

    /// Write bytes to memory internally
    fn write_slice_internal(&mut self, offset: usize, src: &[u8]) {
        for (i, &byte) in src.iter().enumerate() {
            self.set_byte(offset + i, byte);
        }
    }

    /// Create a snapshot of memory contents
    pub fn snapshot(&self) -> Vec<u8> {
        let mut result = vec![0u8; self.size];
        self.read_slice(0, &mut result);
        result
    }

    /// Restore from a snapshot
    pub fn restore_from(&mut self, snapshot: &[u8]) {
        self.pages.clear();
        self.size = 0;
        if !snapshot.is_empty() {
            self.store_bytes(0, snapshot);
        }
    }

    /// Clear all memory
    pub fn clear(&mut self) {
        self.pages.clear();
        self.size = 0;
    }

    /// Calculate gas cost for memory expansion
    pub fn expansion_cost(current_size: usize, new_size: usize) -> u64 {
        if new_size <= current_size {
            return 0;
        }
        let new_words = (new_size + 31) / 32;
        let old_words = (current_size + 31) / 32;
        let new_cost = (new_words * new_words) / 512 + 3 * new_words;
        let old_cost = (old_words * old_words) / 512 + 3 * old_words;
        (new_cost - old_cost) as u64
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Memory {
    fn clone(&self) -> Self {
        let mut new_mem = Self::new();
        new_mem.pages = self.pages.clone();
        new_mem.size = self.size;
        new_mem
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_load() {
        let mut mem = Memory::new();
        let value = U256::from(0xDEADBEEFu64);
        mem.store(0, value);
        assert_eq!(mem.load(0), value);
    }

    #[test]
    fn test_byte_operations() {
        let mut mem = Memory::new();
        mem.store_byte(100, 0x42);
        assert_eq!(mem.load_byte(100), 0x42);
        assert_eq!(mem.load_byte(99), 0x00); // Unwritten = zero
    }

    #[test]
    fn test_memory_growth() {
        let mut mem = Memory::new();
        assert_eq!(mem.size(), 0);
        mem.load(1000);
        assert!(mem.size() >= 1032); // 1000 + 32
    }

    #[test]
    fn test_snapshot_restore() {
        let mut mem = Memory::new();
        mem.store_byte(0, 1);
        mem.store_byte(1, 2);
        mem.store_byte(2, 3);
        
        let snap = mem.snapshot();
        mem.clear();
        mem.restore_from(&snap);
        
        assert_eq!(mem.load_byte(0), 1);
        assert_eq!(mem.load_byte(1), 2);
        assert_eq!(mem.load_byte(2), 3);
    }
}
