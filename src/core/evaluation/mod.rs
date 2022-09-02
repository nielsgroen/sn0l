use std::ops::BitAnd;
use chess::{BitBoard, Board, BoardStatus, ChessMove, Color};
use crate::core::score::{BoardEvaluation, Centipawns, score_tables};

mod incremental;

// TODO: Make return type of searches
pub struct SearchResult {
    pub best_move: ChessMove,
    pub board_evaluation: BoardEvaluation,
    pub nodes_searched: u32,
    pub critical_path: Vec<ChessMove>, // The line of play of best moves (in reverse order, first move is at the end)
}

impl SearchResult {
    pub fn new(
        best_move: ChessMove,
        board_evaluation: BoardEvaluation,
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

impl Default for SearchResult {
    fn default() -> Self {
        Self {
            best_move: ChessMove::default(),
            board_evaluation: BoardEvaluation::PieceScore(Centipawns::new(0)),
            nodes_searched: 0,
            critical_path: Vec::new(),
        }
    }
}

pub fn single_evaluation(board: &Board) -> BoardEvaluation {
    if board.status() == BoardStatus::Checkmate {
        return match board.side_to_move() {
            Color::White => BoardEvaluation::BlackMate(0), // black has checkmated white
            Color::Black => BoardEvaluation::WhiteMate(0),
        }
    }

    if board.status() == BoardStatus::Stalemate {
        return BoardEvaluation::PieceScore(Centipawns::new(0));
    }

    let mut score = Centipawns::new(0);

    for color in chess::ALL_COLORS {
        for piece in chess::ALL_PIECES {
            let BitBoard(mut piece_positions) = board.pieces(piece).bitand(board.color_combined(color));

            'inner: for index in 0..64 {
                let square_score = score_tables::piece_table(color, piece)[index] * (piece_positions & 1);
                score += Centipawns::new(
                    match color {
                        Color::White => square_score as i64,
                        Color::Black => -(square_score as i64),
                    }
                );

                piece_positions >>= 1;
                if piece_positions == 0 {
                    break 'inner;
                }
            }
        }
    }

    BoardEvaluation::PieceScore(score)
}

/// Updates an eval so that the Mate(1) becomes Mate(2)
pub fn bubble_evaluation(evaluation: BoardEvaluation) -> BoardEvaluation {
    match evaluation {
        BoardEvaluation::WhiteMate(x) => BoardEvaluation::WhiteMate(x + 1),
        BoardEvaluation::BlackMate(x) => BoardEvaluation::BlackMate(x + 1),
        a => a,
    }
}

#[cfg(test)]
mod tests {
    use chess::Board;

    use crate::core::score::{BoardEvaluation, Centipawns};
    use super::single_evaluation;

    #[test]
    fn check_equal_startpos() {
        let board = Board::default();

        assert_eq!(single_evaluation(&board), BoardEvaluation::PieceScore(Centipawns::new(0)));
    }
}