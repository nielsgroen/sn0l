use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use anyhow::{bail, Result};
use chess::Board;
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::iterative_deepening::iterative_deepening_search;
use crate::core::search::mtdf::mtdf_iterative_deepening_search;
use crate::core::search::search_result::debug_search_result::DebugSearchResult;
use crate::core::search::transpositions::no_transposition::NoTranspositionTable;
use crate::input::protocol_interpreter::CalculateOptions;
use crate::tests::{epd, win_at_chess};
use crate::tests::epd::EPDParseError;
use crate::tests::mt_alpha_beta_equivalence::log_dissimilar_answers;

/// Tests whether MTD-F returns the same solutions as Alpha-Beta pruning
const MAX_DEPTH: u32 = 4;

#[test]
fn check_mtdf_alpha_beta_equivalence() -> Result<()> {
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
        println!("alpha beta eval {:?}:", result.board_evaluation);


        let time = Instant::now();
        let (mtdf_result, _, _): (DebugSearchResult, u32, u32) = {
            // let mut transposition_table = HighDepthTranspositionTable::new(SearchDepth::Depth(2));
            let mut transposition_table = NoTranspositionTable::default();

             mtdf_iterative_deepening_search(
                &board,
                &mut transposition_table,
                vec![],
                BoardEvaluation::PieceScore(Centipawns::new(0)),
                CalculateOptions::Depth(MAX_DEPTH),
            )
        };
        println!("mtdf time ms: {}", time.elapsed().as_millis());
        println!("mtdf time nodes searched: {}", mtdf_result.nodes_searched);
        println!("mtdf eval {:?}:", mtdf_result.board_evaluation);

        if result.board_evaluation != mtdf_result.board_evaluation {
            failed_positions.push((record.id.clone(), result, mtdf_result));
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