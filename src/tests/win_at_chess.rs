use std::path::{Path, PathBuf};
use std::error::Error;
use std::str::FromStr;
use crate::tests::epd::{EPDParseError, EPDRecord};
use super::epd;
use anyhow::{Result, anyhow, bail};
use chess::{Board, ChessMove};
use crate::core::search::alpha_beta::search_depth_pruned;
use crate::core::search::iterative_deepening::iterative_deepening_search;
use crate::core::search::search_result;
use crate::core::search::search_result::debug_search_result::DebugSearchResult;
use crate::core::search::search_result::SearchResult;
use crate::core::search::transpositions::no_transposition::NoTranspositionTable;
use crate::input::protocol_interpreter::CalculateOptions;
use crate::tests::TestError;

const DEPTH: u32 = 10;

#[test]
fn check_positions() -> Result<()> {
    let epd_path = PathBuf::from("./src/tests/assets/win_at_chess.epd");

    println!("{:?}", epd_path);
    let records = epd::read_epd(epd_path.as_path()).expect("failed to read epd");
    let records = &records[0..3]; // TODO: remove

    let mut failed_positions = vec![];
    for record in records.into_iter() {
        let result = check_position(&record, |board| {
            let mut transposition_table = NoTranspositionTable::default();
            let (result, _, _): (DebugSearchResult, u32, u32) = iterative_deepening_search(
                board,
                &mut transposition_table,
                vec![],
                CalculateOptions::Depth(6),
            );

            println!("{result:?}");

            result
        });

        if let Err(TestError::WrongMove {expected, actual}) = result {
            failed_positions.push((record.id.clone(), expected, actual))
        }

        // let board = Board::from_str(&record.fen).map_err(|_| EPDParseError::InvalidFEN)?;
        // let mut transposition_table = NoTranspositionTable::default();
        //
        // let (search_result, _, _): (DebugSearchResult, _, _) = iterative_deepening_search(
        //     &board,
        //     &mut transposition_table,
        //     vec![],
        //     CalculateOptions::Depth(8),
        // );

        // assert_eq!(search_result.best_move, record.best_move);
    }

    let mut some_failed_position = false;
    for failed_position in failed_positions.into_iter() {
        some_failed_position = true;
        let id = failed_position.0;
        let expected = failed_position.1;
        let actual = failed_position.2;
        println!("Failed {id:?}, (expected {expected}, got {actual})");
    }

    if some_failed_position {
        bail!("Failed some positions");
    }
    Ok(())
}

pub fn check_position<F>(record: &EPDRecord, search_method: F) -> Result<(), TestError>
    where F: Fn(&Board) -> DebugSearchResult {

    let board = Board::from_str(&record.fen).map_err(|_| EPDParseError::InvalidFEN).unwrap();
    let found_result = search_method(&board);

    match record.best_move == found_result.best_move {
        true => Ok(()),
        false => Err(TestError::WrongMove {
            expected: record.best_move,
            actual: found_result.best_move,
        }),
    }
}

