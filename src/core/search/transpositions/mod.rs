use chess::Board;
use crate::core::score::BoardEvaluation;
use crate::core::search::{SearchDepth, SearchInfo};

pub mod hash_transposition;
pub mod no_transposition;

pub trait TranspositionTable {
    fn update(
        &mut self,
        board: &Board,
        search_depth: SearchDepth,
        evaluation: BoardEvaluation,
    );

    fn get_transposition(
        &mut self,
        board: &Board,
        minimal_search_depth: Option<SearchDepth>,
    ) -> Option<&SearchInfo>;
}


