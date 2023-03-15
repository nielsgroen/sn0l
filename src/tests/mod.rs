use chess::ChessMove;
use thiserror::Error;

#[cfg(test)]
pub mod win_at_chess;
pub mod epd;


#[derive(Error, Debug, Copy, Clone)]
pub enum TestError {
    #[error("Engine made the wrong move (expected {expected}, got {actual})")]
    WrongMove {
        expected: ChessMove,
        actual: ChessMove,
    },
}
