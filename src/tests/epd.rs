use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::{Path};
use std::str::FromStr;
use chess::{Board, ChessMove};
use anyhow::Result;
use thiserror::Error;
use crate::analysis::chess_position::ChessPosition;

#[derive(Clone, Debug)]
pub struct EPDRecord {
    pub fen: String,
    pub best_move: ChessMove,
    pub id: Option<String>,
}

#[derive(Error, Debug, Copy, Clone)]
pub enum EPDParseError {
    #[error("EPD Record is empty")]
    Empty,
    #[error("EPD Record has invalid FEN")]
    InvalidFEN,
    #[error("EPD Record has invalid best move")]
    InvalidBestMove,
    #[error("EPD Record has invalid id")]
    InvalidID,
}

#[derive(Error, Debug, Copy, Clone)]
pub enum PuzzleParseError {
    #[error("Puzzle Record is empty")]
    Empty,
    #[error("Puzzle Record has invalid FEN")]
    InvalidFEN,
    #[error("Puzzle Record has invalid best move")]
    InvalidBestMove,
}

impl Display for EPDRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let fen = &self.fen;
        let best_move = self.best_move.to_string();
        let id = &self.id;

        if let Some(text) = id {
            write!(f, "{fen} bm {best_move}; id {text};")
        } else {
            write!(f, "{fen} bm {best_move};")
        }
    }
}

impl FromStr for EPDRecord {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split("; ");

        let fen_best_move = parts.next().ok_or(EPDParseError::Empty)?;
        let binding = fen_best_move.split(" ").collect::<Vec<_>>();
        let (fen, best_move) = binding.split_at(4);

        let mut fen = fen.clone().join(" ");
        fen.push_str(" 0 1");

        let best_move = best_move.get(1).ok_or(EPDParseError::InvalidBestMove)?;
        let board = Board::from_str(&fen).map_err(|_| EPDParseError::InvalidFEN)?;

        let best_move = ChessMove::from_san(&board, best_move).map_err(|_| EPDParseError::InvalidBestMove)?;

        let mut id = None;
        for part in parts {
            let mut id_part = part.split(" ");

            if let Some("id") = id_part.next() {
                id = Some(id_part.next().ok_or(EPDParseError::InvalidID)?);
            }
        }

        Ok(Self {
            fen,
            best_move,
            id: id.map(|x| x.to_string()),
        })
    }
}

impl ChessPosition for EPDRecord {
    fn uci_position(&self) -> (Option<String>, String) {
        (None, self.fen.clone())
    }
}

impl EPDRecord {
    pub fn from_puzzle(s: &str) -> Result<Self, anyhow::Error> {
        let mut parts = s.split("; ");

        let fen = parts.next().ok_or(PuzzleParseError::Empty)?;
        let mut moves = parts.next().ok_or(PuzzleParseError::InvalidBestMove)?.split(" ");

        let board = Board::from_str(fen).map_err(|_| PuzzleParseError::InvalidFEN)?;
        let best_move = ChessMove::from_san(&board, moves.next().ok_or(PuzzleParseError::InvalidBestMove)?).map_err(|_| PuzzleParseError::InvalidBestMove)?;

        let id = parts.next().map(|x| x.split(" ").skip(1).next()).flatten().map(|x| x.to_string());

        Ok(Self {
            fen: fen.to_string(),
            best_move,
            id,
        })
    }
}

pub fn read_to_epd<F>(path: &Path, parser: F) -> Result<Vec<EPDRecord>>
    where F: Fn(&str) -> Result<EPDRecord> {

    let file = File::open(path)?;
    let lines = io::BufReader::new(file).lines();

    let mut result = vec![];

    for line in lines {
        result.push(parser(&line?)?);
    }

    Ok(result)
}

pub fn read_epd(path: &Path) -> Result<Vec<EPDRecord>> {
    read_to_epd(path, EPDRecord::from_str)
}

pub fn read_puzzle(path: &Path) -> Result<Vec<EPDRecord>> {
    read_to_epd(path, EPDRecord::from_puzzle)
}
