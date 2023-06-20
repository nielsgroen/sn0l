use std::time::Instant;
use chess::{Board, ChessMove};
use crate::core::evaluation::single_evaluation;
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::conspiracy_counter::ConspiracyCounter;
use crate::core::search::conspiracy_search::log_info_search_results;
use crate::core::search::conspiracy_search::mt_w_conspiracy::search_mt_w_conspiracy;
use crate::core::search::iterative_deepening::is_still_searching;
use crate::core::search::mt::search_mt;
use crate::core::search::mtdf::mtdf_search;
use crate::core::search::search_result::SearchResult;
use crate::core::search::transpositions::{EvalBound, TranspositionTable};
use crate::input::protocol_interpreter::CalculateOptions;

/// The base implementation of the mtd framework but with conspiracy counters


pub fn mtd_iterative_deepening_search<T: SearchResult + Default + Clone>(
    board: &Board,
    transposition_table: &mut impl TranspositionTable,
    visited_boards: Vec<u64>,
    options: CalculateOptions,
    step_fn: fn(BoardEvaluation, BoardEvaluation, BoardEvaluation) -> BoardEvaluation,
    bucket_size: u32,
    num_buckets: usize,
    conspiracy_merge_fn: fn(&mut ConspiracyCounter, &ConspiracyCounter, &EvalBound, &EvalBound),
) -> (T, ConspiracyCounter, u32, u32) { // (SearchResult, depth, selective_depth)
    let now = Instant::now();
    let mut current_depth = 2;
    let first_result: (T, ConspiracyCounter) = mtd_search(
    // let mut search_result: T = mtd_search(
        board,
        transposition_table,
        visited_boards.clone(),
        1,
        BoardEvaluation::PieceScore(Centipawns::new(0)),
        step_fn.clone(),
        bucket_size,
        num_buckets,
        conspiracy_merge_fn,
    );
    let mut search_result = first_result.0;
    let mut conspiracy_counter = first_result.1;

    while is_still_searching(options, board, now, current_depth) {
        let temp_search_result = mtd_search(
            board,
            transposition_table,
            visited_boards.clone(),
            current_depth,
            search_result.eval_bound().board_evaluation(),
            step_fn.clone(),
            bucket_size,
            num_buckets,
            conspiracy_merge_fn,
        );
        search_result = temp_search_result.0;
        conspiracy_counter = temp_search_result.1;

        let duration = now.elapsed();
        log_info_search_results(
            &search_result,
            board.side_to_move(),
            duration,
            current_depth,
            current_depth, // TODO: Change to actual selective depth
            &conspiracy_counter,
        );
        current_depth += 1;
    }

    (
        search_result,
        conspiracy_counter,
        current_depth,
        current_depth,
    )
}


pub fn mtd_search<T: SearchResult + Default + Clone>(
    board: &Board,
    transposition_table: &mut impl TranspositionTable,
    visited_boards: Vec<u64>,
    depth: u32,
    start_point: BoardEvaluation,
    step_fn: fn(BoardEvaluation, BoardEvaluation, BoardEvaluation) -> BoardEvaluation,
    bucket_size: u32,
    num_buckets: usize,
    conspiracy_merge_fn: fn(&mut ConspiracyCounter, &ConspiracyCounter, &EvalBound, &EvalBound),
) -> (T, ConspiracyCounter) {
    let mut current_test_value = start_point;
    let current_evaluation = single_evaluation(board, board.status());

    let simple_evaluation;
    match current_evaluation {
        BoardEvaluation::PieceScore(x) => {
            simple_evaluation = x;
        },
        _ => {
            return (
                T::make_search_result(
                    ChessMove::default(),
                    EvalBound::Exact(current_evaluation),
                    None,
                    None,
                ),
                ConspiracyCounter::new(bucket_size, num_buckets)
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
    let mut conspiracy_counter = None;
    let mut nodes_searched = 0;
    while lowerbound < upperbound {
        let search_result = search_mt_w_conspiracy(
            board,
            transposition_table,
            visited_boards.clone(),
            simple_evaluation,
            EvalBound::Exact(current_test_value),
            0,
            depth,
            bucket_size,
            num_buckets,
        );
        result = search_result.0;
        let found_conspiracy_counter = search_result.1;

        match result.eval_bound() {
            EvalBound::Exact(x) => {
                lowerbound = x;
                upperbound = x;
            },
            EvalBound::UpperBound(x) => {
                upperbound = x;
            },
            EvalBound::LowerBound(x) => {
                lowerbound = x;
            },
        }

        if conspiracy_counter.is_none() {
            conspiracy_counter = Some(found_conspiracy_counter);
        } else {
            conspiracy_merge_fn(&mut conspiracy_counter.as_mut().unwrap(), &found_conspiracy_counter, &EvalBound::LowerBound(lowerbound), &EvalBound::UpperBound(upperbound));
        }

        nodes_searched += result.nodes_searched().unwrap_or(1);

        // Apparently the search was unstable
        // TODO: make debug only
        if upperbound < lowerbound {
            println!("UNSTABLE SEARCH");
            println!("lowerbound {:?}, upperbound {:?}", lowerbound, upperbound);
            let path = result.critical_path().unwrap();
            for chess_move in path.into_iter() {
                print!("{} ", chess_move);
            }
            println!();
            // println!("result path: {}", result.critical_path());
            println!("result best_move: {}", result.best_move());
            println!("result nodes_searched: {:?}", result.nodes_searched());
        }
        current_test_value = step_fn(current_test_value, lowerbound, upperbound);
    }

    // Accounts for instability in search due to transposition table
    // Usually `lowerbound == upperbound` here
    // But not always: just return anyways
    (
        T::make_search_result(
            result.best_move(),
            EvalBound::Exact(result.eval_bound().board_evaluation()),
            Some(nodes_searched),
            result.critical_path(),
        ),
        conspiracy_counter.unwrap()
    )
}
