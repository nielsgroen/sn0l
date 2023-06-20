use std::io;
use std::io::{BufRead};
use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};
use anyhow;
use chess::{Board};
use clap::Parser;
use sn0l::core::search::iterative_deepening::iterative_deepening_search;
use sn0l::core::search::search_result::minimal_search_result::MinimalSearchResult;
use sn0l::core::search::SearchCommand;
use sn0l::core::search::transpositions::no_transposition::NoTranspositionTable;
use sn0l::input;
use sn0l::input::protocol_interpreter::{CalculateOptions, Command};
use sn0l::input::stdin::listen_to_stdin;

// use input::stdin::listen_to_stdin;
// use input::protocol_interpreter::Command;
// use sn0l::core::search::iterative_deepening::iterative_deepening_search;
// use sn0l::core::search::search_result::minimal_search_result::MinimalSearchResult;
//
// use sn0l::core::search::SearchCommand;
// use sn0l::core::search::transpositions::no_transposition::NoTranspositionTable;
// use sn0l::input::protocol_interpreter::CalculateOptions;

// mod core;
// mod input;
// mod tests;

// maybe this needs to be split up, because rust requires a lock around the whole thing?
// struct GlobalState {
//     debug_enabled: bool,
//
// }

fn main() -> anyhow::Result<()> {
    let cli = input::command_line::Cli::parse();

    println!("{:?}", cli);

    match cli.benchmark {
        true => run_benchmark(),
        false => start_uci_protocol(),
    }


}

fn start_uci_protocol() -> anyhow::Result<()> {
    // Make sure host (GUI) uses UCI protocol
    println!("started!");
    loop {
        let mut buffer = String::new();
        io::stdin().lock().read_line(&mut buffer).expect("Failed stdin read");

        println!("{buffer}");
        if buffer == "uci\n" || buffer == "uci" {
            break;
        } else if buffer == "quit\n" || buffer == "quit" {
            return Ok(());
        }

        break; // TODO: remove
        // run again: GUI may have tried some other protocol, e.g. `xboard`
    }

    // stdin Channel
    let (input_tx, input_rx) = channel::<Command>();
    // Search Engine Channel
    let (search_tx, search_rx) = channel::<SearchCommand>();

    pre_option_init(input_tx, search_rx);
    // let mut current_board = Board::default();  // TODO move this to the core engine
    println!("uciok"); // confirm pre-init

    loop {
        let command = input_rx.recv().unwrap();

        if let Some(search_command) = SearchCommand::from_command(command.clone()) {
            search_tx.send(search_command).unwrap();
        }

        match command {
            Command::IsReady => println!("readyok"),  // Main thread unblocked, so must be ready
            Command::Quit => break,
            // Command::SetPosition(board) => current_board = board,
            // Command::Calculate(_) => {
            //     // let mut candidate_moves = MoveGen::new_legal(&current_board);
            //     // let chosen_move = candidate_moves.next().unwrap();
            //     let chosen_move = core::evaluation_old::best_move_depth(&current_board, 5).unwrap();
            //     println!("bestmove {}", chosen_move)
            // }
            _ => (),  // TODO
        }
    }

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
        sn0l::core::search::start_search_engine(search_rx);
    });

    println!("id name sn0l 0.1");
    println!("id author Niels Groeneveld");

    // list options here, once there are some
}

