use std::path::PathBuf;
use clap::{Parser, ValueEnum};
use crate::analysis::chess_position::ChessPosition;
use crate::analysis::database::rows::ConspiracyMergeFn;
use crate::analysis::{match_orchestration, openings_dataset};
use crate::analysis::match_orchestration::{ConspiracySearchOptions, TranspositionOptions};
use crate::analysis::mtd_h_utils::MtdHParams;
use crate::tests::{epd, win_at_chess};
use crate::tests::win_at_chess::EPD_PATH;


#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum ChessDataset {
    WinAtChess,
    LichessOpenings,
}

impl ChessDataset {
    /// Returns a string of UCI positions
    pub fn positions(&self) -> Vec<(Option<String>, String)> {
        match self {
            ChessDataset::WinAtChess => {
                let epd_path = PathBuf::from(EPD_PATH);
                let records = epd::read_epd(epd_path.as_path()).expect("failed to read epd");

                records.into_iter()
                    .map(|x| x.uci_position())
                    .collect()
            },
            ChessDataset::LichessOpenings => {
                let records = openings_dataset::get_opening_records();

                records.into_iter()
                    .map(|x| x.uci_position())
                    .collect()
            },
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum PlayOptions {
    Match,
    Position,
}

#[derive(Parser, Debug)]
#[command(author, version)]
#[command(about = "Runs engine analysis, and stores it to a DB.")]
pub struct Args {
    /// The path to of the DB to write to.
    #[arg(short, long, default_value = "sqlite://sqlite.db")]
    pub db_path: String,

    /// Which positions to analyze.
    #[arg(short, long, default_value = "win-at-chess")]
    pub dataset: ChessDataset,

    /// Whether, for each position, to play a match or only that position.
    #[arg(short, long, default_value = "position")]
    pub play_options: PlayOptions,

    /// The depth to search to.
    #[arg(short, long, default_value_t = 6)]
    pub search_depth: u32,

    /// Which search algorithm will be used.
    #[arg(short, long, value_enum, default_value = "mtd-bi-iterative-deepening-conspiracy")]
    pub algorithm: match_orchestration::SearchAlgorithm,

    /// Determines the width of the conspiracy buckets in Centipawns.
    #[arg(short, long, default_value_t = 20)]
    bucket_size: u32,

    /// How many buckets there are for conspiracy search. Must be uneven.
    #[arg(short, long, default_value_t = 101)]
    num_buckets: usize,

    /// Which merge function to use for merging multiple mt_searches.
    #[arg(short, long, value_enum, default_value = "merge-remove-overwritten")]
    merge_fn: ConspiracyMergeFn,

    /// Disables the transposition table.
    #[arg(long, default_value_t = false)]
    neglect_transposition_table: bool,

    /// The minimum entry depth required to be considered for the transposition table.
    #[arg(long, default_value_t = 2)]
    minimum_transposition_depth: u32,

    /// The path for the mtd-h parameters.
    #[arg(long, default_value = "./python/analysis_output/optimal_params.csv")]
    pub mtd_h_params_path: String,

    /// The default distance for the distance between training and target distance for MTDH params.
    #[arg(long, default_value_t = 2)]
    pub mtd_h_training_distance: u32,
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
        if !self.neglect_transposition_table {
            TranspositionOptions::WithTransposition {
                minimum_transposition_depth: self.minimum_transposition_depth,
            }
        } else {
            TranspositionOptions::NoTransposition
        }
    }

    pub fn mtd_h_params(&self) -> Vec<MtdHParams> {
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b',')
            .has_headers(true)
            .from_path(&self.mtd_h_params_path);

        if reader.is_err() {
            return vec![];
        }

        let mut reader = reader.unwrap();

        let mut result: Vec<MtdHParams> = Vec::new();
        for record in reader.deserialize() {
            let record_result: Result<MtdHParams, _> = record;

            if record_result.is_ok() {
                result.push(record_result.ok().unwrap());
            }

            // println!("{:?}", record);
        }

        result
    }

    pub fn filtered_mtd_h_params(&self) -> Vec<MtdHParams> {
        let result = self.mtd_h_params();

        let filtered_result = result.into_iter()
            .filter(|x| x.target_depth.saturating_sub(x.training_depth) == self.mtd_h_training_distance)
            .collect();

        filtered_result
    }
}
