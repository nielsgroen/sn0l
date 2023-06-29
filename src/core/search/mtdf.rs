use std::cmp::{max, min};
use std::time::Instant;
use chess::{Board, ChessMove};
use crate::analysis::database::rows::{MTSearchRow, PositionSearchRow};
use crate::core::evaluation::single_evaluation;
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::iterative_deepening::{is_still_searching, log_info_search_results};
use crate::core::search::mt::search_mt;
use crate::core::search::mtd::{avg_bounds, mtd_iterative_deepening_search, mtd_search};
use crate::core::search::search_result::debug_search_result::DebugSearchResult;
use crate::core::search::search_result::SearchResult;
use crate::core::search::transpositions::{EvalBound, TranspositionTable};
use crate::input::protocol_interpreter::CalculateOptions;

/// The code implementing the MTD-F search algorithm

const MTDF_STEP_SIZE: Centipawns = Centipawns::new(30);


pub fn mtdf_iterative_deepening_search<T: SearchResult + Default + Clone, L>(
    board: &Board,
    // transposition_table: &mut impl TranspositionTable,
    transposition_table: &mut Box<dyn TranspositionTable>,
    visited_boards: Vec<u64>,
    options: CalculateOptions,
    search_logging: L,
) -> (T, u32, u32) where
    L: Fn(PositionSearchRow, Vec<MTSearchRow>) { // (SearchResult, depth, selective_depth)
    mtd_iterative_deepening_search(
        board,
        transposition_table,
        visited_boards,
        options,
        determine_mtdf_step,
        search_logging,
    )
}


pub fn mtdf_search<T: SearchResult + Default + Clone>(
    board: &Board,
    // transposition_table: &mut impl TranspositionTable,
    transposition_table: &mut Box<dyn TranspositionTable>,
    visited_boards: Vec<u64>,
    depth: u32,
    start_point: BoardEvaluation,
) -> (T, Vec<MTSearchRow>, PositionSearchRow)  {
    mtd_search(
        board,
        transposition_table,
        visited_boards,
        depth,
        start_point,
        determine_mtdf_step,
    )
}


fn determine_mtdf_step(last_test_value: BoardEvaluation, lowerbound: BoardEvaluation, upperbound: BoardEvaluation) -> BoardEvaluation {
    if last_test_value >= upperbound {
        let mut new_value = min(upperbound, last_test_value.change_centipawns(-MTDF_STEP_SIZE));
        if new_value <= lowerbound {
            new_value = avg_bounds(lowerbound, upperbound);
        }

        return new_value;
    } else if last_test_value <= lowerbound {
        let mut new_value = max(lowerbound, last_test_value.change_centipawns(MTDF_STEP_SIZE));
        if new_value >= upperbound {
            new_value = avg_bounds(lowerbound, upperbound);
        }

        return new_value;
    } else {
        panic!("MTD-F boundary calculations: last_test_value {last_test_value} wasn't lower than lowerbound {lowerbound}, or higher than upperbound {upperbound}");
    }
}
