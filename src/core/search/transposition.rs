use std::cmp::max;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use chess::Board;
use nohash::BuildNoHashHasher;
use crate::core::score::BoardEvaluation;
use crate::core::search::{SearchDepth, SearchInfo};

pub type TranspositionTable = HashMap<Board, SearchInfo, BuildNoHashHasher<u64>>;

pub fn update_transpositions(
    mut transposition_table: &mut TranspositionTable,
    board: &Board,
    search_depth: SearchDepth,
    evaluation: BoardEvaluation,
) {
    let current_entry = transposition_table
        .entry(board.clone());

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
