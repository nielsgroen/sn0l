use chess::ChessMove;
use crate::core::score::{BoardEvaluation, Centipawns};

pub mod debug_search_result;
pub mod minimal_search_result;

pub trait SearchResult {
    fn make_search_result(
        best_move: ChessMove,
        board_evaluation: BoardEvaluation,
        nodes_searched: Option<u32>,
        critical_path: Option<Vec<ChessMove>>,
    ) -> Self;

    fn set_best_move(&mut self, chess_move: ChessMove);
    fn best_move(&self) -> ChessMove;

    fn set_board_evaluation(&mut self, board_evaluation: BoardEvaluation);
    fn board_evaluation(&self) -> BoardEvaluation;

    fn set_nodes_searched(&mut self, nodes_searched: Option<u32>);
    fn nodes_searched(&self) -> Option<u32>;

    fn set_critical_path(&mut self, critical_path: Option<Vec<ChessMove>>);
    fn prepend_move(&mut self, chess_move: ChessMove);
    fn critical_path(&self) -> Option<Vec<ChessMove>>;
}

