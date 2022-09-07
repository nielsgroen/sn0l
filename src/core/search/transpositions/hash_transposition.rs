// use std::collections::hash_map::Entry;
// use std::collections::HashMap;
// use chess::Board;
// use nohash::BuildNoHashHasher;
// use crate::core::score::BoardEvaluation;
// use crate::core::search::{SearchDepth, SearchInfo};
//
// pub type TranspositionTable = HashMap<Board, SearchInfo, BuildNoHashHasher<u64>>;
//
// pub fn update_transposition(
//     mut transposition_table: &mut TranspositionTable,
//     board: &Board,
//     search_depth: SearchDepth,
//     evaluation: BoardEvaluation,
// ) {
//     let current_entry = transposition_table
//         .entry(board.clone());
//
//     match current_entry {
//         Entry::Vacant(mut o) => {
//             o.insert(SearchInfo {
//                 depth_searched: search_depth,
//                 evaluation
//             });
//         },
//         Entry::Occupied(mut o) => {
//             let search_info = o.get();
//
//             if search_info.depth_searched != max(search_info.depth_searched, search_depth) {
//                 o.insert(SearchInfo {
//                     depth_searched: search_depth,
//                     evaluation
//                 });
//             }
//         },
//     }
// }
//
// pub fn get_transposition<'a>(
//     mut transposition_table: &'a mut TranspositionTable,
//     board: &Board,
//     minimal_search_depth: SearchDepth,
// ) -> Option<&'a SearchInfo> {
//     let search_info = transposition_table.get(board)?;
//
//     if search_info.depth_searched.clone() >= minimal_search_depth {
//         return Some(search_info);
//     }
//     None
// }
