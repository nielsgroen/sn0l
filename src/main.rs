use std::io;
use std::io::{BufRead};
use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};
use anyhow;
use sn0l::core::search::SearchCommand;
use sn0l::input::protocol_interpreter::Command;
use sn0l::input::stdin::listen_to_stdin;


fn main() -> anyhow::Result<()> {
    start_uci_protocol()
}

fn start_uci_protocol() -> anyhow::Result<()> {
    // Make sure host (GUI) uses UCI protocol
    loop {
        let mut buffer = String::new();
        io::stdin().lock().read_line(&mut buffer).expect("Failed stdin read");

        println!("{buffer}");
        if buffer == "uci\n" || buffer == "uci" {
            break;
        } else if buffer == "quit\n" || buffer == "quit" {
            return Ok(());
        }

        // run again: GUI may have tried some other protocol, e.g. `xboard`
    }

    // stdin Channel
    let (input_tx, input_rx) = channel::<Command>();
    // Search Engine Channel
    let (search_tx, search_rx) = channel::<SearchCommand>();

    pre_option_init(input_tx, search_rx);
    println!("uciok"); // confirm pre-init

    loop {
        let command = input_rx.recv().unwrap();

        if let Some(search_command) = SearchCommand::from_command(command.clone()) {
            search_tx.send(search_command).unwrap();
        }

        match command {
            Command::IsReady => println!("readyok"),  // Main thread unblocked, so must be ready
            Command::Quit => break,  // TODO: should interrupt search, instead of letting it finish
            _ => (),  // currently unsupported command: Ignore, may have supported new protocols
        }
    }

    return Ok(());
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

