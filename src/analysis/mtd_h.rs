use std::cmp::{max, min};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use chess::{Board, ChessMove, Color};
use crate::analysis::database::rows::{MTSearchRow, PositionSearchRow};
use crate::analysis::mtd_h_utils::{MtdHParams, select_test_point, select_test_point_w_mate, update_probability_distribution};
use crate::core::evaluation::single_evaluation;
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::conspiracy_counter::ConspiracyCounter;
use crate::core::search::conspiracy_search::log_info_search_results;
use crate::core::search::conspiracy_search::merging::MergeFn;
use crate::core::search::conspiracy_search::mt_w_conspiracy::search_mt_w_conspiracy;
use crate::core::search::conspiracy_search::mtd_w_conspiracy::mtd_search;
use crate::core::search::iterative_deepening::is_still_searching;
use crate::core::search::mtdbi::determine_mtdbi_step;
use crate::core::search::search_result::SearchResult;
use crate::core::search::transpositions::{EvalBound, TranspositionTable};
use crate::input::protocol_interpreter::CalculateOptions;


pub fn mtd_h_iterative_deepening_search<T: SearchResult + Default + Clone, L>(
    board: &Board,
    transposition_table: &mut Box<dyn TranspositionTable>,
    visited_boards: Vec<u64>,
    options: CalculateOptions,
    // step_fn_h: fn(BoardEvaluation, BoardEvaluation, BoardEvaluation) -> BoardEvaluation,
    bucket_size: u32,
    num_buckets: usize,
    conspiracy_merge_fn: MergeFn,
    search_logging: L,
    probability_distribution_params: &[MtdHParams],
) -> (T, ConspiracyCounter, u32, u32) where
    L: Fn(PositionSearchRow, Vec<MTSearchRow>) { // (SearchResult, ConspiracyCounter, depth, selective_depth)
    let now = Instant::now();

    let mut conspiracy_counters = vec![];

    let mut current_depth = 2;
    let first_result: (T, ConspiracyCounter, Vec<MTSearchRow>, PositionSearchRow) = mtd_search(
        // let mut search_result: T = mtd_search(
        board,
        transposition_table,
        visited_boards.clone(),
        1,
        BoardEvaluation::PieceScore(Centipawns::new(0)),
        determine_mtdbi_step,
        bucket_size,
        num_buckets,
        conspiracy_merge_fn,
    );
    let mut search_result = first_result.0;
    let mut conspiracy_counter = first_result.1;
    conspiracy_counters.push(conspiracy_counter.clone());
    search_logging(first_result.3, first_result.2);

    while is_still_searching(options, board, now, current_depth) {
        let applicable_probability_params = MtdHParams::find_applicable_param(probability_distribution_params, current_depth);
        let applicable_conspiracy_counter = applicable_probability_params.map(|x| {
            conspiracy_counters.get(x.training_depth.saturating_sub(1) as usize)
        }).flatten();

        if applicable_probability_params.is_none() {
            println!("no applicable param found for depth {}", current_depth);
        }
        if applicable_conspiracy_counter.is_none() {
            println!("no applicable conspiracy_counter found for depth {}", current_depth);
        }


        let temp_search_result;
        if applicable_probability_params.is_some() && applicable_conspiracy_counter.is_some() {
            temp_search_result = mtd_h_search(
                board,
                transposition_table,
                visited_boards.clone(),
                current_depth,
                search_result.eval_bound().board_evaluation(),
                // step_fn.clone(),
                select_test_point_w_mate,
                bucket_size,
                num_buckets,
                conspiracy_merge_fn,
                applicable_probability_params.unwrap(),
                applicable_conspiracy_counter.unwrap(),
            );
        } else {
            temp_search_result = mtd_search(
                board,
                transposition_table,
                visited_boards.clone(),
                current_depth,
                search_result.eval_bound().board_evaluation(),
                determine_mtdbi_step,
                bucket_size,
                num_buckets,
                conspiracy_merge_fn,
            );
        }
        search_result = temp_search_result.0;
        conspiracy_counter = temp_search_result.1;

        conspiracy_counters.push(conspiracy_counter.clone());

        // OPTIONAL LOGGING TO DB
        search_logging(temp_search_result.3, temp_search_result.2);

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



pub fn mtd_h_search<T: SearchResult + Default + Clone>(
    board: &Board,
    transposition_table: &mut Box<dyn TranspositionTable>,
    visited_boards: Vec<u64>,
    depth: u32,
    start_point: BoardEvaluation,
    step_fn_h: fn(&[f64], u32, BoardEvaluation, BoardEvaluation, BoardEvaluation) -> BoardEvaluation,
    bucket_size: u32,
    num_buckets: usize,
    conspiracy_merge_fn: MergeFn,
    probability_params: &MtdHParams,
    old_conspiracy_counter: &ConspiracyCounter,
) -> (T, ConspiracyCounter, Vec<MTSearchRow>, PositionSearchRow) {
    let mut probability_distribution = probability_params.generate_probability_distribution(
        old_conspiracy_counter,
        start_point,
    );

    let mut current_test_value = step_fn_h(
        &probability_distribution,
        old_conspiracy_counter.bucket_size,
        BoardEvaluation::BlackMate(0),
        BoardEvaluation::WhiteMate(0),
        start_point,
    );

    let current_evaluation = single_evaluation(board, board.status());

    let simple_evaluation;
    match current_evaluation {
        BoardEvaluation::PieceScore(x) => {
            simple_evaluation = x;
        },
        x => {
            return (
                T::make_search_result(
                    ChessMove::default(),
                    EvalBound::Exact(current_evaluation),
                    None,
                    None,
                ),
                ConspiracyCounter::new(bucket_size, num_buckets),
                vec![],
                PositionSearchRow {
                    run_id: 0,
                    uci_position: "".to_string(),
                    depth,
                    time_taken: 0,
                    nodes_evaluated: 1,
                    evaluation: x,
                    conspiracy_counter: None,
                    move_num: 0,
                    timestamp: 0,
                }
            );
        },
        // _ => {
        //     panic!("searching finished position");
        // }
    }
    let total_search_time = SystemTime::now();

    let mut mt_search_num = 0;
    let mut mt_searches = vec![];

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
    let mut conspiracy_counter = None;
    let mut nodes_searched = 0;
    // while lowerbound < upperbound {
    while !result.eval_bound().is_exact() {
        let time = SystemTime::now();
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

        nodes_searched += result.nodes_searched().unwrap_or(1);

        // Update the mt_searches log
        mt_searches.push(MTSearchRow {
            position_search_id: 0, // THIS NEEDS TO BE OVERWRITTEN ON POSITION_SEARCH INSERT
            test_value: current_test_value,
            time_taken: time.elapsed().expect("time went backwards").as_millis() as u32,
            nodes_evaluated: result.nodes_searched().unwrap_or(0),
            eval_bound: result.eval_bound(),
            conspiracy_counter: Some(found_conspiracy_counter.clone()),
            search_num: mt_search_num,
            timestamp: time.duration_since(UNIX_EPOCH).expect("time went backwards").as_secs() as i64,
        });
        mt_search_num += 1;

        if conspiracy_counter.is_none() {
            conspiracy_counter = Some(found_conspiracy_counter);
        } else {
            conspiracy_merge_fn(&mut conspiracy_counter.as_mut().unwrap(), &found_conspiracy_counter, &EvalBound::LowerBound(lowerbound), &EvalBound::UpperBound(upperbound));
        }

        let newest_eval;
        match result.eval_bound() {
            EvalBound::Exact(_) => {
                let position_search = PositionSearchRow {
                    run_id: 0, // NEEDS TO BE CHANGED HIGHER UP
                    uci_position: "".to_string(), // NEEDS TO BE CHANGED HIGHER UP
                    depth,
                    time_taken: total_search_time.elapsed().unwrap_or(Duration::from_secs(0)).as_millis() as u32,
                    nodes_evaluated: nodes_searched,
                    evaluation: result.eval_bound().board_evaluation(),
                    conspiracy_counter: conspiracy_counter.clone(),
                    move_num: 0, // NEEDS TO BE CHANGED HIGHER UP
                    timestamp: total_search_time.duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0)).as_secs() as i64,
                };

                return (
                    T::make_search_result(
                        result.best_move(),
                        result.eval_bound(),
                        Some(nodes_searched),
                        result.critical_path(),
                    ),
                    conspiracy_counter.unwrap(),
                    mt_searches,
                    position_search,
                );
            },
            EvalBound::UpperBound(x) => {
                upperbound = x;
                newest_eval = x;
            },
            EvalBound::LowerBound(x) => {
                lowerbound = x;
                newest_eval = x;
            },
        }

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
                        let position_search = PositionSearchRow {
                            run_id: 0, // NEEDS TO BE CHANGED HIGHER UP
                            uci_position: "".to_string(), // NEEDS TO BE CHANGED HIGHER UP
                            depth,
                            time_taken: total_search_time.elapsed().unwrap_or(Duration::from_secs(0)).as_millis() as u32,
                            nodes_evaluated: nodes_searched,
                            evaluation: result.eval_bound().board_evaluation(),
                            conspiracy_counter: conspiracy_counter.clone(),
                            move_num: 0, // NEEDS TO BE CHANGED HIGHER UP
                            timestamp: total_search_time.duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0)).as_secs() as i64,
                        };
                        return (
                            T::make_search_result(
                                result.best_move(),
                                EvalBound::Exact(result.eval_bound().board_evaluation()),
                                Some(nodes_searched),
                                result.critical_path(),
                            ),
                            conspiracy_counter.unwrap(),
                            mt_searches,
                            position_search,
                        );
                    },
                    (Color::Black, EvalBound::UpperBound(_)) => {
                        let position_search = PositionSearchRow {
                            run_id: 0, // NEEDS TO BE CHANGED HIGHER UP
                            uci_position: "".to_string(), // NEEDS TO BE CHANGED HIGHER UP
                            depth,
                            time_taken: total_search_time.elapsed().unwrap_or(Duration::from_secs(0)).as_millis() as u32,
                            nodes_evaluated: nodes_searched,
                            evaluation: result.eval_bound().board_evaluation(),
                            conspiracy_counter: conspiracy_counter.clone(),
                            move_num: 0, // NEEDS TO BE CHANGED HIGHER UP
                            timestamp: total_search_time.duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0)).as_secs() as i64,
                        };
                        return (
                            T::make_search_result(
                                result.best_move(),
                                EvalBound::Exact(result.eval_bound().board_evaluation()),
                                Some(nodes_searched),
                                result.critical_path(),
                            ),
                            conspiracy_counter.unwrap(),
                            mt_searches,
                            position_search,
                        );
                    },
                    _ => {
                        // we're not gonna hit a favourable bound, just make a small blunder before it gets worse
                        if unstable_search_counter > 6 {
                            let position_search = PositionSearchRow {
                                run_id: 0, // NEEDS TO BE CHANGED HIGHER UP
                                uci_position: "".to_string(), // NEEDS TO BE CHANGED HIGHER UP
                                depth,
                                time_taken: total_search_time.elapsed().unwrap_or(Duration::from_secs(0)).as_millis() as u32,
                                nodes_evaluated: nodes_searched,
                                evaluation: result.eval_bound().board_evaluation(),
                                conspiracy_counter: None,
                                move_num: 0, // NEEDS TO BE CHANGED HIGHER UP
                                timestamp: total_search_time.duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0)).as_secs() as i64,
                            };

                            return (
                                T::make_search_result(
                                    result.best_move(),
                                    EvalBound::Exact(result.eval_bound().board_evaluation()),
                                    Some(nodes_searched),
                                    result.critical_path(),
                                ),
                                conspiracy_counter.unwrap(),
                                mt_searches,
                                position_search,
                            );
                        }
                    },
                }
            }
        }
        update_probability_distribution(
            &mut probability_distribution,
            result.eval_bound(),
            old_conspiracy_counter.bucket_size,
        );

        current_test_value = step_fn_h(
            &probability_distribution,
            bucket_size,
            lowerbound,
            upperbound,
            result.eval_bound().board_evaluation(),
        );

        // prevents the program from gobbling up all the system memory
        // and destabilizing the OS (had to restart)
        if mt_search_num > 100 {
            panic!("already done more than 100 mt_searches, likely in infinite loop");
        }
    }

    let position_search = PositionSearchRow {
        run_id: 0, // NEEDS TO BE CHANGED HIGHER UP
        uci_position: "".to_string(), // NEEDS TO BE CHANGED HIGHER UP
        depth,
        time_taken: total_search_time.elapsed().unwrap_or(Duration::from_secs(0)).as_millis() as u32,
        nodes_evaluated: nodes_searched,
        evaluation: result.eval_bound().board_evaluation(),
        conspiracy_counter: conspiracy_counter.clone(),
        move_num: 0, // NEEDS TO BE CHANGED HIGHER UP
        timestamp: total_search_time.duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0)).as_secs() as i64,
    };

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
        conspiracy_counter.unwrap(),
        mt_searches,
        position_search,
    )
}


