use std::time::{Duration, Instant};
use chess::{Board, ChessMove, Color};
use crate::analysis::database::rows::{MTSearchRow, PositionSearchRow};
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::alpha_beta::search_depth_pruned;
use crate::core::search::search_result::SearchResult;

use crate::core::search::transpositions::TranspositionTable;
use crate::input::protocol_interpreter::CalculateOptions;


/// Determines whether the next depth should be searched
pub fn is_still_searching(
    calculate_options: CalculateOptions,
    board: &Board,
    search_start: Instant,
    depth_to_search: u32,
) -> bool {
    match calculate_options {
        CalculateOptions::Depth(x) => depth_to_search <= x,
        CalculateOptions::Infinite => true,
        CalculateOptions::Game {
            white_time,
            white_increment,
            black_time,
            black_increment,
        } => {
            match board.side_to_move() {
                Color::White => {
                    let already_searched = search_start.elapsed().as_millis() as u64;
                    let extra_calc_time = 5 * already_searched + 10;

                    (already_searched + extra_calc_time).saturating_sub(white_increment) < white_time / 50
                },
                Color::Black => {
                    let already_searched = search_start.elapsed().as_millis() as u64;
                    let extra_calc_time = 5 * already_searched;

                    (already_searched + extra_calc_time).saturating_sub(black_increment) < black_time / 50
                },
            }
        },
        CalculateOptions::MoveTime(x) => {
            // TODO: engine actually should abort as soon as the ms passed
            // This will go to the next depth as long as MoveTime x hasn't passed yet.
            (search_start.elapsed().as_millis() as u64) < x
        },
    }
}

pub fn iterative_deepening_search<T: SearchResult + Default, L>(
    board: &Board,
    // transposition_table: &mut impl TranspositionTable,
    transposition_table: &mut Box<dyn TranspositionTable>,
    visited_boards: Vec<u64>,
    options: CalculateOptions,
    search_logging: L,
) -> (T, u32, u32) where
    L: Fn(PositionSearchRow, Vec<MTSearchRow>) { // (SearchResult, depth, selective_depth)
    // let mut max_search_depth: u32 = 1;

    // let selective_depth: u32 = min(10, max_search_depth); // TODO
    // let selective_depth: u32 = 10;
    // TODO: Set this up
    // for max_depth in 1..max_search_depth {
    //

    // }
    let now = Instant::now();
    let mut current_depth = 2;
    let (mut search_result, position_row): (T, PositionSearchRow) = search_depth_pruned(
        board,
        transposition_table,
        visited_boards.clone(),
        1,
        None,
    );
    search_logging(position_row, vec![]);

    while is_still_searching(options, board, now, current_depth) {
        let temp_search_result = search_depth_pruned(
            board,
            transposition_table,
            visited_boards.clone(),
            current_depth,
            None,
        );
        search_result = temp_search_result.0;
        search_logging(temp_search_result.1, vec![]);

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

pub fn log_info_search_results<T: SearchResult>(
    search_result: &T,
    side_to_move: Color,
    duration: Duration,
    depth: u32,
    selective_depth: u32
) {
    let score_string = match (side_to_move, search_result.eval_bound().board_evaluation()) {
        (Color::White, BoardEvaluation::PieceScore(Centipawns(x))) => {
            format!("cp {}", x)
        },
        (Color::White, BoardEvaluation::WhiteMate(x)) => {
            format!("mate {}", x / 2)
        },
        (Color::White, BoardEvaluation::BlackMate(x)) => {
            format!("mate -{}", x / 2)
        },
        (Color::Black, BoardEvaluation::PieceScore(Centipawns(x))) => {
            format!("cp {}", -x)
        },
        (Color::Black, BoardEvaluation::WhiteMate(x)) => {
            format!("mate -{}", x / 2)
        },
        (Color::Black, BoardEvaluation::BlackMate(x)) => {
            format!("mate {}", x / 2)
        },
    };

    let nodes_string = match search_result.nodes_searched() {
        None => "".to_string(),
        Some(x) => format!("nodes {x}"),
    };

    let critical_path_string = determine_critical_path_string(search_result.critical_path());
    let millis = duration.as_millis();

    if millis > 0 && search_result.nodes_searched().is_some() {
        let nodes_per_second = search_result.nodes_searched().unwrap() as u128 / duration.as_millis() * 1000;
        println!("info nps {nodes_per_second}");
    }
    println!(
        "info score {score_string} depth {depth} seldepth {selective_depth} {nodes_string} time {} {critical_path_string}",
        duration.as_millis(),
    );
}

pub fn determine_critical_path_string(critical_path: Option<Vec<ChessMove>>) -> String {
    let critical_path_string;
    if critical_path.is_some() {
        let critical_path = critical_path
            .clone()
            .unwrap()
            .into_iter()
            .rev()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        critical_path_string = format!("pv {critical_path}");
    } else {
        critical_path_string = "".to_string();
    }

    critical_path_string
}