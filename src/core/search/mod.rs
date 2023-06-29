use std::sync::mpsc::Receiver;
use chess::{Board, ChessMove};
use crate::core::search::conspiracy_search::merging::merge_remove_overwritten;
use crate::input::protocol_interpreter::{CalculateOptions, Command};

use crate::core::search::iterative_deepening::iterative_deepening_search;
use crate::core::search::mtd::mtd_iterative_deepening_search;
use crate::core::search::mtdbi::{determine_mtdbi_step, mtdbi_iterative_deepening_search};
use crate::core::search::mtdf::mtdf_iterative_deepening_search;
use crate::core::search::search_result::debug_search_result::DebugSearchResult;
use crate::core::search::search_result::SearchResult;
use crate::core::search::transpositions::{EvalBound, TranspositionTable};
use crate::core::search::transpositions::high_depth_transposition::HighDepthTranspositionTable;

pub mod search_result;
pub mod transpositions;
mod draw_detection;
pub mod iterative_deepening;
mod move_ordering;
pub mod alpha_beta;
pub mod mtdf;
pub mod mt;
pub mod common;
pub mod mtdbi;
pub mod mtd;
pub mod conspiracy_search;
pub mod conspiracy_counter;


/// The information about what search has been done on a particular node.
#[derive(Clone, Debug)]
pub struct SearchInfo {
    pub depth_searched: SearchDepth,
    pub evaluation: EvalBound,
    pub best_move: ChessMove,
    pub prime_variation: Option<Vec<ChessMove>>,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum SearchDepth {
    // Ordering matters for derive Ord
    Single, // Did a simple single board eval
    QuiescentDepth(u32), // Performed Quiescence search at depth `x` from here.
    Quiescent, // Performed exhaustive Quiescence Search from this node
    Depth(u32), // Depth still left to go
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum SearchCommand {
    NewGame,
    SetPosition(Board, Vec<u64>),
    Calculate(CalculateOptions),
    Stop,
}

impl SearchCommand {
    pub fn from_command(command: Command) -> Option<Self> {
        match command {
            Command::NewGame => Some(SearchCommand::NewGame),
            Command::SetPosition(board, moves) => Some(SearchCommand::SetPosition(board, moves)),
            Command::Calculate(options) => Some(SearchCommand::Calculate(options)),
            Command::Stop => Some(SearchCommand::Stop),
            _ => None,
        }
    }
}


/// The function to have a thread start functioning as the search engine.
pub fn start_search_engine(search_rx: Receiver<SearchCommand>) {
    // init Transposition Table
    // let mut transposition_table = HighDepthTranspositionTable::new(SearchDepth::Depth(2));
    let mut transposition_table: Box<dyn TranspositionTable> = Box::new(HighDepthTranspositionTable::new(SearchDepth::Depth(2)));
    let mut main_board: Board = Board::default();
    let mut visited_boards: Vec<u64> = Vec::new(); // List of board hashes

    loop {
        let command = search_rx.recv().expect("search receiver error");
        println!("{:?}", command);

        match command {
            SearchCommand::SetPosition(board, visited) => {
                main_board = board;
                visited_boards = visited;
            },
            // SearchCommand::NewGame => transposition_table = HighDepthTranspositionTable::new(SearchDepth::Depth(2)),
            SearchCommand::NewGame => transposition_table = Box::new(HighDepthTranspositionTable::new(SearchDepth::Depth(2))),
            SearchCommand::Calculate(options) => {

                let options = CalculateOptions::Depth(6);
                // let (search_result, depth, selective_depth): (DebugSearchResult, _, _) = iterative_deepening_search(
                //     &main_board,
                //     &mut transposition_table,
                //     visited_boards.clone(),
                //     options,
                // );
                // let (search_result, depth, selective_depth): (DebugSearchResult, _, _) = mtdbi_iterative_deepening_search(
                //     &main_board,
                //     &mut transposition_table,
                //     visited_boards.clone(),
                //     options,
                // );
                let (search_result, _conspiracy_counter, _depth, _selective_depth): (DebugSearchResult, _, _, _) = conspiracy_search::mtd_w_conspiracy::mtd_iterative_deepening_search(
                    &main_board,
                    &mut transposition_table,
                    visited_boards.clone(),
                    options,
                    determine_mtdbi_step,
                    20,
                    101,
                    merge_remove_overwritten,
                    |_, _| {},
                );

                println!("bestmove {}", search_result.best_move());
            },
            SearchCommand::Stop => (),
        }
    }
}
