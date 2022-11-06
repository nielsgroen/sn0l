use std::cmp::min;
use std::num::NonZeroU32;
use std::time::{Duration, Instant};
use chess::{Board, ChessMove, Color};
use super::search_result::debug_search_result::DebugSearchResult;
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::alpha_beta::search_depth_pruned;
use crate::core::search::search_result::SearchResult;

use crate::core::search::transpositions::TranspositionTable;
use crate::input::protocol_interpreter::CalculateOptions;


pub fn iterative_deepening_search<T: SearchResult + Default>(
    board: &Board,
    transposition_table: &mut impl TranspositionTable,
    visited_boards: Vec<u64>, // TODO: use
    options: CalculateOptions,
) -> (T, u32, u32) { // (SearchResult, depth, selective_depth)
    let mut max_search_depth: u32 = 1;

    match options {
        CalculateOptions::Depth(x) => max_search_depth = x,
        CalculateOptions::Infinite => max_search_depth = 5, // todo
        _ => panic!("unsupported iterative deepening calculate options"),
    }

    // let selective_depth: u32 = min(10, max_search_depth); // TODO
    let selective_depth: u32 = 10;
    // TODO: Set this up
    // for max_depth in 1..max_search_depth {
    //

    // }

    let now = Instant::now();
    let search_result: T = search_depth_pruned(
        board,
        transposition_table,
        visited_boards,
        max_search_depth,
        Some(selective_depth), // todo
    );
    let duration = now.elapsed();
    log_info_search_results(
        &search_result,
        board.side_to_move(),
        duration,
        max_search_depth, // TODO: change to actual depth when iteratively deepening
        selective_depth,
    );

    (
        search_result,
        max_search_depth,
        selective_depth,
    )
}

fn log_info_search_results<T: SearchResult>(
    search_result: &T,
    side_to_move: Color,
    duration: Duration,
    depth: u32,
    selective_depth: u32
) {
    let score_string = match (side_to_move, search_result.board_evaluation()) {
        (Color::White, BoardEvaluation::PieceScore(Centipawns(x))) => {
            format!("cp {}", x)
        },
        (Color::White, BoardEvaluation::WhiteMate(x)) => {
            format!("mate {}", x)
        },
        (Color::White, BoardEvaluation::BlackMate(x)) => {
            format!("mate -{}", x)
        },
        (Color::Black, BoardEvaluation::PieceScore(Centipawns(x))) => {
            format!("cp {}", -x)
        },
        (Color::Black, BoardEvaluation::WhiteMate(x)) => {
            format!("mate -{}", x)
        },
        (Color::Black, BoardEvaluation::BlackMate(x)) => {
            format!("mate {}", x)
        },
    };

    let nodes_string = match search_result.nodes_searched() {
        None => "".to_string(),
        Some(x) => format!("nodes {x}"),
    };

    let critical_path_string;
    if search_result.critical_path().is_some() {
        let critical_path = search_result.critical_path()
            .clone()
            .unwrap()
            .into_iter()
            // .rev()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        critical_path_string = format!("pv {critical_path}");
    } else {
        critical_path_string = "".to_string();
    }

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
