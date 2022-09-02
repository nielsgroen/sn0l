use std::num::NonZeroU32;
use std::time::{Duration, Instant};
use chess::{Board, ChessMove, Color};
use crate::core::evaluation::SearchResult;
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::alpha_beta::search_depth_pruned;

use crate::core::search::transposition::TranspositionTable;
use crate::input::protocol_interpreter::CalculateOptions;


pub fn iterative_deepening_search(
    board: &Board,
    transposition_table: &mut TranspositionTable,
    options: CalculateOptions,
) -> (SearchResult, u32, u32) { // (SearchResult, depth, selective_depth)
    let mut max_search_depth: u32 = 1;

    match options {
        CalculateOptions::Depth(x) => max_search_depth = x,
        CalculateOptions::Infinite => max_search_depth = 5, // todo
        _ => panic!("unsupported iterative deepening calculate options"),
    }

    let selective_depth: u32 = 8; // TODO
    // TODO: Set this up
    // for max_depth in 1..max_search_depth {
    //

    // }

    let now = Instant::now();
    let search_result = search_depth_pruned(
        board,
        transposition_table,
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

fn log_info_search_results(
    search_result: &SearchResult,
    side_to_move: Color,
    duration: Duration,
    depth: u32,
    selective_depth: u32
) {
    let nodes_per_second = search_result.nodes_searched as u128 / duration.as_millis() * 1000;

    let score_string = match (side_to_move, search_result.board_evaluation) {
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

    let nodes = search_result.nodes_searched;
    let critical_path = search_result.critical_path.clone()
        .into_iter()
        .rev()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    println!("info nps {nodes_per_second}");
    println!(
        "info score {score_string} depth {depth} seldepth {selective_depth} nodes {nodes} time {} pv {critical_path}",
        duration.as_millis(),
    );
}
