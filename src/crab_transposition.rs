use chess::{ChessMove, Square};

#[derive(Clone, Copy, PartialEq, PartialOrd)]

pub enum NodeType {
    UpperBound,
    Exact,
    LowerBound,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Transposition {
    pub depth: usize,
    pub ply: usize,
    pub score: i32,
    pub node_type: NodeType,
    pub best_move: ChessMove,
}

impl Transposition {
    fn empty() -> Self {
        Self {
            depth: 0,
            ply: 0,
            score: 0,
            node_type: NodeType::Exact,
            best_move: ChessMove::new(Square::A1, Square::A1, None),
        }
    }
}

pub struct TranspositionTable(chess::CacheTable<Transposition>);

impl TranspositionTable {
    pub fn new() -> Self {
        Self(
            // 2^18 is 262,144
            // at ~40b each that's around 10.5 megabytes
            chess::CacheTable::new(1 << 19, Transposition::empty()),
        )
    }

    pub fn insert(&mut self, key: u64, value: Transposition) {
        self.0.add(key, value);
    }

    pub fn get(&self, key: u64, depth: usize) -> Option<Transposition> {
        if let Some(val) = self.0.get(key) {
            if val.depth < depth {
                return None;
            }
        } else {
            return None;
        }
        self.0.get(key)
    }
}
