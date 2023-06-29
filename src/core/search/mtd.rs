use std::cmp::{max, min};
use std::time::Instant;
use chess::{Board, ChessMove, Color};
use crate::core::evaluation::single_evaluation;
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::alpha_beta::search_alpha_beta;
use crate::core::search::iterative_deepening::{determine_critical_path_string, is_still_searching, log_info_search_results};
use crate::core::search::mt::search_mt;
use crate::core::search::mtdf::mtdf_search;
use crate::core::search::search_result::SearchResult;
use crate::core::search::transpositions::{EvalBound, TranspositionTable};
use crate::input::protocol_interpreter::CalculateOptions;

/// The base implementation of the mtd framework


pub fn mtd_iterative_deepening_search<T: SearchResult + Default + Clone>(
    board: &Board,
    transposition_table: &mut impl TranspositionTable,
    visited_boards: Vec<u64>,
    options: CalculateOptions,
    step_fn: fn(BoardEvaluation, BoardEvaluation, BoardEvaluation) -> BoardEvaluation,
) -> (T, u32, u32) { // (SearchResult, depth, selective_depth)
    let now = Instant::now();
    let mut current_depth = 2;
    let mut search_result: T = mtd_search(
        board,
        transposition_table,
        visited_boards.clone(),
        1,
        BoardEvaluation::PieceScore(Centipawns::new(0)),
        step_fn.clone(),
    );

    while is_still_searching(options, board, now, current_depth) {
        search_result = mtd_search(
            board,
            transposition_table,
            visited_boards.clone(),
            current_depth,
            search_result.eval_bound().board_evaluation(),
            step_fn.clone(),
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

    (
        search_result,
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
    let mut highest_lowerbound = lowerbound;

    // The starting upperbound is as high as possible
    // `At least immediate mate for white`
    // let mut upperbound = EvalBound::LowerBound(BoardEvaluation::WhiteMate(0));
    let mut upperbound = BoardEvaluation::WhiteMate(0);
    let mut lowest_upperbound = upperbound;

    let mut unstable_search_counter = 0;

    let mut result = T::make_search_result(
        ChessMove::default(),
        EvalBound::UpperBound(BoardEvaluation::BlackMate(0)),
        None,
        None,
    );
    let mut nodes_searched = 0;
    // while lowerbound < upperbound {
    while !result.eval_bound().is_exact() {
        result = search_mt(
            board,
            transposition_table,
            visited_boards.clone(),
            simple_evaluation,
            EvalBound::Exact(current_test_value),
            0,
            depth,
        );
        nodes_searched += result.nodes_searched().unwrap_or(1);
        // println!("----------");
        // println!("start lowerbound {lowerbound}, upperbound {upperbound}");
        // println!("mt_search result eval_bound: {:?}", result.eval_bound());
        // println!("mt_search result path: {:?}", determine_critical_path_string(result.critical_path()));
        // println!("mt_search result best_move: {}", result.best_move());
        // println!("mt_search result nodes_searched: {:?}", result.nodes_searched());

        let newest_eval;
        match result.eval_bound() {
            EvalBound::Exact(_) => {
                return T::make_search_result(
                    result.best_move(),
                    result.eval_bound(),
                    Some(nodes_searched),
                    result.critical_path(),
                );
            },
            EvalBound::UpperBound(x) => {
                // Result found is `x` or less
                // => so `x` is the new upper bound
                upperbound = x;
                newest_eval = x;
            },
            EvalBound::LowerBound(x) => {
                // Result found is `x` or more
                // => so `x` is the new lower bound
                lowerbound = x;
                newest_eval = x;
            },
        };

        if upperbound < lowerbound {
            unstable_search_counter += 1;

            lowest_upperbound = min(upperbound, lowest_upperbound);
            highest_lowerbound = max(lowerbound, highest_lowerbound);
            println!("UNSTABLE SEARCH: logging");
            println!("lowerbound {:?}, upperbound {:?}", lowerbound, upperbound);
            let path = result.critical_path().unwrap();
            for chess_move in path.into_iter() {
                print!("{} ", chess_move);
            }
            println!("");
            // println!("result path: {}", result.critical_path());
            println!("result best_move: {}", result.best_move());
            println!("result nodes_searched: {:?}", result.nodes_searched());
            println!("UNSTABLE SEARCH: end logging");

            upperbound = newest_eval;
            lowerbound = newest_eval;

            // If search is still unstable:
            // - if white: end on lowerbound
            // - if black: end on upperbound
            if unstable_search_counter > 3 {
                match (board.side_to_move(), result.eval_bound()) {
                    (Color::White, EvalBound::LowerBound(_)) => {
                        return T::make_search_result(
                            result.best_move(),
                            EvalBound::Exact(result.eval_bound().board_evaluation()),
                            Some(nodes_searched),
                            result.critical_path(),
                        );
                    },
                    (Color::Black, EvalBound::UpperBound(_)) => {
                        return T::make_search_result(
                            result.best_move(),
                            EvalBound::Exact(result.eval_bound().board_evaluation()),
                            Some(nodes_searched),
                            result.critical_path(),
                        );
                    },
                    _ => (),
                }

                // let alpha = match lowest_upperbound {
                //     BoardEvaluation::BlackMate(_) => panic!("Mate boundary in unstable search: impossible"),
                //     BoardEvaluation::PieceScore(x) => BoardEvaluation::PieceScore(x - Centipawns::new(1)),
                //     BoardEvaluation::WhiteMate(_) => panic!("Mate boundary in unstable search: impossible"),
                // };
                //
                // let beta = match highest_lowerbound {
                //     BoardEvaluation::BlackMate(_) => panic!("Mate boundary in unstable search: impossible"),
                //     BoardEvaluation::PieceScore(x) => BoardEvaluation::PieceScore(x + Centipawns::new(1)),
                //     BoardEvaluation::WhiteMate(_) => panic!("Mate boundary in unstable search: impossible"),
                // };
                //
                // println!("highest_lowerbound {:?}", highest_lowerbound);
                // println!("lowest_upperbound {:?}", lowest_upperbound);
                //
                // let mut alpha_beta_result: T = search_alpha_beta(
                //     board,
                //     transposition_table,
                //     visited_boards.clone(),
                //     simple_evaluation,
                //     EvalBound::Exact(alpha), // TODO: re-enable
                //     EvalBound::Exact(beta), // TODO: re-enable
                //     // EvalBound::Exact(BoardEvaluation::BlackMate(0)), // TODO: remove
                //     // EvalBound::Exact(BoardEvaluation::WhiteMate(0)), // TODO: remove
                //     0,
                //     depth,
                //     depth,
                // );
                //
                // println!("alpha_beta_result best_move: {}", alpha_beta_result.best_move());
                // println!("alpha_beta_result nodes_searched: {:?}", alpha_beta_result.nodes_searched());
                // println!("alpha_beta_result eval: {:?}", alpha_beta_result.eval_bound());
                //
                // match alpha_beta_result.eval_bound() {
                //     EvalBound::UpperBound(_) => panic!("widened search after unstable search failed to return within bounds"),
                //     EvalBound::Exact(_) => (),
                //     EvalBound::LowerBound(_) => panic!("widened search after unstable search failed to return within bounds"),
                // };
                //
                // alpha_beta_result.set_nodes_searched(alpha_beta_result.nodes_searched().map(|x| x + nodes_searched));
                // return alpha_beta_result;
            }
        }
        current_test_value = step_fn(current_test_value, lowerbound, upperbound);
    }

    // Accounts for instability in search due to transposition table
    // Usually `lowerbound == upperbound` here
    // But not always: just return anyways
    T::make_search_result(
        result.best_move(),
        EvalBound::Exact(result.eval_bound().board_evaluation()),
        Some(nodes_searched),
        result.critical_path(),
    )
}


pub fn avg_bounds(lowerbound: BoardEvaluation, upperbound: BoardEvaluation) -> BoardEvaluation {
    match (lowerbound, upperbound) {
        (BoardEvaluation::BlackMate(_), BoardEvaluation::WhiteMate(_)) => {
            return BoardEvaluation::PieceScore(Centipawns::new(0));
        },
        (BoardEvaluation::PieceScore(x), BoardEvaluation::PieceScore(y)) => {
            return BoardEvaluation::PieceScore(Centipawns::new((x.0 + y.0) / 2));
        },
        (BoardEvaluation::BlackMate(_), BoardEvaluation::BlackMate(_)) => {
            return lowerbound;
        },
        (BoardEvaluation::BlackMate(x), _) => {
            // Cut off the initial checkmate boundaries first
            if x == 0 {
                return BoardEvaluation::PieceScore(Centipawns::new(-20000));
            }
            return lowerbound;
        },
        (BoardEvaluation::WhiteMate(_), BoardEvaluation::WhiteMate(_)) => {
            return upperbound;
        },
        (_, BoardEvaluation::WhiteMate(x)) => {
            // Cut off the initial checkmate boundaries first
            if x == 0 {
                return BoardEvaluation::PieceScore(Centipawns::new(20000));
            }
            return upperbound;
        },
        _ => {
            panic!("MTD boundary calculations: lowerbound {lowerbound} and upperbound {upperbound} are impossible");
        },
    }
}
