use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use anyhow::{bail, Result};
use chess::Board;
use crate::core::evaluation::single_evaluation;
use crate::core::score::BoardEvaluation;
use crate::core::search::iterative_deepening::iterative_deepening_search;
use crate::core::search::mt::search_mt;
use crate::core::search::search_result::debug_search_result::DebugSearchResult;
use crate::core::search::SearchDepth;
use crate::core::search::transpositions::high_depth_transposition::HighDepthTranspositionTable;
use crate::core::search::transpositions::no_transposition::NoTranspositionTable;
use crate::input::protocol_interpreter::CalculateOptions;
use crate::tests::{epd, win_at_chess};
use crate::tests::epd::EPDParseError;

/// Tests whether MTD implementations are equal to the alpha-beta implementation

#[test]
fn check_mt_exact() -> Result<()> {
    const MAX_DEPTH: u32 = 6;

    let epd_path = PathBuf::from(win_at_chess::EPD_PATH);
    println!("{:?}", epd_path);

    let records = epd::read_epd(epd_path.as_path()).expect("failed to read epd");
    // let records = &records[0..1]; // TODO: remove

    let mut failed_positions = vec![];
    for record in records.into_iter() {
        if let Some(record_id) = record.id.clone() {
            println!("{record_id}");
        }

        let board = Board::from_str(&record.fen).map_err(|_| EPDParseError::InvalidFEN).unwrap();
        let time = Instant::now();
        let (result, _, _): (DebugSearchResult, u32, u32) = {
            // let mut transposition_table = HighDepthTranspositionTable::new(SearchDepth::Depth(2));
            let mut transposition_table = NoTranspositionTable::default();

            iterative_deepening_search(
                &board,
                &mut transposition_table,
                vec![],
                CalculateOptions::Depth(MAX_DEPTH),
            )
        };
        println!("alpha beta time ms: {}", time.elapsed().as_millis());
        println!("alpha beta nodes searched: {}", result.nodes_searched);

        let time = Instant::now();
        let mt_result: DebugSearchResult = {
            // let mut transposition_table = HighDepthTranspositionTable::new(SearchDepth::Depth(2));
            let mut transposition_table = NoTranspositionTable::default();
            let simple_evaluation = single_evaluation(&board, board.status());

            let simple_score;
            match simple_evaluation {
                BoardEvaluation::PieceScore(x) => simple_score = x,
                _ => panic!("searching finished position"),
            }

            search_mt(
                &board,
                &mut transposition_table,
                vec![],
                simple_score,
                result.board_evaluation,
                0,
                MAX_DEPTH,
            )
        };
        println!("mt time ms: {}", time.elapsed().as_millis());
        println!("mt time nodes searched: {}", mt_result.nodes_searched);

        if result.board_evaluation != mt_result.board_evaluation {
            failed_positions.push((record.id.clone(), result, mt_result));
        }
    }

    let some_failed_positions = failed_positions.len() > 0;
    for failed_position in failed_positions {
        log_dissimilar_answers(&failed_position.0.unwrap_or("unknown ID".to_string()), &failed_position.1, &failed_position.2);
    }

    if some_failed_positions {
        bail!("Failed some positions");
    }
    Ok(())
}

fn log_dissimilar_answers(id: &str, expected: &DebugSearchResult, actual: &DebugSearchResult) {
    println!("Failed {id}, (expected {expected:?}, got {actual:?})");
}
