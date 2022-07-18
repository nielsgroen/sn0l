use std::fmt::{Display, Formatter};
use std::num::NonZeroU32;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};
use chess::NUM_PIECES;
use chess::BitBoard;

pub mod score_tables;

pub const PAWN_COST: Centipawns = Centipawns(100);
pub const KNIGHT_COST: Centipawns = Centipawns(300);
pub const BISHOP_COST: Centipawns = Centipawns(300);
pub const ROOK_COST: Centipawns = Centipawns(500);
pub const QUEEN_COST: Centipawns = Centipawns(900);
pub const KING_COST: Centipawns = Centipawns(1_000_000);


pub const CENTER: BitBoard = BitBoard(0x00_00_3C_3C_3C_3C_00_00);
pub const VERY_CENTER: BitBoard = BitBoard(0x00_00_00_18_18_00_00_00);

// Corresponds to Chess::ALL_PIECES
pub const PIECE_EVALUATIONS: [Centipawns; NUM_PIECES] = [
    Centipawns(100),
    Centipawns(300),
    Centipawns(300),
    Centipawns(500),
    Centipawns(900),
    Centipawns(1_000_000),
];

// Used for board evaluation, scored in 100ths of a pawn
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Centipawns(pub i64);

impl Centipawns {
    pub fn new(val: i64) -> Centipawns {
        Centipawns(val)
    }
}

impl Neg for Centipawns {
    type Output = Centipawns;

    fn neg(self) -> Self::Output {
        Centipawns::new(-self.0)
    }
}

impl Add for Centipawns {
    type Output = Centipawns;

    fn add(self, rhs: Self) -> Self::Output {
        Centipawns(self.0 + rhs.0)
    }
}

impl AddAssign for Centipawns {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0 + rhs.0;
    }
}

impl Sub for Centipawns {
    type Output = Centipawns;

    fn sub(self, rhs: Self) -> Self::Output {
        Centipawns(self.0 - rhs.0)
    }
}

impl SubAssign for Centipawns {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0 - rhs.0
    }
}

impl Display for Centipawns {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents the evaluation of a position: (instead of deprecated current player score)
/// Positive evaluations mean, white is estimated to be ahead. Black vice versa.
///
/// For Mate, a sooner checkmate is better than a later one
///
/// Example of order of most positive to least positive:
/// WhiteMate(1) > WhiteMate(+inf) > Centipawn(+inf) > Centipawn(0) > Centipawn(-inf) > BlackMate(+inf) > BlackMate(1)
///
/// `WhiteMate(0)` => White has checkmated Black.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum BoardEvaluation {
    // Make sure to keep this order, so #derive(Ord) works correctly
    BlackMate(u32),
    PieceScore(Centipawns),
    WhiteMate(u32),
}

impl Display for BoardEvaluation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return match self {
            Self::BlackMate(x) => write!(f, "-M{}", x),
            Self::PieceScore(x) if *x > Centipawns::new(0) => write!(f, "+{}", x),
            Self::PieceScore(x) => write!(f, "{}", x),
            Self::WhiteMate(x) => write!(f, "+M{}", x),
        }
    }
}


