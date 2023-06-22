use std::time::{SystemTime, UNIX_EPOCH};
use chess::{Board, BoardStatus, MoveGen};
use chess::File::G;
use sqlx::SqlitePool;
use tokio::io::split;
use crate::analysis::database;
use crate::analysis::database::{CONFIG_TABLE, MT_SEARCH_TABLE, POSITION_SEARCH_TABLE, RUN_TABLE};
use crate::analysis::database::rows::{ConfigRow, ConspiracyMergeFn, RunRow};
use crate::core::evaluation::game_status;
use crate::core::score::BoardEvaluation;
use crate::core::search::conspiracy_counter::ConspiracyCounter;
use crate::core::search::conspiracy_search::merging::merge_remove_overwritten;
use crate::core::search::conspiracy_search::mtd_w_conspiracy;
use crate::core::search::mtdbi::determine_mtdbi_step;
use crate::core::search::search_result::debug_search_result::DebugSearchResult;
use crate::core::search::SearchDepth;
use crate::core::search::transpositions::{EvalBound, TranspositionTable};
use crate::core::search::transpositions::high_depth_transposition::HighDepthTranspositionTable;
use crate::core::search::transpositions::no_transposition::NoTranspositionTable;
use crate::input;
use crate::input::protocol_interpreter::{CalculateOptions, Command};
use crate::input::uci_interpreter::UciInterpreter;

/// For simple automatic match playing with DB logging

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ConspiracySearchOptions {
    NoConspiracySearch,
    WithConspiracySearch {
        bucket_size: u32,
        num_buckets: usize,
        merge_fn_name: ConspiracyMergeFn,
    },
}

impl ConspiracySearchOptions {
    pub fn merge_fn(&self) -> Option<fn(&mut ConspiracyCounter, &ConspiracyCounter, &EvalBound, &EvalBound)> {
        match self {
            ConspiracySearchOptions::NoConspiracySearch => None,
            ConspiracySearchOptions::WithConspiracySearch {
                merge_fn_name,
                ..
            } => {
                match merge_fn_name {
                    ConspiracyMergeFn::MergeRemoveOverwritten => Some(merge_remove_overwritten),
                }
            }
        }
    }

    pub fn bucket_size(&self) -> Option<u32> {
        match self {
            ConspiracySearchOptions::NoConspiracySearch => None,
            ConspiracySearchOptions::WithConspiracySearch {
                bucket_size,
                ..
            } => Some(*bucket_size),
        }
    }

    pub fn num_buckets(&self) -> Option<usize> {
        match self {
            ConspiracySearchOptions::NoConspiracySearch => None,
            ConspiracySearchOptions::WithConspiracySearch {
                num_buckets,
                ..
            } => Some(*num_buckets),
        }
    }

    pub fn merge_fn_name(&self) -> Option<ConspiracyMergeFn> {
        match self {
            ConspiracySearchOptions::NoConspiracySearch => None,
            ConspiracySearchOptions::WithConspiracySearch {
                merge_fn_name,
                ..
            } => Some(*merge_fn_name),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TranspositionOptions {
    NoTransposition,
    WithTransposition {
        minimum_transposition_depth: u32,
    },
}

impl TranspositionOptions {
    pub fn minimum_transposition_depth(&self) -> Option<u32> {
        match self {
            TranspositionOptions::NoTransposition => None,
            TranspositionOptions::WithTransposition {
                minimum_transposition_depth,
            } => Some(*minimum_transposition_depth),
        }
    }
}

// TODO: keep track of the search algorithms that support DB logging
#[derive(Copy, Clone, Debug)]
pub enum SearchAlgorithm {
    MTDBiIterativeDeepeningConspiracy,
}

impl SearchAlgorithm {
    pub fn to_db_search_algorithm(&self) -> database::rows::SearchAlgorithm {
        match self {
            SearchAlgorithm::MTDBiIterativeDeepeningConspiracy => database::rows::SearchAlgorithm::MtdBi,
        }
    }
}

// TODO: don't forget the cache for visited positions
pub fn play_match(
    position: &str,
    calculate_depth: u32,
    algorithm_used: SearchAlgorithm,
    opening_name: Option<&str>,
    conspiracy_options: ConspiracySearchOptions,
    transposition_options: TranspositionOptions,
    db: &SqlitePool
) {
    let mut transposition_table: Box<dyn TranspositionTable> = match transposition_options {
        TranspositionOptions::NoTransposition => Box::new(NoTranspositionTable::default()),
        TranspositionOptions::WithTransposition {
            minimum_transposition_depth
        } => Box::new(HighDepthTranspositionTable::new(SearchDepth::Depth(minimum_transposition_depth))),
    };

    let mut current_position = position.to_string();
    let mut split = position.split_whitespace();
    let mut board_to_play = UciInterpreter::determine_board(split.clone().into_iter());
    let pre_move_board = UciInterpreter::determine_pre_move_board(split.clone().into_iter());
    let mut visited_board_hashes = UciInterpreter::determine_visited_boards(&pre_move_board, split.clone().into_iter());

    let mut current_move = 0;

    let mut move_gen = MoveGen::new_legal(&board_to_play);
    let mut status = game_status(&board_to_play, move_gen.len() > 0);

    let match_start = SystemTime::now();

    // TODO: move the config to outside of this fn
    let config_row = ConfigRow {
        max_search_depth: calculate_depth,
        algorithm_used: algorithm_used.to_db_search_algorithm(),
        conspiracy_search_used: conspiracy_options != ConspiracySearchOptions::NoConspiracySearch,
        bucket_size: conspiracy_options.bucket_size(),
        num_buckets: conspiracy_options.num_buckets().map(|x| x as u32),
        conspiracy_merge_fn: conspiracy_options.merge_fn_name(),
        transposition_table_used: transposition_options != TranspositionOptions::NoTransposition,
        minimum_transposition_depth: transposition_options.minimum_transposition_depth(),
        timestamp: match_start.duration_since(UNIX_EPOCH).expect("time went backwards").as_secs() as i64,
    };

    let tokio_runtime = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    let config_db_result = tokio_runtime
        .block_on(config_row.insert(db, CONFIG_TABLE));

    let run_row = RunRow {
        config_id: config_db_result.last_insert_rowid(),
        uci_position: position.to_string(),
        opening_name: opening_name.map(|x| x.to_string()),
        timestamp: match_start.duration_since(UNIX_EPOCH).expect("time went backwards").as_secs() as i64,
    };

    let run_db_result = tokio_runtime
        .block_on(run_row.insert(db, RUN_TABLE));
    let run_id = run_db_result.last_insert_rowid();

    while status == BoardStatus::Ongoing {
        match algorithm_used {
            SearchAlgorithm::MTDBiIterativeDeepeningConspiracy => {
                if conspiracy_options == ConspiracySearchOptions::NoConspiracySearch {
                    panic!("No Conspiracy options set for a conspiracy search");
                }
                let (bucket_size, num_buckets, merge_fn) = match conspiracy_options {
                    ConspiracySearchOptions::NoConspiracySearch => panic!("No conspiracy options set for conspiracy search"),
                    ConspiracySearchOptions::WithConspiracySearch {
                        bucket_size,
                        num_buckets,
                        merge_fn_name,
                    } => {
                        (
                            bucket_size,
                            num_buckets,
                            match merge_fn_name {
                                ConspiracyMergeFn::MergeRemoveOverwritten => merge_remove_overwritten,
                            },
                        )
                    }
                };

                let result: (DebugSearchResult, _, _, _) = mtd_w_conspiracy::mtd_iterative_deepening_search(
                    &board_to_play,
                    &mut transposition_table,
                    visited_board_hashes.clone(),
                    CalculateOptions::Depth(calculate_depth),
                    determine_mtdbi_step,
                    bucket_size,
                    num_buckets,
                    merge_fn,
                    |mut position_row, mut mt_rows| {
                        position_row.run_id = run_id;
                        position_row.uci_position = current_position.clone();
                        position_row.move_num = current_move;

                        let position_db_result = tokio_runtime
                            .block_on(position_row.insert(db, POSITION_SEARCH_TABLE));

                        let position_id = position_db_result.last_insert_rowid();

                        for mt_row in mt_rows.iter_mut() {
                            mt_row.position_search_id = position_id;

                            tokio_runtime.block_on(mt_row.insert(db, MT_SEARCH_TABLE));
                        }
                    },
                );

                let search_result = result.0;

                board_to_play = board_to_play.make_move_new(search_result.best_move);
                visited_board_hashes.push(board_to_play.get_hash());

                current_move += 1;
                current_position.push_str(&format!(" {}", search_result.best_move));

                move_gen = MoveGen::new_legal(&board_to_play);
                status = game_status(&board_to_play, move_gen.len() > 0);
            }
        }
    }
}


