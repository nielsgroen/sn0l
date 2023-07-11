use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize};
use clap::ValueEnum;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::SqlitePool;
use crate::analysis::database::CONFIG_TABLE;
use crate::analysis::match_orchestration::MatchResult;
use crate::core::score::BoardEvaluation;
use crate::core::search::conspiracy_counter::ConspiracyCounter;
use crate::core::search::transpositions::EvalBound;

/// The file that contains all the row types for inserting into each table of the DB

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SearchAlgorithm {
    AlphaBeta,
    MtdBi,
    MtdF,
    MtdH,
}

impl Display for SearchAlgorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchAlgorithm::AlphaBeta => write!(f, "AlphaBeta"),
            SearchAlgorithm::MtdBi => write!(f, "MtdBi"),
            SearchAlgorithm::MtdF => write!(f, "MtdF"),
            SearchAlgorithm::MtdH => write!(f, "MtdH"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
pub enum ConspiracyMergeFn {
    MergeRemoveOverwritten,
}

impl Display for ConspiracyMergeFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {

        match self {
            ConspiracyMergeFn::MergeRemoveOverwritten => write!(f, "MergeRemoveOverwritten"),
        }
    }
}

// Can't use async in trait: No trait for now :(
// pub trait DBRow {
//     async fn insert(&self, db: &SqlitePool, table_name: &str) -> SqliteQueryResult;
// }


#[derive(Copy, Clone, Debug)]
pub struct ConfigRow {
    // id omitted: provided by DB
    pub max_search_depth: u32,
    pub algorithm_used: SearchAlgorithm,
    pub conspiracy_search_used: bool,
    pub bucket_size: Option<u32>,
    pub num_buckets: Option<u32>,
    pub conspiracy_merge_fn: Option<ConspiracyMergeFn>,
    pub transposition_table_used: bool,
    pub minimum_transposition_depth: Option<u32>,
    pub timestamp: i64,
}

impl ConfigRow {
    pub async fn insert(&self, db: &SqlitePool, table_name: &str) -> SqliteQueryResult {
        let result = sqlx::query(&format!(r"
            INSERT INTO {} (
                max_search_depth,
                algorithm_used,
                conspiracy_search_used,
                bucket_size,
                num_buckets,
                conspiracy_merge_fn,
                transposition_table_used,
                minimum_transposition_depth,
                timestamp
            ) VALUES (
                ?,
                ?,
                ?,
                ?,
                ?,
                ?,
                ?,
                ?,
                ?
            );
        ", table_name))
            .bind(self.max_search_depth)
            .bind(self.algorithm_used.to_string())
            .bind(self.conspiracy_search_used as u32)
            .bind(self.bucket_size)
            .bind(self.num_buckets)
            .bind(self.conspiracy_merge_fn.map(|x| x.to_string()))
            .bind(self.transposition_table_used as u32)
            .bind(self.minimum_transposition_depth)
            .bind(self.timestamp)
            .execute(db)
            .await
            .unwrap();

        result
    }
}

#[derive(Clone, Debug)]
pub struct RunRow {
    pub config_id: i64,
    pub uci_position: String,
    pub opening_name: Option<String>,
    pub timestamp: i64,
}

impl RunRow {
    pub async fn insert(&self, db: &SqlitePool, table_name: &str) -> SqliteQueryResult {
        let result = sqlx::query(&format!(r"
            INSERT INTO {} (
                config_id,
                uci_position,
                opening_name,
                timestamp
            ) VALUES (
                ?,
                ?,
                ?,
                ?
            );
        ", table_name))
            .bind(self.config_id)
            .bind(self.uci_position.clone())
            .bind(self.opening_name.clone())
            .bind(self.timestamp)
            .execute(db)
            .await
            .unwrap();

        result
    }

    pub async fn update_match_result(id: i64, match_result: MatchResult, db: &SqlitePool, table_name: &str) {
        let _ = sqlx::query(&format!(r"
            UPDATE {}
            SET match_result = ?
            WHERE id = ?;
        ", table_name))
            .bind(format!("{:?}", match_result))
            .bind(id)
            .execute(db)
            .await
            .unwrap();
    }
}

#[derive(Clone, Debug)]
pub struct PositionSearchRow {
    pub run_id: i64,
    pub uci_position: String,
    pub depth: u32,
    pub time_taken: u32,
    pub nodes_evaluated: u32,
    pub evaluation: BoardEvaluation,
    pub conspiracy_counter: Option<ConspiracyCounter>,
    pub move_num: u32,
    pub timestamp: i64,
}

impl PositionSearchRow {
    pub async fn insert(&self, db: &SqlitePool, table_name: &str) -> SqliteQueryResult {
        let result = sqlx::query(&format!(r"
            INSERT INTO {} (
                run_id,
                uci_position,
                depth,
                time_taken,
                nodes_evaluated,
                evaluation,
                conspiracy_counter,
                move_num,
                timestamp
            ) VALUES (
                ?,
                ?,
                ?,
                ?,
                ?,
                ?,
                ?,
                ?,
                ?
            );
        ", table_name))
            .bind(self.run_id)
            .bind(self.uci_position.clone())
            .bind(self.depth)
            .bind(self.time_taken)
            .bind(self.nodes_evaluated)
            .bind(self.evaluation.to_string())
            // .bind(self.conspiracy_counter.clone().map(|x| x.to_string()))
            .bind({
                match self.conspiracy_counter.clone() {
                    None => None,
                    Some(x) => {
                        if x.zeroed_buckets() {
                            None
                        } else {
                            Some(x.to_string())
                        }
                    },
                }
            })
            .bind(self.move_num)
            .bind(self.timestamp)
            .execute(db)
            .await
            .unwrap();

        result
    }
}

pub struct MTSearchRow {
    pub position_search_id: i64,
    pub test_value: BoardEvaluation,
    pub time_taken: u32,
    pub nodes_evaluated: u32,
    pub eval_bound: EvalBound,
    pub conspiracy_counter: Option<ConspiracyCounter>,
    pub search_num: u32,
    pub timestamp: i64,
}

impl MTSearchRow {
    pub async fn insert(&self, db: &SqlitePool, table_name: &str) -> SqliteQueryResult {
        let result = sqlx::query(&format!(r"
            INSERT INTO {} (
                position_search_id,
                test_value,
                time_taken,
                nodes_evaluated,
                eval_bound,
                conspiracy_counter,
                search_num,
                timestamp
            ) VALUES (
                ?,
                ?,
                ?,
                ?,
                ?,
                ?,
                ?,
                ?
            );
        ", table_name))
            .bind(self.position_search_id)
            .bind(self.test_value.to_string())
            .bind(self.time_taken)
            .bind(self.nodes_evaluated)
            .bind(format!("{:?}", self.eval_bound))
            // .bind(self.conspiracy_counter.clone().map(|x| x.to_string()))
            .bind({
                match self.conspiracy_counter.clone() {
                    None => None,
                    Some(x) => {
                        if x.zeroed_buckets() {
                            None
                        } else {
                            Some(x.to_string())
                        }
                    },
                }
            })
            .bind(self.search_num)
            .bind(self.timestamp)
            .execute(db)
            .await
            .unwrap();

        result
    }

}


