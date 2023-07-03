use std::time::{SystemTime, UNIX_EPOCH};
use sqlx::migrate::MigrateDatabase;
use sqlx::{Error, Sqlite};
use clap::Parser;
use sn0l::analysis::args::{Args, PlayOptions};
use sn0l::analysis::database::{CONFIG_TABLE, create_db_if_not_exists, create_tables_if_not_exists, DB_URL, MT_SEARCH_TABLE, POSITION_SEARCH_TABLE, RUN_TABLE};
use sn0l::analysis::database::rows::{ConfigRow, ConspiracyMergeFn, MTSearchRow, PositionSearchRow, RunRow};
use sn0l::analysis::match_orchestration;
use sn0l::analysis::match_orchestration::{ConspiracySearchOptions, MatchResult, play_match, play_position, TranspositionOptions};
use sn0l::core::score::{BoardEvaluation, Centipawns};
use sn0l::core::search::conspiracy_counter::ConspiracyCounter;
use sn0l::core::search::transpositions::EvalBound;

/// This executable is for performing analysis on chess games, and storing those to the DB.
/// It will run a certain configuration, and write everything to an sqlite file


// TODO--------------------------------
// TODO: make conspiracy_search work with TT when researching at lower depths
// when playing a game where you already searched at a given depth on a previous turn: keep that conspiracy_search around!
// this is important for determining the test_values

// #[tokio::main]
fn main() {
    let args = Args::parse();

    let db_path = &args.db_path;
    let search_depth = args.search_depth;
    let algorithm = args.algorithm;
    let conspiracy_search_options = args.conspiracy_options();
    let transposition_options = args.transposition_options();

    let tokio_runtime = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let db = tokio_runtime.block_on(create_db_if_not_exists(db_path));

    tokio_runtime.block_on(create_tables_if_not_exists(&db));

    let positions = args.dataset.positions();

    let time = SystemTime::now();
    let config_row = ConfigRow {
        max_search_depth: search_depth,
        algorithm_used: algorithm.to_db_search_algorithm(),
        conspiracy_search_used: algorithm.is_conspiracy_search(),
        bucket_size: conspiracy_search_options.bucket_size(),
        num_buckets: conspiracy_search_options.num_buckets().map(|x| x as u32),
        conspiracy_merge_fn: conspiracy_search_options.merge_fn_name(),
        transposition_table_used: transposition_options == TranspositionOptions::NoTransposition,
        minimum_transposition_depth: transposition_options.minimum_transposition_depth(),
        timestamp: time.duration_since(UNIX_EPOCH).expect("time went backwards").as_secs() as i64,
    };

    let config_db_result = tokio_runtime
        .block_on(config_row.insert(&db, CONFIG_TABLE));

    match args.play_options {
        PlayOptions::Match => {
            for (opening_name, position) in positions.into_iter() {
                play_match(
                    &position,
                    search_depth,
                    algorithm,
                    opening_name.as_deref(),
                    conspiracy_search_options,
                    transposition_options,
                    &db,
                    config_db_result.last_insert_rowid(),
                );
            }
        },
        PlayOptions::Position => {
            for (opening_name, position) in positions.into_iter() {
                play_position(
                    &position,
                    search_depth,
                    algorithm,
                    opening_name.as_deref(),
                    conspiracy_search_options,
                    transposition_options,
                    &db,
                    config_db_result.last_insert_rowid(),
                );
            }
        },
    }
}
