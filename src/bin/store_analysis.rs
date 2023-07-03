use sqlx::migrate::MigrateDatabase;
use sqlx::{Error, Sqlite};
use clap::Parser;
use sn0l::analysis::args::Args;
use sn0l::analysis::database::{CONFIG_TABLE, create_db_if_not_exists, create_tables_if_not_exists, DB_URL, MT_SEARCH_TABLE, POSITION_SEARCH_TABLE, RUN_TABLE};
use sn0l::analysis::database::rows::{ConfigRow, ConspiracyMergeFn, MTSearchRow, PositionSearchRow, RunRow};
use sn0l::analysis::match_orchestration;
use sn0l::analysis::match_orchestration::{ConspiracySearchOptions, MatchResult, play_match, TranspositionOptions};
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






    todo!();

    play_match(
        "startpos",
        search_depth,
        algorithm,
        Some("Henk opening"),
        conspiracy_search_options,
        transposition_options,
        &db,
    );

    // let dummy_config = ConfigRow {
    //     max_search_depth: 0,
    //     algorithm_used: SearchAlgorithm::AlphaBeta,
    //     conspiracy_search_used: true,
    //     bucket_size: Some(20),
    //     num_buckets: Some(101),
    //     conspiracy_merge_fn: None,
    //     transposition_table_used: false,
    //     minimum_transposition_depth: None,
    //     timestamp: 0,
    // };
    //
    // let insert_result = dummy_config.insert(&db, CONFIG_TABLE).await;
    //
    // println!("insert_result: {:?}", insert_result);
    //
    // let dummy_run = RunRow {
    //     config_id: 1,
    //     uci_position: "startpos b1c3".to_string(),
    //     opening_name: Some("THE HENK TEST OPENING".to_string()),
    //     timestamp: 0,
    // };
    //
    // let insert_result = dummy_run.insert(&db, RUN_TABLE).await;
    // println!("insert_result: {:?}", insert_result);
    //
    // let dummy_search = PositionSearchRow {
    //     run_id: 1,
    //     uci_position: "startpos g1f3".to_string(),
    //     depth: 0,
    //     time_taken: 0,
    //     nodes_evaluated: 0,
    //     evaluation: BoardEvaluation::WhiteMate(0),
    //     conspiracy_counter: Some(ConspiracyCounter::new(20, 101)),
    //     move_num: 0,
    //     timestamp: 0,
    // };
    //
    // let insert_result = dummy_search.insert(&db, POSITION_SEARCH_TABLE).await;
    // println!("insert_result: {:?}", insert_result);
    //
    // let dummy_mt_search = MTSearchRow {
    //     position_search_id: 1,
    //     test_value: BoardEvaluation::PieceScore(Centipawns::new(-14)),
    //     time_taken: 100,
    //     nodes_evaluated: 69,
    //     eval_bound: EvalBound::UpperBound(BoardEvaluation::PieceScore(Centipawns::new(-32))),
    //     conspiracy_counter: Some(ConspiracyCounter::new(20, 101)),
    //     search_num: 0,
    //     timestamp: 0,
    // };
    //
    // let insert_result = dummy_mt_search.insert(&db, MT_SEARCH_TABLE).await;
    // println!("insert_result: {:?}", insert_result);

}
