// This is where the actual calculations happen.

use chess::{ChessMove, Square};

pub mod score;
pub mod evaluation_old;
pub mod evaluation;
pub mod search;

pub fn is_default_move(chess_move: &ChessMove) -> bool {
    chess_move.get_source() == Square::default() && chess_move.get_dest() == Square::default()
}
