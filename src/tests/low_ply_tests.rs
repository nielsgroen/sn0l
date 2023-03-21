use std::path::PathBuf;
use anyhow::{bail, Result};
use crate::core::search::iterative_deepening::iterative_deepening_search;
use crate::core::search::search_result::debug_search_result::DebugSearchResult;
use crate::core::search::transpositions::no_transposition::NoTranspositionTable;
use crate::input::protocol_interpreter::CalculateOptions;
use crate::tests::{check_position, epd, log_failed_positions, TestError};


const FOUR_PLY_PATH: &str = "./src/tests/assets/4ply_tests.puz";

#[test]
fn check_positions() -> Result<()> {
    let four_ply_path = PathBuf::from(FOUR_PLY_PATH);

    println!("{:?}", four_ply_path);
    let records = epd::read_puzzle(four_ply_path.as_path()).expect("failed to read puzzles");

    let mut failed_positions = vec![];
    for record in records.into_iter() {
        let result = check_position(&record, |board| {
            let mut transposition_table = NoTranspositionTable::default();
            let (result, _, _): (DebugSearchResult, _, _) = iterative_deepening_search(
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
    }

    let mut some_failed_position = failed_positions.len() > 0;
    log_failed_positions(failed_positions);

    if some_failed_position {
        bail!("Failed some positions");
    }
    Ok(())
}