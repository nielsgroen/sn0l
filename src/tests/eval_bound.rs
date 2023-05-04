use anyhow::{bail, Result};
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::transpositions::EvalBound;

#[test]
fn check_eval_bound() {
    assert_eq!(EvalBound::UpperBound(BoardEvaluation::PieceScore(Centipawns::new(-870))) >= EvalBound::Exact(BoardEvaluation::BlackMate(0)), true);
}