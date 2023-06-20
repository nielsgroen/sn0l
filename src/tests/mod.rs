use std::str::FromStr;
use chess::{Board, ChessMove};
use thiserror::Error;
use crate::core::search::search_result::debug_search_result::DebugSearchResult;
use crate::tests::epd::{EPDParseError, EPDRecord};

mod win_at_chess;
mod low_ply_tests;
mod epd;
mod eval_bound;
mod mt_alpha_beta_equivalence;
mod mtdf_alpha_beta_equivalence;
mod mtdbi_alpha_beta_equivalence;
mod conspiracy_counter;
mod low_ply_tests_conspiracy;


#[derive(Error, Debug, Copy, Clone)]
pub enum TestError {
    #[error("Engine made the wrong move (expected {expected}, got {actual})")]
    WrongMove {
        expected: ChessMove,
        actual: ChessMove,
    },
}

pub fn check_position<F>(record: &EPDRecord, search_method: F) -> Result<(), TestError>
    where F: Fn(&Board) -> DebugSearchResult {

    if let Some(record_id) = record.id.clone() {
        println!("{record_id}");
    }

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

pub fn log_failed_positions(positions: impl IntoIterator<Item = (Option<String>, ChessMove, ChessMove)>) {
    for position in positions.into_iter() {
        let id = position.0;
        let expected = position.1;
        let actual = position.2;
        println!("Failed {id:?}, (expected {expected}, got {actual})");
    }
}
