use std::cmp::max;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use chess::Board;
use nohash::BuildNoHashHasher;
use crate::core::score::BoardEvaluation;
use crate::core::search::{SearchDepth, SearchInfo};
use crate::core::search::transpositions::TranspositionTable;

pub type HashTranspositionTable = HashMap<Board, SearchInfo, BuildNoHashHasher<u64>>;

impl TranspositionTable for HashTranspositionTable {
    fn update(&mut self, board: &Board, search_depth: SearchDepth, evaluation: BoardEvaluation) {
        let current_entry = self.entry(board.clone());

        match current_entry {
            Entry::Vacant(mut o) => {
                o.insert(SearchInfo {
                    depth_searched: search_depth,
                    evaluation
                });
            },
            Entry::Occupied(mut o) => {
                let search_info = o.get();

                if search_info.depth_searched != max(search_info.depth_searched, search_depth) {
                    o.insert(SearchInfo {
                        depth_searched: search_depth,
                        evaluation
                    });
                }
            },
        }
    }

    fn get_transposition(&mut self, board: &Board, minimal_search_depth: Option<SearchDepth>) -> Option<&SearchInfo> {
        let search_info = self.get(board)?;

        if search_info.depth_searched >= minimal_search_depth.unwrap_or(SearchDepth::Single) {
            return Some(search_info);
        }
        None
    }
}
