use std::ops::BitAnd;
use chess::{BitBoard, Board, BoardStatus, Color};
use crate::core::score::{BoardEvaluation, Centipawns, score_tables};

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
