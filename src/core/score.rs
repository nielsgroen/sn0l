use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};
use chess::NUM_PIECES;


pub const PAWN_COST: Centipawns = Centipawns(100);
pub const KNIGHT_COST: Centipawns = Centipawns(300);
pub const BISHOP_COST: Centipawns = Centipawns(300);
pub const ROOK_COST: Centipawns = Centipawns(500);
pub const QUEEN_COST: Centipawns = Centipawns(900);
pub const KING_COST: Centipawns = Centipawns(1_000_000);

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
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
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