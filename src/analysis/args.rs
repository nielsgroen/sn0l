use clap::Parser;
use crate::analysis::database::rows::ConspiracyMergeFn;
use crate::analysis::match_orchestration;
use crate::analysis::match_orchestration::{ConspiracySearchOptions, TranspositionOptions};

#[derive(Parser, Debug)]
#[command(author, version)]
#[command(about = "Runs engine analysis, and stores it to a DB.")]
pub struct Args {
    #[arg(short, long, default_value = "sqlite://sqlite.db")]
    pub db_path: String,

    #[arg(short, long, default_value_t = 6)]
    pub search_depth: u32,

    #[arg(short, long, value_enum, default_value = "mtd-bi-iterative-deepening-conspiracy")]
    pub algorithm: match_orchestration::SearchAlgorithm,

    #[arg(short, long, default_value_t = 20)]
    bucket_size: u32,

    #[arg(short, long, default_value_t = 101)]
    num_buckets: usize,

    #[arg(short, long, value_enum, default_value = "merge-remove-overwritten")]
    merge_fn: ConspiracyMergeFn,

    #[arg(short, long, default_value_t = true)]
    use_transposition: bool,

    #[arg(short, long, default_value_t = 2)]
    minimum_transposition_depth: u32,
}

impl Args {
    pub fn conspiracy_options(&self) -> ConspiracySearchOptions {
        if self.algorithm.is_conspiracy_search() {
            ConspiracySearchOptions::WithConspiracySearch {
                bucket_size: self.bucket_size,
                num_buckets: self.num_buckets,
                merge_fn_name: self.merge_fn,
            }
        } else {
            ConspiracySearchOptions::NoConspiracySearch
        }
    }

    pub fn transposition_options(&self) -> TranspositionOptions {
        if self.use_transposition {
            TranspositionOptions::WithTransposition {
                minimum_transposition_depth: self.minimum_transposition_depth,
            }
        } else {
            TranspositionOptions::NoTransposition
        }
    }
}
