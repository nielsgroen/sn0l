use chess::{Board, ChessMove};
use crate::core::score::BoardEvaluation;
use crate::core::search::transposition::TranspositionTable;

/// The module for alpha-beta search;


pub fn search_depth_pruned(
    board: &Board,
    transposition_table: &mut TranspositionTable,
    depth: u32
) -> (ChessMove, BoardEvaluation) {
    todo!()
}

pub fn search_alpha_beta(
    board: &Board,
    transposition_table: &mut TranspositionTable,
    alpha: i64,
    beta: i64,
    current_depth: u32,
    max_depth: u32
) -> (ChessMove, i64, i64) { // (_, alpha, beta)

    // TODO: use `move_ordering::order_captures` and `move_ordering::order_non_captures`

    todo!()
}