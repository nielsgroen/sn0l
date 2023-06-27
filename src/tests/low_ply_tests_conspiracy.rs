use std::path::PathBuf;
use anyhow::{bail, Result};
use crate::core::search::conspiracy_search::merging::merge_remove_overwritten;
use crate::core::search::conspiracy_search::mtd_w_conspiracy::mtd_iterative_deepening_search;
use crate::core::search::mtdbi::determine_mtdbi_step;
use crate::core::search::search_result::debug_search_result::DebugSearchResult;
use crate::core::search::SearchDepth;
use crate::core::search::transpositions::high_depth_transposition::HighDepthTranspositionTable;
use crate::core::search::transpositions::no_transposition::NoTranspositionTable;
use crate::core::search::transpositions::TranspositionTable;
use crate::input::protocol_interpreter::CalculateOptions;
use crate::tests::{check_position, epd, log_failed_positions, TestError};


const FOUR_PLY_PATH: &str = "./src/tests/assets/4ply_tests.puz";
const BUCKET_SIZE: u32 = 20;
const NUM_BUCKETS: usize = 101;

#[test]
fn check_positions() -> Result<()> {
    let four_ply_path = PathBuf::from(FOUR_PLY_PATH);

    println!("{:?}", four_ply_path);
    let records = epd::read_puzzle(four_ply_path.as_path()).expect("failed to read puzzles");

    let mut failed_positions = vec![];
    for record in records.into_iter() {
        let result = check_position(&record, |board| {
            let mut transposition_table: Box<dyn TranspositionTable> = Box::new(HighDepthTranspositionTable::new(SearchDepth::Depth(2)));
            let (result, conspiracy_counter, _, _) = mtd_iterative_deepening_search(
                board,
                &mut transposition_table,
                vec![],
                CalculateOptions::Depth(6),
                determine_mtdbi_step,
                BUCKET_SIZE,
                NUM_BUCKETS,
                merge_remove_overwritten,
                |_, _| {},
            );

            println!("{result:?}");
            println!("Conspiracy counter: {:?}", conspiracy_counter);

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