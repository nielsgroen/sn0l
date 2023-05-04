use chess::{Board, BoardStatus, ChessMove, Color};
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::draw_detection::detect_draw_incremental;
use crate::core::search::search_result::SearchResult;
use crate::core::search::transpositions::EvalBound;

pub fn check_game_over<T: SearchResult>(
    board: &Board,
    board_status: BoardStatus,
    visited_boards: &Vec<u64>,
) -> Option<T> {
    if board_status == BoardStatus::Checkmate {
        return Some(T::make_search_result(
            ChessMove::default(),
            {
                match board.side_to_move() {
                    Color::White => EvalBound::Exact(BoardEvaluation::BlackMate(1)), // black has checkmated white
                    Color::Black => EvalBound::Exact(BoardEvaluation::WhiteMate(1)),
                }
            },
            None,
            None,
        ));
    }

    if board_status == BoardStatus::Stalemate {
        return Some(T::make_search_result(
            ChessMove::default(),
            EvalBound::Exact(BoardEvaluation::PieceScore(Centipawns::new(0))),
            None,
            None,
        ));
    }

    if detect_draw_incremental(visited_boards) {
        return Some(T::make_search_result(
            ChessMove::default(),
            EvalBound::Exact(BoardEvaluation::PieceScore(Centipawns::new(0))),
            None,
            None,
        ));
    }

    None
}
