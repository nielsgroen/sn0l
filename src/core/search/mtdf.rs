use std::cmp::{max, min};
use std::time::Instant;
use chess::{Board, ChessMove};
use crate::core::evaluation::single_evaluation;
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::iterative_deepening::{is_still_searching, log_info_search_results};
use crate::core::search::mt::search_mt;
use crate::core::search::search_result::debug_search_result::DebugSearchResult;
use crate::core::search::search_result::SearchResult;
use crate::core::search::transpositions::{EvalBound, TranspositionTable};
use crate::input::protocol_interpreter::CalculateOptions;

/// The code implementing the MTD-F search algorithm

const MTDF_STEP_SIZE: Centipawns = Centipawns::new(30);


pub fn mtdf_iterative_deepening_search<T: SearchResult + Default + Clone>(
    board: &Board,
    transposition_table: &mut impl TranspositionTable,
    visited_boards: Vec<u64>,
    start_point: BoardEvaluation,
    options: CalculateOptions,
) -> (T, u32, u32) { // (SearchResult, depth, selective_depth)
    let now = Instant::now();
    let mut current_depth = 2;
    let mut search_result: T = mtdf_search(
        board,
        transposition_table,
        visited_boards.clone(),
        1,
        BoardEvaluation::PieceScore(Centipawns::new(0)),
    );

    while is_still_searching(options, board, now, current_depth) {
        search_result = mtdf_search(
            board,
            transposition_table,
            visited_boards.clone(),
            current_depth,
            search_result.eval_bound().board_evaluation(),
        );
        let duration = now.elapsed();
        log_info_search_results(
            &search_result,
            board.side_to_move(),
            duration,
            current_depth,
            current_depth, // TODO: Change to actual selective depth
        );
        current_depth += 1;
    }

    let search_result = T::make_search_result(
        search_result.best_move(),
        EvalBound::Exact(search_result.eval_bound().board_evaluation()),
        search_result.nodes_searched(),
        search_result.critical_path(),
    );

    (
        search_result,
        current_depth,
        current_depth,
    )
}


pub fn mtdf_search<T: SearchResult + Default + Clone>(
    board: &Board,
    transposition_table: &mut impl TranspositionTable,
    visited_boards: Vec<u64>,
    depth: u32,
    start_point: BoardEvaluation,
    // selective_depth: u32,
) -> T {
    let mut current_test_value = start_point;
    let current_evaluation = single_evaluation(board, board.status());

    let simple_evaluation;
    match current_evaluation {
        BoardEvaluation::PieceScore(x) => {
            simple_evaluation = x;
        },
        _ => {
            return T::make_search_result(
                ChessMove::default(),
                EvalBound::Exact(current_evaluation),
                None,
                None,
            );
        },
    }

    // The starting lowerbound is as low as possible
    // This is not exactly a bound
    // But nothing is lower than `at most immediate mate for black`
    // let mut lowerbound = EvalBound::UpperBound(BoardEvaluation::BlackMate(0));
    let mut lowerbound = BoardEvaluation::BlackMate(0);

    // The starting upperbound is as high as possible
    // `At least immediate mate for white`
    // let mut upperbound = EvalBound::LowerBound(BoardEvaluation::WhiteMate(0));
    let mut upperbound = BoardEvaluation::WhiteMate(0);

    let mut result = T::default();
    while lowerbound < upperbound {
        result = search_mt(
            board,
            transposition_table,
            visited_boards.clone(),
            simple_evaluation,
            EvalBound::Exact(current_test_value),
            0,
            depth,
        );

        match result.eval_bound() {
            EvalBound::Exact(_) => {
                return result;
            },
            EvalBound::UpperBound(x) => {
                // Result found is `x` or less
                // => so `x` is the new upper bound
                upperbound = x;
            },
            EvalBound::LowerBound(x) => {
                // Result found is `x` or more
                // => so `x` is the new lower bound
                lowerbound = x;
            },
        };

        current_test_value = determine_mtdf_step(current_test_value, lowerbound, upperbound);
    }

    // Accounts for instability in search due to transposition table
    // Usually `lowerbound == upperbound` here
    // But not always: just return anyways
    result
}


fn avg_bounds(lowerbound: BoardEvaluation, upperbound: BoardEvaluation) -> BoardEvaluation {
    match (lowerbound, upperbound) {
        (BoardEvaluation::PieceScore(x), BoardEvaluation::PieceScore(y)) => {
            return BoardEvaluation::PieceScore(Centipawns::new((x.0 + y.0) / 2));
        },
        (BoardEvaluation::BlackMate(x), _) => {
            return lowerbound;
        },
        (_, BoardEvaluation::WhiteMate(x)) => {
            return upperbound;
        },
        _ => {
            panic!("MTD-F boundary calculations: lowerbound {lowerbound} and upperbound {upperbound} are impossible");
        },
    }
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

