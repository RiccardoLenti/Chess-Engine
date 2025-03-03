use crate::Move;

#[derive(Clone, Copy)]
pub enum EntryType {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy)]
pub struct TTEntry {
    key: u64,
    pub depth: u8,
    pub score: i32,
    pub entry_type: EntryType,
    pub best_move: Option<Move>,
    age: u8,
}

pub struct TranspositionTable {
    table: Vec<Option<TTEntry>>,
    size: usize,
    age: u8,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<TTEntry>();
        let num_entries = (size_mb * 1024 * 1024) / entry_size;

        TranspositionTable {
            table: vec![None; num_entries],
            size: num_entries,
            age: 0,
        }
    }

    pub fn increment_age(&mut self) {
        self.age = self.age.wrapping_add(1);
    }

    fn index(&self, key: u64) -> usize {
        (key % self.size as u64) as usize
    }

    pub fn store(
        &mut self,
        key: u64,
        depth: u8,
        score: i32,
        entry_type: EntryType,
        best_move: Option<Move>,
    ) {
        let idx = self.index(key);

        // replacement strategy: replace if
        // 1) None
        // 2) new entry is deeper
        // 3) old entry is older
        if let Some(existing) = &self.table[idx] {
            if existing.depth > depth && existing.age == self.age {
                return;
            }
        }

        self.table[idx] = Some(TTEntry {
            key,
            depth,
            score,
            entry_type,
            best_move,
            age: self.age,
        });
    }

    #[inline]
    pub fn probe(&self, key: u64) -> Option<&TTEntry> {
        let idx = self.index(key);

        if let Some(entry) = &self.table[idx] {
            if entry.key == key {
                return Some(entry);
            }
        }

        None
    }
}
