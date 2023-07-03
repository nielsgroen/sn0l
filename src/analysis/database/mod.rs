use sqlx::migrate::MigrateDatabase;
use sqlx::{Pool, Sqlite, SqlitePool};

pub mod rows;

pub const DB_URL: &str = "sqlite://sqlite.db";
pub const CONFIG_TABLE: &str = "config";
pub const RUN_TABLE: &str = "run";
pub const POSITION_SEARCH_TABLE: &str = "position_search";
pub const MT_SEARCH_TABLE: &str = "mt_search";

pub async fn create_db_if_not_exists(url: &str) -> SqlitePool {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating Database {}", DB_URL);

        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Succesfully created DB"),
            Err(error) => panic!("Failed to create DB {}", error),
        }
    } else {
        println!("Existing Database found");
    }

    SqlitePool::connect(DB_URL).await.unwrap()
}

pub async fn create_tables_if_not_exists(db: &SqlitePool) {
    // id, max_search_depth, algorithm_used, conspiracy_search_used, bucket_size, num_buckets, conspiracy_merge_fn, transposition_table_used, minimum_transposition_depth, timestamp
    let result = sqlx::query(r"
        CREATE TABLE IF NOT EXISTS config (
            id INTEGER PRIMARY KEY NOT NULL,
            max_search_depth INTEGER,
            algorithm_used TEXT NOT NULL,
            conspiracy_search_used INTEGER NOT NULL,
            bucket_size INTEGER,
            num_buckets INTEGER,
            conspiracy_merge_fn TEXT,
            transposition_table_used INTEGER NOT NULL,
            minimum_transposition_depth INTEGER,
            timestamp INTEGER
        );
    ").execute(db).await.unwrap();

    println!("Created config table result: {:?}", result);

    // id, foreign key Run config, uci_position (e.g. `startpos moves b1c3`), opening_name (optional), timestamp,
    let result = sqlx::query(r"
        CREATE TABLE IF NOT EXISTS run (
            id INTEGER PRIMARY KEY NOT NULL,
            config_id INTEGER NOT NULL,
            uci_position TEXT NOT NULL,
            opening_name TEXT,
            match_result TEXT,
            timestamp INTEGER,
            FOREIGN KEY(config_id) REFERENCES config(id)
        );
    ").execute(db).await.unwrap();

    println!("Created runs table result: {:?}", result);

    // id, foreign key Run starts, uci position, depth, time_taken, nodes_evaluated, evaluation, conspiracy_counter (optional), timestamp
    let result = sqlx::query(r"
        CREATE TABLE IF NOT EXISTS position_search (
            id INTEGER PRIMARY KEY NOT NULL,
            run_id INTEGER NOT NULL,
            uci_position TEXT NOT NULL,
            depth INTEGER NOT NULL,
            time_taken INTEGER NOT NULL,
            nodes_evaluated INTEGER NOT NULL,
            evaluation TEXT NOT NULL,
            conspiracy_counter TEXT,
            move_num INTEGER,
            timestamp INTEGER,
            FOREIGN KEY(run_id) REFERENCES run(id)
        );
    ").execute(db).await.unwrap();

    println!("Created search_position table result: {:?}", result);

    // id, foreign key Position search, test_value, time_taken, nodes_evaluated, eval_boundary_type, evaluation, conspiracy_counter (optional), timestamp
    let result = sqlx::query(r"
        CREATE TABLE IF NOT EXISTS mt_search (
            id INTEGER PRIMARY KEY NOT NULL,
            position_search_id INTEGER NOT NULL,
            test_value TEXT NOT NULL,
            time_taken INTEGER NOT NULL,
            nodes_evaluated INTEGER NOT NULL,
            eval_bound TEXT NOT NULL,
            conspiracy_counter TEXT,
            search_num INTEGER,
            timestamp INTEGER,
            FOREIGN KEY(position_search_id) REFERENCES position_search(id)
        );
    ").execute(db).await.unwrap();

    println!("Created mt_search table result: {:?}", result);
}




