use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::mpsc::Receiver;
use chess::Board;
use crate::Command;
use crate::input::protocol_interpreter::CalculateOptions;

use transposition::TranspositionTable;

pub mod transposition;
mod iterative_deepening;
mod move_ordering;


/// The information about what search has been done on a particular node.
pub struct SearchInfo {
    depth_searched: SearchDepth,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum SearchDepth {
    // Ordering matters for derive Ord
    Single, // Did a simple single board eval
    Quiescent, // Performed Quiescence Search from this node
    Depth(NonZeroU32), // Depth still left to go
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SearchCommand {
    NewGame,
    SetPosition(Board),
    Calculate(CalculateOptions),
    Stop,
}

impl SearchCommand {
    pub fn from_command(command: Command) -> Option<Self> {
        match command {
            Command::NewGame => Some(SearchCommand::NewGame),
            Command::SetPosition(board) => Some(SearchCommand::SetPosition(board)),
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
    let mut transposition_table: TranspositionTable = TranspositionTable::default();
    let mut main_board: Board = Board::default();

    loop {
        let command = search_rx.recv().expect("search receiver error");

        match command {
            SearchCommand::SetPosition(board) => main_board = board,
            SearchCommand::NewGame => transposition_table = TranspositionTable::default(),
            SearchCommand::Calculate(options) => {

            },
            SearchCommand::Stop => (),
        }


    }
}