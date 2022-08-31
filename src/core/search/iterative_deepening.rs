use std::num::NonZeroU32;
use chess::{Board, ChessMove};
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::alpha_beta::search_depth_pruned;

use crate::core::search::transposition::TranspositionTable;
use crate::input::protocol_interpreter::CalculateOptions;


pub fn iterative_deepening_search(
    board: &Board,
    transposition_table: &mut TranspositionTable,
    options: CalculateOptions,
) -> (ChessMove, BoardEvaluation, u32) { // (best_move, eval, nodes)
    let mut max_search_depth: u32 = 1;

    match options {
        CalculateOptions::Depth(x) => max_search_depth = x,
        CalculateOptions::Infinite => max_search_depth = 5, // todo
        _ => panic!("unsupported iterative deepening calculate options"),
    }

    // for max_depth in 1..max_search_depth {
    //
    // }

    // TODO: remove this
    search_depth_pruned(
        board,
        transposition_table,
        max_search_depth,
        Some(12), // todo
    )
}