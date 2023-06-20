use chess::{Board, ChessMove};
use crate::core::evaluation::single_evaluation;
use crate::core::score::BoardEvaluation;
use crate::core::search::mt::search_mt;
use crate::core::search::mtd::{avg_bounds, mtd_iterative_deepening_search, mtd_search};
use crate::core::search::search_result::SearchResult;
use crate::core::search::transpositions::{EvalBound, TranspositionTable};
use crate::input::protocol_interpreter::CalculateOptions;

/// The code for implementing the MTD-BI search algorithm


pub fn mtdbi_iterative_deepening_search<T: SearchResult + Default + Clone>(
    board: &Board,
    transposition_table: &mut impl TranspositionTable,
    visited_boards: Vec<u64>,
    options: CalculateOptions,
) -> (T, u32, u32) { // (SearchResult, depth, selective_depth)
    mtd_iterative_deepening_search(
        board,
        transposition_table,
        visited_boards,
        options,
        determine_mtdbi_step,
    )
}


pub fn mtdbi_search<T: SearchResult + Default + Clone>(
    board: &Board,
    transposition_table: &mut impl TranspositionTable,
    visited_boards: Vec<u64>,
    depth: u32,
    start_point: BoardEvaluation,
    // selective_depth: u32,
) -> T {
    mtd_search(
        board,
        transposition_table,
        visited_boards,
        depth,
        start_point,
        determine_mtdbi_step,
    )
}

pub fn determine_mtdbi_step(_last_test_value: BoardEvaluation, lowerbound: BoardEvaluation, upperbound: BoardEvaluation) -> BoardEvaluation {
    avg_bounds(lowerbound, upperbound)
}
