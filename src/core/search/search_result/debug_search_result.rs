use chess::ChessMove;
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::search_result::SearchResult;
use crate::core::search::transpositions::EvalBound;

#[derive(Clone, Debug)]
pub struct DebugSearchResult {
    pub best_move: ChessMove,
    pub board_evaluation: EvalBound,
    pub nodes_searched: u32,
    pub critical_path: Vec<ChessMove>, // The line of play of best moves (in reverse order, first move is at the end)
}

impl DebugSearchResult {
    pub fn new(
        best_move: ChessMove,
        board_evaluation: EvalBound,
        nodes_searched: Option<u32>,
        critical_path: Option<Vec<ChessMove>>,
    ) -> Self {
        Self {
            best_move,
            board_evaluation,
            nodes_searched: nodes_searched.unwrap_or(1),
            critical_path: critical_path.unwrap_or(Vec::new()),
        }
    }
}

impl SearchResult for DebugSearchResult {
    fn make_search_result(best_move: ChessMove, board_evaluation: EvalBound, nodes_searched: Option<u32>, critical_path: Option<Vec<ChessMove>>) -> Self {
        Self::new(
            best_move,
            board_evaluation,
            nodes_searched,
            critical_path,
        )
    }

    fn set_best_move(&mut self, chess_move: ChessMove) {
        self.best_move = chess_move;
    }

    fn best_move(&self) -> ChessMove {
        self.best_move
    }

    fn set_board_evaluation(&mut self, board_evaluation: EvalBound) {
        self.board_evaluation = board_evaluation;
    }

    fn board_evaluation(&self) -> EvalBound {
        self.board_evaluation
    }

    fn set_nodes_searched(&mut self, nodes_searched: Option<u32>) {
        self.nodes_searched = nodes_searched.unwrap_or(1);
    }

    fn nodes_searched(&self) -> Option<u32> {
        Some(self.nodes_searched)
    }

    fn set_critical_path(&mut self, critical_path: Option<Vec<ChessMove>>) {
        self.critical_path = critical_path.unwrap_or(Vec::new());
    }

    fn prepend_move(&mut self, chess_move: ChessMove) {
        self.critical_path.push(chess_move);
    }

    fn critical_path(&self) -> Option<Vec<ChessMove>> {
        Some(self.critical_path.clone())
        // Some(self.critical_path.clone().into_iter().rev().collect())
    }
}

impl Default for DebugSearchResult {
    fn default() -> Self {
        Self {
            best_move: ChessMove::default(),
            board_evaluation: EvalBound::Exact(BoardEvaluation::PieceScore(Centipawns::new(0))),
            nodes_searched: 0,
            critical_path: Vec::new(),
        }
    }
}
