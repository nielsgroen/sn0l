use std::cmp::{max, min};
use serde::Deserialize;
use crate::core::score::BoardEvaluation;
use crate::core::search::conspiracy_counter::{ConspiracyCounter, ConspiracyValue};
use crate::core::search::mtd::avg_bounds;
use crate::core::search::transpositions::EvalBound;

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct MtdHParams {
    pub training_depth: u32,
    pub target_depth: u32,
    pub p: f64,
    pub w_side_down: f64,
    pub w_side_up: f64,
    pub c: f64,
}


impl MtdHParams {
    /// Returns the probability for each bucket
    pub fn generate_probability_distribution(&self, conspiracy_counter: &ConspiracyCounter, previous_evaluation: BoardEvaluation) -> Vec<f64> {
        let num_buckets = conspiracy_counter.up_buckets.len();

        let mut up_probabilities = vec![0.0; num_buckets];
        let mut down_probabilities = vec![0.0; num_buckets];


        // handle checkmates first
        match previous_evaluation {
            BoardEvaluation::BlackMate(_) => {
                up_probabilities[0] = 0.5;
                down_probabilities[0] = 0.5;
            },
            BoardEvaluation::WhiteMate(_) => {
                up_probabilities[num_buckets.saturating_sub(1)] = 0.5;
                down_probabilities[num_buckets.saturating_sub(1)] = 0.5;
            },
            _ => {
                let mut cumulative_value = ConspiracyValue::Count(0);
                for (index, up_value) in conspiracy_counter.up_buckets.iter().enumerate() {
                    up_probabilities[index] = self.bucket_probability_up(index, *up_value, cumulative_value, num_buckets);
                    cumulative_value += *up_value;
                }

                let mut cumulative_value = ConspiracyValue::Count(0);
                for (index, down_value) in conspiracy_counter.down_buckets.iter().enumerate().rev() {
                    down_probabilities[index] = self.bucket_probability_down(index, *down_value, cumulative_value, num_buckets);
                    cumulative_value += *down_value;
                }
            },
        }

        let probabilities = up_probabilities.into_iter()
            .zip(down_probabilities.into_iter())
            .map(|(p, q)| p + q)
            .collect::<Vec<_>>();

        let area: f64 = probabilities.iter().sum();

        // Make the area of the probability distribution 1.
        let probabilities = probabilities.into_iter()
            .map(|x| x / area)
            .collect::<Vec<_>>();

        probabilities
    }

    pub fn bucket_probability_up(&self, index: usize, marginal_value: ConspiracyValue, cumulative_value: ConspiracyValue, num_buckets: usize) -> f64 {
        let mut marginal_value = marginal_value;
        if index == num_buckets.saturating_sub(1) {
            marginal_value = ConspiracyValue::Unreachable;
        }

        if marginal_value.is_zero() && cumulative_value.is_zero() {
            return 0.0;
        }

        let cumulative_val;
        match cumulative_value {
            ConspiracyValue::Count(x) => {
                cumulative_val = x;
            },
            ConspiracyValue::Unreachable => {
                return 0.0;
            }
        }

        match marginal_value {
            ConspiracyValue::Count(x) => {
                self.w_side_up * (1.0 - self.p.powi(x as i32)) * self.p.powi(cumulative_val as i32) + self.c
            },
            ConspiracyValue::Unreachable => {
                self.w_side_up * self.p.powi(cumulative_val as i32) + self.c
            },
        }
    }

    pub fn bucket_probability_down(&self, index: usize, marginal_value: ConspiracyValue, cumulative_value: ConspiracyValue, num_buckets: usize) -> f64 {
        let mut marginal_value = marginal_value;
        if index == num_buckets.saturating_sub(1) {
            marginal_value = ConspiracyValue::Unreachable;
        }

        if marginal_value.is_zero() && cumulative_value.is_zero() {
            return 0.0;
        }

        let cumulative_val;
        match cumulative_value {
            ConspiracyValue::Count(x) => {
                cumulative_val = x;
            },
            ConspiracyValue::Unreachable => {
                return 0.0;
            }
        }

        match marginal_value {
            ConspiracyValue::Count(x) => {
                self.w_side_down * (1.0 - self.p.powi(x as i32)) * self.p.powi(cumulative_val as i32) + self.c
            },
            ConspiracyValue::Unreachable => {
                self.w_side_down * self.p.powi(cumulative_val as i32) + self.c
            },
        }
    }

    pub fn find_applicable_param(param_list: &[Self], target_depth: u32) -> Option<&Self> {
        let suitable_params = param_list.into_iter()
            .filter(|x| x.target_depth == target_depth)
            .collect::<Vec<_>>();

        suitable_params.get(0).copied()
    }
}

pub fn select_test_point(probability_distribution: &[f64], bucket_size: u32, lowerbound: BoardEvaluation, upperbound: BoardEvaluation) -> BoardEvaluation {
    let num_buckets = probability_distribution.len();

    // get the index of the halfway point of the cumulative distribution
    let cumulative_distribution = probability_distribution.into_iter()
        .scan(0.0, |accumulator, x| {
            *accumulator = *accumulator + *x;

            Some(accumulator.clone())
        })
        .collect::<Vec<_>>();

    for (index, cumulative_score) in cumulative_distribution.into_iter().enumerate() {
        if cumulative_score > 0.5 {
            let (mut bucket_lowerbound, mut bucket_upperbound) = ConspiracyCounter::bucket_bounds(
                index,
                bucket_size,
                num_buckets,
            );

            if index == 0 {
                bucket_lowerbound = BoardEvaluation::BlackMate(0);
            } else if index == num_buckets.saturating_sub(1) {
                bucket_upperbound = BoardEvaluation::WhiteMate(0);
            }

            bucket_lowerbound = min(bucket_lowerbound, upperbound);
            bucket_upperbound = max(bucket_upperbound, lowerbound);

            let test_lowerbound = max(bucket_lowerbound, lowerbound);
            let test_upperbound = min(bucket_upperbound, upperbound);

            // account for unstable search
            // if test_lowerbound > test_upperbound {
            //     panic!("select_test_point test_lowerbound higher than test_upperbound: index {}, bucket_lowerbound {:?}, lowerbound {:?}, bucket_upperbound, {:?}, upperbound {:?}", index, bucket_lowerbound, lowerbound, bucket_upperbound, upperbound);
            // }
            let test_point = avg_bounds(test_lowerbound, test_upperbound);

            return test_point;
        }
    }

    avg_bounds(lowerbound, upperbound)
    // panic!("no bucket with cumulative score of > 0.5. {:?}", probability_distribution);
}

pub fn select_test_point_w_mate(
    probability_distribution: &[f64],
    bucket_size: u32,
    lowerbound: BoardEvaluation,
    upperbound: BoardEvaluation,
    last_evaluation: BoardEvaluation,
) -> BoardEvaluation {

    match last_evaluation {
        BoardEvaluation::BlackMate(_) => {
            last_evaluation
        },
        BoardEvaluation::PieceScore(_) => {
            select_test_point(
                &probability_distribution,
                bucket_size,
                lowerbound,
                upperbound,
            )
        },
        BoardEvaluation::WhiteMate(_) => {
            last_evaluation
        },
    }
}


pub fn update_probability_distribution(probability_distribution: &mut [f64], boundary: EvalBound, bucket_size: u32) {
    let num_buckets = probability_distribution.len();

    let target_bucket = ConspiracyCounter::which_bucket(boundary.board_evaluation(), bucket_size, num_buckets);

    let remove_lower;
    let remove_upper;
    match boundary {
        EvalBound::UpperBound(_) => {
            remove_lower = false;
            remove_upper = true;
        },
        EvalBound::Exact(_) => {
            remove_lower = true;
            remove_upper = true;
        },
        EvalBound::LowerBound(_) => {
            remove_lower = true;
            remove_upper = false;
        },
    }

    // for (index, bucket) in probability_distribution.into_iter().enumerate() {
    for index in 0..num_buckets {
        if index < target_bucket && remove_lower {
            probability_distribution[index] = 0.0;
        }
        if index > target_bucket && remove_upper {
            probability_distribution[index] = 0.0;
        }
    }

    let new_area: f64 = probability_distribution.iter().sum();

    for bucket in probability_distribution.into_iter() {
        *bucket = *bucket / new_area;
    }
}
