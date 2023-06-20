use std::path::PathBuf;
use std::str::FromStr;
use anyhow::Result;
use chess::ChessMove;

pub const A_OPENING_PATH: &str = "./src/analysis/assets/a.tsv";
pub const B_OPENING_PATH: &str = "./src/analysis/assets/b.tsv";
pub const C_OPENING_PATH: &str = "./src/analysis/assets/c.tsv";
pub const D_OPENING_PATH: &str = "./src/analysis/assets/d.tsv";
pub const E_OPENING_PATH: &str = "./src/analysis/assets/e.tsv";

// eco_code, name, pgn, uci, fen
type ChessOpeningRecord = (String, String, String, String, String);

pub struct ChessOpening {
    pub eco_code: String,
    pub name: String,
    pub moves: Vec<ChessMove>,
}

pub fn get_opening_records() -> Vec<ChessOpening> {
    let record_paths  = [
        A_OPENING_PATH,
        B_OPENING_PATH,
        C_OPENING_PATH,
        D_OPENING_PATH,
        E_OPENING_PATH,
    ].into_iter().map(|x| PathBuf::from(x));

    let mut readers = record_paths.map(|x| {
        csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_path(x)
            .expect("Failed to get asset path of opening file.")
    }).collect::<Vec<_>>();

    let openings = readers.into_iter().map(|mut x| {
        let result: Vec<ChessOpening> = x.deserialize().map(
            |record| {
                let record: ChessOpeningRecord = record.expect("Misformatted opening record");

                let string_moves = record.3.split(" ");
                let chess_moves = string_moves.map(|x| ChessMove::from_str(x).expect("Misformatted chess move"));

                ChessOpening {
                    eco_code: record.0.clone(),
                    name: record.1.clone(),
                    moves: chess_moves.collect::<Vec<_>>(),
                }
            }
        ).collect();

        result
    }).flatten().collect::<Vec<_>>();

    openings
}
