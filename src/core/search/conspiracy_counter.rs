use std::cmp::{max, min};
use std::ops::{Add, AddAssign, Sub};
use crate::core::score::{BoardEvaluation, Centipawns};

// enum order is important for the derive Ord
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum ConspiracyValue {
    Count(u32),
    Unreachable,
}

impl Add for ConspiracyValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ConspiracyValue::Count(x), ConspiracyValue::Count(y)) => ConspiracyValue::Count(x + y),
            _ => ConspiracyValue::Unreachable,
        }
    }
}

impl Sub for ConspiracyValue {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ConspiracyValue::Count(x), ConspiracyValue::Count(y)) => ConspiracyValue::Count(x - y),
            _ => ConspiracyValue::Unreachable,
        }
    }
}

impl AddAssign for ConspiracyValue {
    fn add_assign(&mut self, rhs: Self) {
        match (*self, rhs) {
            (ConspiracyValue::Count(x), ConspiracyValue::Count(y)) => {
                *self = ConspiracyValue::Count(x + y);
            }
            _ => {
                *self = ConspiracyValue::Unreachable;
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConspiracyCounter {
    // Holds the delta-needed(i,T,V) for each bucket: the number of conspirators needed to get V
    // node i in tree T, with target value V
    pub bucket_size: u32,
    pub node_value: BoardEvaluation,
    pub up_buckets: Vec<ConspiracyValue>,
    pub down_buckets: Vec<ConspiracyValue>,
}

impl ConspiracyCounter {
    pub fn new(bucket_size: u32, num_buckets: usize) -> Self {
        if num_buckets % 2 == 0 {
            panic!("Expected uneven amount of buckets, got {num_buckets} buckets");
        }

        Self {
            bucket_size,
            node_value: BoardEvaluation::PieceScore(Centipawns::new(0)),
            up_buckets: vec![ConspiracyValue::Count(0); num_buckets],
            down_buckets: vec![ConspiracyValue::Count(0); num_buckets],
        }
    }

    pub fn from_leaf(bucket_size: u32, num_buckets: usize, value: BoardEvaluation) -> Self {
        let mut result = ConspiracyCounter::new(bucket_size, num_buckets);

        let corresponding_bucket = ConspiracyCounter::which_bucket(value, bucket_size, num_buckets);
        result.up_buckets[corresponding_bucket] = ConspiracyValue::Count(1);
        result.down_buckets[corresponding_bucket] = ConspiracyValue::Count(1);

        result.node_value = value;

        result
    }

    pub fn from_terminal_node(bucket_size: u32, num_buckets: usize, value: BoardEvaluation) -> Self {
        let mut result = ConspiracyCounter::new(bucket_size, num_buckets);

        let corresponding_bucket = ConspiracyCounter::which_bucket(value, bucket_size, num_buckets);
        result.up_buckets[corresponding_bucket] = ConspiracyValue::Unreachable;
        result.down_buckets[corresponding_bucket] = ConspiracyValue::Unreachable;

        result.node_value = value;

        result

    }

    pub fn reset(&mut self) {
        for item in &mut self.up_buckets {
            *item = ConspiracyValue::Count(0);
        }

        for item in &mut self.down_buckets {
            *item = ConspiracyValue::Count(0);
        }
    }

    pub fn merge_max_node_children(&mut self, other: &Self) {
        // assume bucket len of `other` is the same.
        let num_buckets = self.up_buckets.len();

        let new_node_value = max(self.node_value, other.node_value);

        // Setting up the up_buckets
        let mut new_up_buckets = vec![ConspiracyValue::Count(0); num_buckets];

        let mut own_cumulative_score = ConspiracyValue::Count(0);
        let mut other_cumulative_score = ConspiracyValue::Count(0);
        let mut cumulative_score = ConspiracyValue::Count(0);

        // Remember: When merging children of MAX nodes
        // take the MINIMUM of the cumulative scores for the delta-needed for values V
        // that are LARGER than the current node value (the UP-buckets)
        for i in 0..num_buckets {
            own_cumulative_score += self.up_buckets[i];
            other_cumulative_score += other.up_buckets[i];

            let new_cumulative_score = min(own_cumulative_score, other_cumulative_score);
            new_up_buckets[i] = new_cumulative_score - cumulative_score;
            cumulative_score = new_cumulative_score;
        }

        // Setting up the down_buckets
        let mut new_down_buckets = vec![ConspiracyValue::Count(0); num_buckets];

        let mut own_cumulative_score = ConspiracyValue::Count(0);
        let mut other_cumulative_score = ConspiracyValue::Count(0);
        let mut cumulative_score = ConspiracyValue::Count(0);

        // Remember: When merging children of MAX nodes
        // take the SUM of the cumulative scores for the delta-needed for values V
        // that are SMALLER than the current node value (the DOWN-buckets)
        for i in (0..num_buckets).rev() {
            own_cumulative_score += self.down_buckets[i];
            other_cumulative_score += other.down_buckets[i];

            let new_cumulative_score = own_cumulative_score + other_cumulative_score;
            new_down_buckets[i] = new_cumulative_score - cumulative_score;
            cumulative_score = new_cumulative_score;
        }

        self.node_value = new_node_value;
        self.up_buckets = new_up_buckets;
        self.down_buckets = new_down_buckets;
    }

    pub fn merge_min_node_children(&mut self, other: &Self) {
        // assume bucket len of `other` is the same.
        let num_buckets = self.up_buckets.len();

        let new_node_value = min(self.node_value, other.node_value);

        // Setting up the down_buckets
        let mut new_down_buckets = vec![ConspiracyValue::Count(0); num_buckets];

        let mut own_cumulative_score = ConspiracyValue::Count(0);
        let mut other_cumulative_score = ConspiracyValue::Count(0);
        let mut cumulative_score = ConspiracyValue::Count(0);

        // Remember: When merging children of MIN nodes
        // take the MINIMUM of the cumulative scores for the delta-needed for values V
        // that are SMALLER than the current node value (the DOWN-buckets)
        for i in (0..num_buckets).rev() {
            own_cumulative_score += self.down_buckets[i];
            other_cumulative_score += other.down_buckets[i];

            let new_cumulative_score = min(own_cumulative_score, other_cumulative_score);
            new_down_buckets[i] = new_cumulative_score - cumulative_score;
            cumulative_score = new_cumulative_score;
        }

        // Setting up the up_buckets
        let mut new_up_buckets = vec![ConspiracyValue::Count(0); num_buckets];

        let mut own_cumulative_score = ConspiracyValue::Count(0);
        let mut other_cumulative_score = ConspiracyValue::Count(0);
        let mut cumulative_score = ConspiracyValue::Count(0);

        // Remember: When merging children of MIN nodes
        // take the SUM of the cumulative scores for the delta-needed for values V
        // that are LARGER than the current node value (the UP-buckets)
        for i in 0..num_buckets {
            own_cumulative_score += self.up_buckets[i];
            other_cumulative_score += other.up_buckets[i];

            let new_cumulative_score = own_cumulative_score + other_cumulative_score;
            new_up_buckets[i] = new_cumulative_score - cumulative_score;
            cumulative_score = new_cumulative_score;
        }

        self.node_value = new_node_value;
        self.down_buckets = new_down_buckets;
        self.up_buckets = new_up_buckets;
    }

    /// Returns the index of which bucket the value corresponds to
    fn which_bucket(value: BoardEvaluation, bucket_size: u32, num_buckets: usize) -> usize {
        match value {
            BoardEvaluation::BlackMate(_) => 0,
            BoardEvaluation::PieceScore(x) => {
                // num_buckets assumed to be uneven
                let middle_bucket = num_buckets / 2;

                // The `+ bucket_size / 2` makes sure that the middle bucket is centered around 0
                let bucket_offset = (x.0 + bucket_size as i64 / 2) / bucket_size as i64;
                let bucket_index = (middle_bucket as i64 + bucket_offset);

                max(0, min(num_buckets.saturating_sub(1) as i64, bucket_index)) as usize
            },
            BoardEvaluation::WhiteMate(_) => num_buckets.saturating_sub(1),
        }
    }

}