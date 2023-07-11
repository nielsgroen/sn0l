use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use sqlx::migrate::MigrateDatabase;
use sqlx::{Error, Sqlite};
use clap::Parser;
use sn0l::analysis::args::{Args, PlayOptions};
use sn0l::analysis::database::{CONFIG_TABLE, create_db_if_not_exists, create_tables_if_not_exists, DB_URL, MT_SEARCH_TABLE, POSITION_SEARCH_TABLE, RUN_TABLE};
use sn0l::analysis::database::rows::{ConfigRow, ConspiracyMergeFn, MTSearchRow, PositionSearchRow, RunRow};
use sn0l::analysis::match_orchestration;
use sn0l::analysis::match_orchestration::{ConspiracySearchOptions, MatchResult, play_match, play_position, TranspositionOptions};
use sn0l::analysis::mtd_h_utils::{select_test_point, update_probability_distribution};
use sn0l::core::score::{BoardEvaluation, Centipawns};
use sn0l::core::search::conspiracy_counter::{ConspiracyCounter, ConspiracyValue};
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

    let mtd_params = args.filtered_mtd_h_params();

    // // TODO: remove `a`
    // let mut a = None;
    // for param in mtd_params.iter() {
    //     if param.training_depth == 4 && param.target_depth == 6 {
    //         a = Some(param.clone());
    //     }
    // }

    // let prob = a.unwrap().generate_probability_distribution()

    // let dummy_conspiracy_counter = ConspiracyCounter {
    //     bucket_size: 20,
    //     node_value: BoardEvaluation::PieceScore(Centipawns::new(-111)),
    //     up_buckets: vec![ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(5), ConspiracyValue::Count(1), ConspiracyValue::Count(0), ConspiracyValue::Count(4), ConspiracyValue::Count(0), ConspiracyValue::Count(1), ConspiracyValue::Count(1), ConspiracyValue::Count(0), ConspiracyValue::Count(2), ConspiracyValue::Count(1), ConspiracyValue::Count(1), ConspiracyValue::Count(4), ConspiracyValue::Count(8), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(1), ConspiracyValue::Count(1), ConspiracyValue::Count(0),
    //     ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(4), ConspiracyValue::Count(2), ConspiracyValue::Count(13), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0)],
    //     down_buckets: vec![ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(2), ConspiracyValue::Count(7), ConspiracyValue::Count(0), ConspiracyValue::Count(2), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(1), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(2), ConspiracyValue::Count(16), ConspiracyValue::Count(13), ConspiracyValue::Count(49), ConspiracyValue::Count(16), ConspiracyValue::Count(10), ConspiracyValue::Count(9), ConspiracyValue::Count(3), ConspiracyValue::Count(4), ConspiracyValue::Count(4), ConspiracyValue::Count(5), ConspiracyValue::Count(5), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0),
    //     ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0), ConspiracyValue::Count(0)]
    // };
    //
    // let mut prob = a.unwrap().generate_probability_distribution(&dummy_conspiracy_counter, BoardEvaluation::PieceScore(Centipawns::new(239)));
    // println!("probability distribution {:?}", prob);
    // let test_value = select_test_point(
    //     &prob,
    //     dummy_conspiracy_counter.bucket_size,
    //     BoardEvaluation::BlackMate(0),
    //     BoardEvaluation::WhiteMate(0),
    // );
    // println!("test_value {:?}", test_value);
    //
    // let test_value = EvalBound::LowerBound(BoardEvaluation::PieceScore(Centipawns::new(-11)));
    // println!("which_bucket {:?}", ConspiracyCounter::which_bucket(test_value.board_evaluation(), dummy_conspiracy_counter.bucket_size, prob.len()));
    // update_probability_distribution(&mut prob, test_value, dummy_conspiracy_counter.bucket_size);
    // println!("new probability distribution {:?}", prob);
    //
    // let test_value = select_test_point(
    //     &prob,
    //     dummy_conspiracy_counter.bucket_size,
    //     BoardEvaluation::BlackMate(0),
    //     BoardEvaluation::WhiteMate(0),
    // );
    // println!("new test_value {:?}", test_value);
    //
    // exit(0);

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
        transposition_table_used: transposition_options != TranspositionOptions::NoTransposition,
        minimum_transposition_depth: transposition_options.minimum_transposition_depth(),
        timestamp: time.duration_since(UNIX_EPOCH).expect("time went backwards").as_secs() as i64,
    };

    let config_db_result = tokio_runtime
        .block_on(config_row.insert(&db, CONFIG_TABLE));

    match args.play_options {
        PlayOptions::Match => {
            for (index, (opening_name, position)) in positions.into_iter().enumerate() {
                println!("starting match {}", index);
                play_match(
                    &position,
                    search_depth,
                    algorithm,
                    opening_name.as_deref(),
                    conspiracy_search_options,
                    transposition_options,
                    &mtd_params,
                    &db,
                    config_db_result.last_insert_rowid(),
                );
            }
        },
        PlayOptions::Position => {
            for (index, (opening_name, position)) in positions.into_iter().enumerate() {
                println!("starting position {}", index);
                play_position(
                    &position,
                    search_depth,
                    algorithm,
                    opening_name.as_deref(),
                    conspiracy_search_options,
                    transposition_options,
                    &mtd_params,
                    &db,
                    config_db_result.last_insert_rowid(),
                );
            }
        },
    }
}
