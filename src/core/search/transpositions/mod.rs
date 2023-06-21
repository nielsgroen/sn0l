use std::cmp::Ordering;
use chess::{Board, ChessMove};
use serde::{Serialize, Deserialize};
use crate::core::score::BoardEvaluation;
use crate::core::search::{SearchDepth, SearchInfo};

pub mod hash_transposition;
pub mod no_transposition;
pub mod high_depth_transposition;

pub trait TranspositionTable {
    fn update(
        &mut self,
        board: &Board,
        search_depth: SearchDepth,
        evaluation: EvalBound,
        best_move: ChessMove,
        prime_variation: Option<Vec<ChessMove>>,
    );

    fn get_transposition(
        &mut self,
        board: &Board,
        minimal_search_depth: Option<SearchDepth>,
    ) -> Option<&SearchInfo>;
}


#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum EvalBound {
    UpperBound(BoardEvaluation),
    Exact(BoardEvaluation),
    LowerBound(BoardEvaluation),
}

impl EvalBound {
    pub fn board_evaluation(&self) -> BoardEvaluation {
        match self {
            EvalBound::UpperBound(a) => *a,
            EvalBound::Exact(a) => *a,
            EvalBound::LowerBound(a) => *a,
        }
    }

    pub fn set_board_evaluation(&mut self, board_evaluation: BoardEvaluation) {
        *self = match self.clone() {
            EvalBound::UpperBound(_) => EvalBound::UpperBound(board_evaluation),
            EvalBound::Exact(_) => EvalBound::Exact(board_evaluation),
            EvalBound::LowerBound(_) => EvalBound::LowerBound(board_evaluation),
        };
    }
}

impl PartialOrd for EvalBound {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (EvalBound::UpperBound(x), EvalBound::UpperBound(y)) => Some(x.cmp(y)),
            (EvalBound::UpperBound(x), EvalBound::Exact(y)) => {
                if x <= y {
                    Some(Ordering::Less)
                } else {
                    None
                }
            },
            (EvalBound::UpperBound(x), EvalBound::LowerBound(y)) => {
                if x <= y {
                    Some(Ordering::Less)
                } else {
                    None
                }
            },
            (EvalBound::Exact(x), EvalBound::UpperBound(y)) => {
                if x >= y {
                    Some(Ordering::Greater)
                } else {
                    None
                }
            },
            (EvalBound::Exact(x), EvalBound::Exact(y)) => {
                Some(x.cmp(y))
            },
            (EvalBound::Exact(x), EvalBound::LowerBound(y)) => {
                if x <= y {
                    Some(Ordering::Less)
                } else {
                    None
                }
            },
            (EvalBound::LowerBound(x), EvalBound::LowerBound(y)) => Some(x.cmp(y)),
            (EvalBound::LowerBound(x), EvalBound::Exact(y)) => {
                if x >= y {
                    Some(Ordering::Greater)
                } else {
                    None
                }
            },
            (EvalBound::LowerBound(x), EvalBound::UpperBound(y)) => {
                if x >= y {
                    Some(Ordering::Greater)
                } else {
                    None
                }
            },
        }
    }
}
