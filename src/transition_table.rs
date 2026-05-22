use shakmaty::{Move};
use shakmaty::zobrist::Zobrist64;

#[derive(Clone, Copy, PartialEq)]
pub enum EntryType {
    Exact,      // exact value
    LowerBound, // alpha cutoff
    UpperBound, // beta cutoff
}

#[derive(Clone, Copy)]
pub struct TTEntry {
    pub hash: Zobrist64,
    pub depth: i32,
    pub score: i32,
    pub entry_type: EntryType,
    pub best_move: Option<Move>,
}

pub struct TranspositionTable {
    entries: Vec<Option<TTEntry>>,
    size: usize,
}

impl TranspositionTable {
    pub fn new(num_entries: usize) -> Self {
        TranspositionTable {
            entries: vec![None; num_entries],
            size: num_entries,
        }
    }

    pub fn get(&self, hash: Zobrist64) -> Option<&TTEntry> {
        let idx = (hash.0 as usize) % self.size;
        self.entries[idx].as_ref().filter(|e| e.hash == hash)
    }

    pub fn insert(&mut self, entry: TTEntry) {
        let idx = (entry.hash.0 as usize) % self.size;
        self.entries[idx] = Some(entry);
    }
}