use chess::ChessMove;
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::search_result::SearchResult;
use crate::core::search::transpositions::EvalBound;

#[derive(Copy, Clone, Debug)]
pub struct MinimalSearchResult {
    best_move: ChessMove,
    board_evaluation: EvalBound,
}

impl MinimalSearchResult {
    pub fn new(
        chess_move: ChessMove,
        board_evaluation: EvalBound,
    ) -> Self {
        Self {
            best_move: chess_move,
            board_evaluation,
        }
    }
}

impl SearchResult for MinimalSearchResult {
    fn make_search_result(best_move: ChessMove, board_evaluation: EvalBound, nodes_searched: Option<u32>, critical_path: Option<Vec<ChessMove>>) -> Self {
        Self {
            best_move,
            board_evaluation,
        }
    }

    fn set_best_move(&mut self, chess_move: ChessMove) {
        self.best_move = chess_move;
    }

    fn best_move(&self) -> ChessMove {
        self.best_move
    }

    fn set_eval_bound(&mut self, board_evaluation: EvalBound) {
        self.board_evaluation = board_evaluation;
    }

    fn eval_bound(&self) -> EvalBound {
        self.board_evaluation
    }

    fn set_nodes_searched(&mut self, nodes_searched: Option<u32>) {
        ()
    }

    fn nodes_searched(&self) -> Option<u32> {
        None
    }

    fn set_critical_path(&mut self, critical_path: Option<Vec<ChessMove>>) {
        ()
    }

    fn prepend_move(&mut self, chess_move: ChessMove) {
        ()
    }

    fn critical_path(&self) -> Option<Vec<ChessMove>> {
        None
    }
}

impl Default for MinimalSearchResult {
    fn default() -> Self {
        MinimalSearchResult {
            best_move: ChessMove::default(),
            board_evaluation: EvalBound::Exact(BoardEvaluation::PieceScore(Centipawns::new(0))),
        }
    }
}
