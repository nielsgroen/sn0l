use std::cmp::{max, min};
use crate::core::score::BoardEvaluation;
use crate::core::search::conspiracy_counter::{ConspiracyCounter, ConspiracyValue};
use crate::core::search::transpositions::EvalBound;

/// This module contains the multiple merging strategies for
/// merging the ConspiracyCounters of the MT searches at the same depth.

pub fn merge_keep_everything(
    old_conspiracy_counter: &mut ConspiracyCounter,
    new_conspiracy_counter: &ConspiracyCounter,
    old_eval: &EvalBound,
    new_eval: &EvalBound,
) {
    let mut upper_bound = None;
    let mut lower_bound = None;

    match (old_eval, new_eval) {
        (EvalBound::Exact(x), _) => {
            upper_bound = Some(x.clone());
            lower_bound = Some(x.clone());
        },
        (_, EvalBound::Exact(y)) => {
            upper_bound = Some(y.clone());
            lower_bound = Some(y.clone());
        },
        (EvalBound::UpperBound(x), EvalBound::UpperBound(y)) => {
            upper_bound = Some(min(*x, *y));
        },
        (EvalBound::UpperBound(x), EvalBound::LowerBound(y)) => {
            upper_bound = Some(x.clone());
            lower_bound = Some(y.clone());
        },
        (EvalBound::LowerBound(x), EvalBound::UpperBound(y)) => {
            upper_bound = Some(y.clone());
            lower_bound = Some(x.clone());
        },
        (EvalBound::LowerBound(x), EvalBound::LowerBound(y)) => {
            lower_bound = Some(max(*x, *y));
        },
    }

    upper_bound.get_or_insert(BoardEvaluation::WhiteMate(0));
    lower_bound.get_or_insert(BoardEvaluation::BlackMate(0));
    let upper_bound = upper_bound.unwrap();
    let lower_bound = lower_bound.unwrap();

    let num_buckets = old_conspiracy_counter.up_buckets.len();

    // Iterate over the up buckets
    for i in 0..num_buckets {
        let (_, bucket_upperbound) = ConspiracyCounter::bucket_bounds(i, old_conspiracy_counter.bucket_size, num_buckets);

        old_conspiracy_counter.up_buckets[i] += new_conspiracy_counter.up_buckets[i];

        if bucket_upperbound < lower_bound {
            old_conspiracy_counter.up_buckets[i] = ConspiracyValue::Count(0);
        }
    }

    // Iterate over the down buckets
    for i in 0..num_buckets {
        let (bucket_lowerbound, _) = ConspiracyCounter::bucket_bounds(i, old_conspiracy_counter.bucket_size, num_buckets);

        old_conspiracy_counter.down_buckets[i] += new_conspiracy_counter.down_buckets[i];

        if upper_bound < bucket_lowerbound {
            old_conspiracy_counter.down_buckets[i] = ConspiracyValue::Count(0);
        }
    }
}
