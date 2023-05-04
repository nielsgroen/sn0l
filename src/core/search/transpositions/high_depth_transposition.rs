use std::cmp::max;
use std::collections::hash_map::Entry;
use chess::{Board, ChessMove};
use crate::core::search::{SearchDepth, SearchInfo};
use crate::core::search::transpositions::hash_transposition::HashTranspositionTable;
use crate::core::search::transpositions::{EvalBound, TranspositionTable};

#[derive(Clone, Debug)]
pub struct HighDepthTranspositionTable {
    pub minimal_depth: SearchDepth,
    transposition_table: HashTranspositionTable,
}

impl HighDepthTranspositionTable {
    pub fn new(minimal_depth: SearchDepth) -> Self {
        HighDepthTranspositionTable {
            minimal_depth,
            transposition_table: HashTranspositionTable::default(),
        }
    }
}

impl TranspositionTable for HighDepthTranspositionTable {
    fn update(&mut self, board: &Board, search_depth: SearchDepth, evaluation: EvalBound, best_move: ChessMove, prime_variation: Option<Vec<ChessMove>>) {
        // Only keep entries of sufficient depth
        if search_depth < self.minimal_depth {
            return;
        }

        let current_entry = self.transposition_table.entry(board.clone());

        match current_entry {
            Entry::Vacant(mut o) => {
                o.insert(SearchInfo {
                    depth_searched: search_depth,
                    evaluation,
                    best_move,
                    prime_variation,
                });
            },
            Entry::Occupied(mut o) => {
                let search_info = o.get();

                if search_info.depth_searched != max(search_info.depth_searched, search_depth) {
                    o.insert(SearchInfo {
                        depth_searched: search_depth,
                        evaluation,
                        best_move,
                        prime_variation,
                    });
                }
            },
        }
    }

    fn get_transposition(&mut self, board: &Board, minimal_search_depth: Option<SearchDepth>) -> Option<&SearchInfo> {
        let search_info = self.transposition_table.get(board)?;

        if search_info.depth_searched >= minimal_search_depth.unwrap_or(SearchDepth::Single) {
            return Some(search_info);
        }
        None
    }
}