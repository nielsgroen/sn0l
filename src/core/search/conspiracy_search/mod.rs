use std::time::Duration;
use chess::Color;
use crate::core::search::conspiracy_counter::ConspiracyCounter;
use crate::core::search::search_result::SearchResult;

pub mod mt_w_conspiracy;
pub mod mtd_w_conspiracy;
pub mod merging;

pub fn log_info_search_results<T: SearchResult>(
    search_result: &T,
    side_to_move: Color,
    duration: Duration,
    depth: u32,
    selective_depth: u32,
    conspiracy_counter: &ConspiracyCounter,
) {
    super::iterative_deepening::log_info_search_results(
        search_result,
        side_to_move,
        duration,
        depth,
        selective_depth,
    );

    log_conspiracy_counter(conspiracy_counter);
}

fn log_conspiracy_counter(conspiracy_counter: &ConspiracyCounter) {
    // Just the debug representation for now
    println!("{:?}", conspiracy_counter);
}
