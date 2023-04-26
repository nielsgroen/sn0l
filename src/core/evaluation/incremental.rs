use chess::{Board, ChessMove, Color, Piece, Rank, Square};
use crate::core::score::{Centipawns};
use crate::core::score::score_tables::determine_piece_score;


/// If the current color improves: score always positive
pub fn incremental_evaluation(
    board: &Board,
    chess_move: &ChessMove,
    our_color: Color,
) -> Centipawns {
    let mut result = Centipawns::new(0);

    let source_square = chess_move.get_source();
    let to_square = chess_move.get_dest();
    // let source_piece = board.piece_on(source_square).expect(&format!("Move needs to have a piece on source square, source: {source_square:?}, to: {to_square:?}"));
    let source_piece = board.piece_on(source_square);
    if let None = source_piece {
        println!("failed move {chess_move:?}, color: {our_color:?}");
        println!("failed board: {board}")
    }
    let source_piece = source_piece.expect(&format!("Move needs to have a piece on source square, source: {source_square:?}, to: {to_square:?}"));


    result += incremental_move_diff(chess_move, source_piece, our_color);

    // Castling check
    if source_piece == Piece::King {
        result += match (source_square, to_square) {
            (Square::E1, Square::G1) =>
                incremental_move_diff(
                    &ChessMove::new(Square::H1, Square::F1, None),
                    Piece::Rook,
                    our_color,
                ),
            (Square::E1, Square::C1) =>
                incremental_move_diff(
                    &ChessMove::new(Square::A1, Square::D1, None),
                    Piece::Rook,
                    our_color,
                ),
            (Square::E8, Square::G8) =>
                incremental_move_diff(
                    &ChessMove::new(Square::H8, Square::E8, None),
                    Piece::Rook,
                    our_color,
                ),
            (Square::E8, Square::C8) =>
                incremental_move_diff(
                    &ChessMove::new(Square::A8, Square::D8, None),
                    Piece::Rook,
                    our_color,
                ),
            (_, _) => Centipawns::new(0),
        }
    }

    // Check capture
    if let Some(opponent_piece) = board.piece_on(to_square) {
        // Positive, since removing from opposing color
        result += determine_piece_score(to_square, !our_color, opponent_piece);
    }

    // Check en passant
    let ep_square = board.en_passant();
    if let Some(en_passant_square) = ep_square  {
        // Check if pawn changes file, and there is no piece on target_square
        if source_piece == Piece::Pawn
            && board.piece_on(to_square) == None
            && source_square.get_file() != to_square.get_file() {

            let remove_rank = match to_square.get_rank() {
                Rank::Sixth => Rank::Fifth,
                Rank::Third => Rank::Fourth,
                _ => panic!("Taking en passant can only move to rank 6th or 3rd rank"),
            };

            let remove_square = Square::make_square(remove_rank, en_passant_square.get_file());

            // Positive, since removing from opposing color
            result += determine_piece_score(remove_square, !our_color, Piece::Pawn);
        }
    }

    result
}


/// Calculates `score target square` - `score source square`
/// Moving from a worse to a better square, is positive for either color
#[inline(always)]
fn incremental_move_diff(
    chess_move: &ChessMove,
    piece: Piece,
    color: Color
) -> Centipawns {
    let mut result = Centipawns::new(0);

    let source_square = chess_move.get_source();
    let source_piece = piece;
    let to_square = chess_move.get_dest();
    let to_piece = match chess_move.get_promotion() {
        Some(piece) => piece,
        None => source_piece,
    };

    let from_score = determine_piece_score(source_square, color, source_piece);
    result -= from_score;

    let to_score = determine_piece_score(to_square, color, to_piece);
    result += to_score;

    result
}
