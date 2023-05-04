use chess::{Board, ChessMove};
use crate::core::search::{SearchDepth, SearchInfo};
use crate::core::search::transpositions::{EvalBound, TranspositionTable};

/// A TranspositionTable implementation that is always empty
#[derive(Copy, Clone, Debug, Default)]
pub struct NoTranspositionTable;

impl TranspositionTable for NoTranspositionTable {
    fn update(&mut self, board: &Board, search_depth: SearchDepth, evaluation: EvalBound, best_move: ChessMove, prime_variation: Option<Vec<ChessMove>>) {
        ()
    }

    fn get_transposition(&mut self, board: &Board, minimal_search_depth: Option<SearchDepth>) -> Option<&SearchInfo> {
        None
    }
}
