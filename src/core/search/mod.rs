use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};
use chess::{Board, ChessMove, Color};
use crate::Command;
use crate::input::protocol_interpreter::CalculateOptions;

use transpositions::TranspositionTable;
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::iterative_deepening::iterative_deepening_search;
use crate::core::search::search_result::debug_search_result::DebugSearchResult;
use crate::core::search::search_result::minimal_search_result::MinimalSearchResult;
use crate::core::search::search_result::SearchResult;
use crate::core::search::transpositions::hash_transposition::HashTranspositionTable;
use crate::core::search::transpositions::no_transposition::NoTranspositionTable;

pub mod search_result;
pub mod transpositions;
mod draw_detection;
pub mod iterative_deepening;
mod move_ordering;
mod alpha_beta;


/// The information about what search has been done on a particular node.
pub struct SearchInfo {
    pub depth_searched: SearchDepth,
    pub evaluation: BoardEvaluation,
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

// Should this be impl?
// impl TryFrom<Command> for SearchCommand {
//     type Error = ();
//
//     fn try_from(value: Command) -> Result<Self, Self::Error> {
//         match value {
//             Command::NewGame => Ok(SearchCommand::NewGame),
//             Command::SetPosition(board) => Ok(SearchCommand::SetPosition(board)),
//             Command::Calculate(options) => Ok(SearchCommand::Calculate(options)),
//             Command::Stop => Ok(SearchCommand::Stop),
//             _ => Err(()),
//         }
//     }
// }



/// The function to have a thread start functioning as the search engine.
pub fn start_search_engine(search_rx: Receiver<SearchCommand>) {

    // init Transposition Table
    let mut transposition_table = NoTranspositionTable::default();
    let mut main_board: Board = Board::default();
    let mut visited_boards: Vec<u64> = Vec::new(); // List of board hashes

    loop {
        let command = search_rx.recv().expect("search receiver error");

        match command {
            SearchCommand::SetPosition(board, visited) => {
                main_board = board;
                visited_boards = visited;
            },
            SearchCommand::NewGame => transposition_table = NoTranspositionTable::default(),
            SearchCommand::Calculate(options) => {

                let (search_result, depth, selective_depth): (DebugSearchResult, _, _) = iterative_deepening_search(
                    &main_board,
                    &mut transposition_table,
                    visited_boards.clone(),
                    options,
                );

                println!("bestmove {}", search_result.best_move());

            },
            SearchCommand::Stop => (),
        }

    }
}
