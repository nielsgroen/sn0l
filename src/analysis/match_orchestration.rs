use std::time::{SystemTime, UNIX_EPOCH};
use chess::{Board, BoardStatus, ChessMove, Color, MoveGen, Piece};
use chess::File::G;
use clap::ValueEnum;
use serde::{Serialize, Deserialize};
use sqlx::SqlitePool;
use tokio::io::split;
use crate::analysis::database;
use crate::analysis::database::{CONFIG_TABLE, MT_SEARCH_TABLE, POSITION_SEARCH_TABLE, RUN_TABLE};
use crate::analysis::database::rows::{ConfigRow, ConspiracyMergeFn, MTSearchRow, PositionSearchRow, RunRow};
use crate::core::evaluation::game_status;
use crate::core::score::BoardEvaluation;
use crate::core::search;
use crate::core::search::conspiracy_counter::ConspiracyCounter;
use crate::core::search::conspiracy_search::merging::{merge_remove_overwritten, MergeFn};
use crate::core::search::conspiracy_search::mtd_w_conspiracy;
use crate::core::search::iterative_deepening::iterative_deepening_search;
use crate::core::search::mtdbi::{determine_mtdbi_step, mtdbi_iterative_deepening_search};
use crate::core::search::mtdf::{determine_mtdf_step, mtdf_iterative_deepening_search};
use crate::core::search::search_result::debug_search_result::DebugSearchResult;
use crate::core::search::SearchDepth;
use crate::core::search::transpositions::{EvalBound, TranspositionTable};
use crate::core::search::transpositions::high_depth_transposition::HighDepthTranspositionTable;
use crate::core::search::transpositions::no_transposition::NoTranspositionTable;
use crate::input;
use crate::input::protocol_interpreter::{CalculateOptions, Command};
use crate::input::uci_interpreter::UciInterpreter;

/// For simple automatic match playing with DB logging

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum MatchResult {
    WhiteWon,
    BlackWon,
    Draw,
    Undetermined,
}

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
    pub fn merge_fn(&self) -> Option<MergeFn> {
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
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum SearchAlgorithm {
    MTDBiIterativeDeepeningConspiracy,
    MTDFIterativeDeepeningConspiracy,
    MTDBiIterativeDeepening,
    MTDFIterativeDeepening,
    AlphaBetaIterativeDeepening,
}

impl SearchAlgorithm {
    pub fn is_conspiracy_search(&self) -> bool {
        match self {
            SearchAlgorithm::MTDBiIterativeDeepeningConspiracy => true,
            SearchAlgorithm::MTDFIterativeDeepeningConspiracy => true,
            SearchAlgorithm::MTDBiIterativeDeepening => false,
            SearchAlgorithm::MTDFIterativeDeepening => false,
            SearchAlgorithm::AlphaBetaIterativeDeepening => false,
        }
    }

    pub fn to_db_search_algorithm(&self) -> database::rows::SearchAlgorithm {
        match self {
            SearchAlgorithm::MTDBiIterativeDeepeningConspiracy => database::rows::SearchAlgorithm::MtdBi,
            SearchAlgorithm::MTDFIterativeDeepeningConspiracy => database::rows::SearchAlgorithm::MtdF,
            SearchAlgorithm::MTDBiIterativeDeepening => database::rows::SearchAlgorithm::MtdBi,
            SearchAlgorithm::MTDFIterativeDeepening => database::rows::SearchAlgorithm::MtdF,
            SearchAlgorithm::AlphaBetaIterativeDeepening => database::rows::SearchAlgorithm::AlphaBeta,
        }
    }
}

pub fn play_position(
    position: &str,
    calculate_depth: u32,
    algorithm_used: SearchAlgorithm,
    opening_name: Option<&str>,
    conspiracy_options: ConspiracySearchOptions,
    transposition_options: TranspositionOptions,
    db: &SqlitePool,
    config_id: i64,
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
    // let mut status = game_status(&board_to_play, move_gen.len() > 0);

    let position_start = SystemTime::now();

    let tokio_runtime = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    let run_row = RunRow {
        config_id,
        uci_position: position.to_string(),
        opening_name: opening_name.map(|x| x.to_string()),
        timestamp: position_start.duration_since(UNIX_EPOCH).expect("time went backwards").as_secs() as i64,
    };

    let run_db_result = tokio_runtime
        .block_on(run_row.insert(db, RUN_TABLE));
    let run_id = run_db_result.last_insert_rowid();

    let default_search_logging_fn = |mut position_row: PositionSearchRow, mut mt_rows: Vec<MTSearchRow>| {
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
    };

    match algorithm_used {
        SearchAlgorithm::MTDBiIterativeDeepeningConspiracy => {
            let (bucket_size, num_buckets, merge_fn) = unwrap_conspiracy_options(conspiracy_options);

            let _: (DebugSearchResult, _, _, _) = mtd_w_conspiracy::mtd_iterative_deepening_search(
                &board_to_play,
                &mut transposition_table,
                visited_board_hashes.clone(),
                CalculateOptions::Depth(calculate_depth),
                determine_mtdbi_step,
                bucket_size,
                num_buckets,
                merge_fn,
                default_search_logging_fn,
            );
        },
        SearchAlgorithm::MTDFIterativeDeepeningConspiracy => {
            let (bucket_size, num_buckets, merge_fn) = unwrap_conspiracy_options(conspiracy_options);

            let _: (DebugSearchResult, _, _, _) = mtd_w_conspiracy::mtd_iterative_deepening_search(
                &board_to_play,
                &mut transposition_table,
                visited_board_hashes.clone(),
                CalculateOptions::Depth(calculate_depth),
                determine_mtdf_step,
                bucket_size,
                num_buckets,
                merge_fn,
                default_search_logging_fn,
            );
        },
        SearchAlgorithm::MTDBiIterativeDeepening => {
            let _: (DebugSearchResult, _, _) = mtdbi_iterative_deepening_search(
                &board_to_play,
                &mut transposition_table,
                visited_board_hashes.clone(),
                CalculateOptions::Depth(calculate_depth),
                default_search_logging_fn,
            );
        },
        SearchAlgorithm::MTDFIterativeDeepening => {
            let _: (DebugSearchResult, _, _) = mtdf_iterative_deepening_search(
                &board_to_play,
                &mut transposition_table,
                visited_board_hashes.clone(),
                CalculateOptions::Depth(calculate_depth),
                default_search_logging_fn,
            );
        },
        SearchAlgorithm::AlphaBetaIterativeDeepening => {
            let _: (DebugSearchResult, _, _) = iterative_deepening_search(
                &board_to_play,
                &mut transposition_table,
                visited_board_hashes.clone(),
                CalculateOptions::Depth(calculate_depth),
                default_search_logging_fn,
            );
        },
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
    db: &SqlitePool,
    config_id: i64,
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

    let tokio_runtime = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    let run_row = RunRow {
        config_id,
        uci_position: position.to_string(),
        opening_name: opening_name.map(|x| x.to_string()),
        timestamp: match_start.duration_since(UNIX_EPOCH).expect("time went backwards").as_secs() as i64,
    };

    let run_db_result = tokio_runtime
        .block_on(run_row.insert(db, RUN_TABLE));
    let run_id = run_db_result.last_insert_rowid();

    let mut fifty_move_rule_counter = 0;

    while status == BoardStatus::Ongoing {
        let search_result;
        match algorithm_used {
            SearchAlgorithm::MTDBiIterativeDeepeningConspiracy => {
                let (bucket_size, num_buckets, merge_fn) = unwrap_conspiracy_options(conspiracy_options);

                let result: (DebugSearchResult, _, _, _) = mtd_w_conspiracy::mtd_iterative_deepening_search(
                    &board_to_play,
                    &mut transposition_table,
                    visited_board_hashes.clone(),
                    CalculateOptions::Depth(calculate_depth),
                    determine_mtdbi_step,
                    bucket_size,
                    num_buckets,
                    merge_fn,
                    |mut position_row: PositionSearchRow, mut mt_rows: Vec<MTSearchRow>| {
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
                    }
                );

                search_result = result.0;
            },
            SearchAlgorithm::MTDFIterativeDeepeningConspiracy => {
                let (bucket_size, num_buckets, merge_fn) = unwrap_conspiracy_options(conspiracy_options);

                let result: (DebugSearchResult, _, _, _) = mtd_w_conspiracy::mtd_iterative_deepening_search(
                    &board_to_play,
                    &mut transposition_table,
                    visited_board_hashes.clone(),
                    CalculateOptions::Depth(calculate_depth),
                    determine_mtdf_step,
                    bucket_size,
                    num_buckets,
                    merge_fn,
                    |mut position_row: PositionSearchRow, mut mt_rows: Vec<MTSearchRow>| {
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
                    }
                );

                search_result = result.0;
            },
            SearchAlgorithm::MTDBiIterativeDeepening => {
                let result: (DebugSearchResult, _, _) = mtdbi_iterative_deepening_search(
                    &board_to_play,
                    &mut transposition_table,
                    visited_board_hashes.clone(),
                    CalculateOptions::Depth(calculate_depth),
                    |mut position_row: PositionSearchRow, mut mt_rows: Vec<MTSearchRow>| {
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
                    }
                );

                search_result = result.0;
            },
            SearchAlgorithm::MTDFIterativeDeepening => {
                let result: (DebugSearchResult, _, _) = mtdf_iterative_deepening_search(
                    &board_to_play,
                    &mut transposition_table,
                    visited_board_hashes.clone(),
                    CalculateOptions::Depth(calculate_depth),
                    |mut position_row: PositionSearchRow, mut mt_rows: Vec<MTSearchRow>| {
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
                    }
                );

                search_result = result.0;
            },
            SearchAlgorithm::AlphaBetaIterativeDeepening => {
                let result: (DebugSearchResult, _, _) = iterative_deepening_search(
                    &board_to_play,
                    &mut transposition_table,
                    visited_board_hashes.clone(),
                    CalculateOptions::Depth(calculate_depth),
                    |mut position_row: PositionSearchRow, mut mt_rows: Vec<MTSearchRow>| {
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
                    }
                );

                search_result = result.0;
            },
        }

        board_to_play = board_to_play.make_move_new(search_result.best_move);

        if breaks_50_move_rule(&board_to_play, search_result.best_move) {
            fifty_move_rule_counter = 0;
        }

        visited_board_hashes.push(board_to_play.get_hash());

        current_move += 1;
        current_position.push_str(&format!(" {}", search_result.best_move));

        move_gen = MoveGen::new_legal(&board_to_play);
        status = game_status(&board_to_play, move_gen.len() > 0);

        fifty_move_rule_counter += 1;
        if fifty_move_rule_counter > 50 {
            status = BoardStatus::Stalemate;
        }
    }

    let match_result = match status {
        BoardStatus::Ongoing => MatchResult::Draw,
        BoardStatus::Stalemate => MatchResult::Draw,
        BoardStatus::Checkmate => {
            match board_to_play.side_to_move() {
                Color::White => MatchResult::BlackWon,
                Color::Black => MatchResult::WhiteWon,
            }
        }
    };

    tokio_runtime.block_on(RunRow::update_match_result(run_id, match_result, &db, RUN_TABLE));
}

/// Returns true on captures or pawn moves
pub fn breaks_50_move_rule(board: &Board, chess_move: ChessMove) -> bool {
    let source_piece = board.piece_on(chess_move.get_source());
    if source_piece.is_none() {
        return false;
    }

    let is_pawn_move = source_piece.map(|x| x == Piece::Pawn).unwrap_or(false);

    // no need to check en passant: is a pawn move
    board.piece_on(chess_move.get_dest()).is_some() || is_pawn_move
}

fn unwrap_conspiracy_options(options: ConspiracySearchOptions) -> (u32, usize, MergeFn) {
    match options{
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
    }
}
