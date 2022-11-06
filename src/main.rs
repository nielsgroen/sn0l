use std::collections::HashMap;
use std::io;
use std::io::{BufRead, stdout};
use std::str::FromStr;
use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::sleep;
use std::time::Duration;
use anyhow;
use chess::{Board, MoveGen};
use clap::Parser;

use input::uci_interpreter::UciInterpreter;
use input::protocol_interpreter::ProtocolInterpreter;
use input::stdin::listen_to_stdin;
use input::protocol_interpreter::Command;
use input::ProtocolSupportError;
use crate::core::search::iterative_deepening::iterative_deepening_search;
use crate::core::search::search_result::minimal_search_result::MinimalSearchResult;

use crate::core::search::SearchCommand;
use crate::core::search::transpositions::no_transposition::NoTranspositionTable;
use crate::input::command_line::Cli;
use crate::input::protocol_interpreter::CalculateOptions;

mod core;
mod input;

// maybe this needs to be split up, because rust requires a lock around the whole thing?
// struct GlobalState {
//     debug_enabled: bool,
//
// }

fn main() -> anyhow::Result<()> {
    let cli = input::command_line::Cli::parse();

    match cli.benchmark {
        true => run_benchmark(),
        false => start_uci_protocol(),
    }


    // loop {
    //     let command = input_rx.recv().unwrap();
    //
    //     if let Some(search_command) = SearchCommand::from_command(command.clone()) {
    //         search_tx.send(search_command).unwrap();
    //     }
    //
    //     match command {
    //         Command::IsReady => println!("readyok"),  // Main thread unblocked, so must be ready
    //         Command::Quit => break,
    //         // Command::SetPosition(board) => current_board = board,
    //         // Command::Calculate(_) => {
    //         //     // let mut candidate_moves = MoveGen::new_legal(&current_board);
    //         //     // let chosen_move = candidate_moves.next().unwrap();
    //         //     let chosen_move = core::evaluation_old::best_move_depth(&current_board, 3).unwrap();
    //         //     println!("bestmove {}", chosen_move)
    //         // }
    //         _ => (),  // TODO
    //     }
    // }
}

fn start_uci_protocol() -> anyhow::Result<()> {
    // Make sure host (GUI) uses UCI protocol
    loop {
        let mut buffer = String::new();
        io::stdin().lock().read_line(&mut buffer).expect("Failed stdin read");

        if buffer == "uci\n" {
            break;
        } else if buffer == "quit\n" {
            return Ok(());
        }

        // run again: GUI may have tried some other protocol, e.g. `xboard`
    }

    // stdin Channel
    let (input_tx, input_rx) = channel::<Command>();
    // Search Engine Channel
    let (search_tx, search_rx) = channel::<SearchCommand>();

    pre_option_init(input_tx, search_rx);
    // let mut current_board = Board::default();  // TODO move this to the core engine
    println!("uciok"); // confirm pre-init

    // TODO remove temp
    search_tx.send(SearchCommand::from_command(
        UciInterpreter::line_to_command("position startpos").unwrap()).unwrap()
    )?;
    search_tx.send(SearchCommand::from_command(
        UciInterpreter::line_to_command("go 8").unwrap()).unwrap()
    )?;
    sleep(Duration::new(60, 0));
    return Ok(());
}

fn run_benchmark() -> anyhow::Result<()> {
    println!("Started benchmark");

    let (result, depth, selective_depth): (MinimalSearchResult, _, _) =
        iterative_deepening_search(
            &Board::default(),
            &mut NoTranspositionTable::default(),
            Vec::new(),
            CalculateOptions::Depth(5),
        );

    println!("Done");
    Ok(())
}


fn pre_option_init(input_tx: Sender<Command>, search_rx: Receiver<SearchCommand>) {
    // The thread that listens to stdin
    thread::spawn(move || {
        listen_to_stdin(input_tx);
    });

    // The thread that runs the search engine
    thread::spawn(move || {
        core::search::start_search_engine(search_rx);
    });

    println!("id name sn0l 0.1");
    println!("id author Niels Groeneveld");

    // list options here, once there are some
}