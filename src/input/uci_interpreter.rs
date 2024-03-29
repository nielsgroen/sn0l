use std::str::{FromStr};
use chess::{Board, ChessMove, Square};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;

use crate::input::protocol_interpreter::{CalculateOptions, DebugState};
use crate::input::protocol_interpreter::CalculateOptions::Game;
use super::protocol_interpreter::{Command, ProtocolInterpreter};

// Interpreter for the Universal Chess Interface protocol
pub struct UciInterpreter;

impl UciInterpreter {
    pub fn determine_board<'a>(mut args: impl Iterator<Item=&'a str>) -> Board {
        match args.next() {
            Some("startpos") => {
                let mut board = Board::default();

                if args.next() == Some("moves") {
                    for arg in args {
                        board = board.make_move_new(ChessMove::from_str(&arg).expect("non-chess move specified"));
                    }
                }

                board
            },
            Some("fen") => {
                let mut args_owned: Vec<String> = Vec::new();
                for arg in args {
                    args_owned.push(arg.to_string());
                }
                let mut board = Board::from_str(&args_owned.join(" ")).expect("invalid FEN code");

                let args = args_owned.join(" ");
                if let Some(moves) = args.split("moves ").skip(1).next() {
                    for chess_move in moves.split(" ").into_iter() {
                        board = board.make_move_new(ChessMove::from_str(chess_move).expect("non-chess move specified"));
                    }
                }

                board
            },
            x => panic!("unsupported position parameters {:?}", x),
        }
    }

    pub fn determine_pre_move_board<'a>(mut args: impl Iterator<Item=&'a str>) -> Board {
        match args.next() {
            Some("startpos") => {
                Board::default()
            },
            Some("fen") => {
                let mut args_owned: Vec<String> = Vec::new();
                for arg in args {
                    args_owned.push(arg.to_string());
                }
                Board::from_str(&args_owned.join(" ")).expect("invalid FEN code")
            },
            _ => panic!("unsupported position parameters")
        }
    }

    fn determine_calculate_options<'a>(mut args: impl Iterator<Item=&'a str>) -> CalculateOptions {
        match args.next() {
            Some("infinite") => CalculateOptions::Infinite,
            Some("movetime") => CalculateOptions::MoveTime(
                args
                    .next()
                    .expect("no movetime value specified")
                    .parse::<u64>()
                    .expect("movetime must be a positive integer")
            ),
            Some("wtime") => {
                lazy_static! {
                    static ref TIME_REGEX: Regex = Regex::new(r"(?P<wtime>\d+) btime (?P<btime>\d+) winc (?P<winc>\d+) binc (?P<binc>\d+)").unwrap();
                }

                let args_joined = args.join(" ");
                let capture = TIME_REGEX.captures(&args_joined).expect("invalid calculation times set");

                let wtime = capture.name("wtime").unwrap();
                let btime = capture.name("btime").unwrap();
                let winc = capture.name("winc").unwrap();
                let binc = capture.name("binc").unwrap();

                let wtime = wtime.as_str().parse::<u64>().expect("wtime must be a positive integer");
                let btime = btime.as_str().parse::<u64>().expect("btime must be a positive integer");
                let winc = winc.as_str().parse::<u64>().expect("winc must be a positive integer");
                let binc = binc.as_str().parse::<u64>().expect("binc must be a positive integer");

                Game {
                    white_time: wtime,
                    black_time: btime,
                    white_increment: winc,
                    black_increment: binc,
                }
            }
            Some("depth") => {
                CalculateOptions::Depth(
                    args
                        .next()
                        .expect("no depth value specified")
                        .parse::<u64>()
                        .expect("depth must be a positive integer") as u32
                )
            },
            Some(_other) => CalculateOptions::Infinite,  // TODO
            // Some(_) => panic!("unsupported calculate option"),
            None => CalculateOptions::Infinite,
        }
    }

    fn determine_debug_state<'a>(mut args: impl Iterator<Item=&'a str>) -> DebugState {
        match args.next() {
            Some("on") => DebugState::On,
            Some("off") => DebugState::Off,
            _ => panic!("unknown debug state"),
        }
    }

    /// Returns a vector of hashes of visited board positions
    pub fn determine_visited_boards<'a>(board: &Board, mut args: impl Iterator<Item=&'a str>) -> Vec<u64> {
        let moves = {
            let mut result = Vec::new();
            let mut listing_moves = false;
            for arg in args {
                if listing_moves {
                    result.push(ChessMove::from_str(arg).expect("expected a chess move"));
                } else {
                    if arg == "moves" {
                        listing_moves = true;
                    }
                }
            }
            result
        };

        let mut result = Vec::from([board.get_hash()]);

        let mut current_board = *board;
        for chess_move in moves {
            current_board = current_board.make_move_new(chess_move);
            result.push(current_board.get_hash());
        }
        // Remove last board, since that board is the actual one in play,
        result.pop();

        result
    }
}

impl ProtocolInterpreter for UciInterpreter {
    fn line_to_command(line: &str) -> Option<Command> {
        let mut split = line.split_whitespace();

        let command_word = split.next()?;

        match command_word {
            "uci" => Some(Command::Identify),
            // "debug" => Some(Command::ToggleDebug),
            "debug" => Some(Command::ToggleDebug(UciInterpreter::determine_debug_state(split.into_iter()))),
            "isready" => Some(Command::IsReady),
            // "setoption" => ,
            "ucinewgame" => Some(Command::NewGame),
            "position" => {
                let board = UciInterpreter::determine_board(split.clone().into_iter());
                let pre_move_board = UciInterpreter::determine_pre_move_board(split.clone().into_iter());
                Some(Command::SetPosition(
                    board.clone(),
                    UciInterpreter::determine_visited_boards(&pre_move_board, split.into_iter())
                ))
            },
            "go" => Some(Command::Calculate(UciInterpreter::determine_calculate_options(split.into_iter()))),
            "stop" => Some(Command::Stop),
            "quit" => Some(Command::Quit),
            _ => None,
        }
    }
}

#[test]
fn check_debug_toggle() {
    let expected = Some(Command::ToggleDebug(DebugState::On));

    assert_eq!(UciInterpreter::line_to_command("debug on"), expected);
}

#[test]
fn check_position_fen_command() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let expected_board = Board::from_str(fen).unwrap();
    let expected = Some(Command::SetPosition(expected_board, vec![]));

    let command_str = format!("{} {}", "position fen", fen);

    assert_eq!(UciInterpreter::line_to_command(&command_str), expected);
}

#[test]
fn check_position_start() {
    let expected = Some(Command::SetPosition(Board::default(), vec![]));

    let command_str = "position startpos";
    assert_eq!(UciInterpreter::line_to_command(&command_str), expected);
}

#[test]
fn check_position_start_with_moves() {
    let board = Board::default();
    let visited = vec![board.get_hash()];
    let board = board.make_move_new(ChessMove::new(Square::D2, Square::D4, None));
    let expected = Some(Command::SetPosition(board, visited));
    let command_str = "position startpos moves d2d4";

    assert_eq!(UciInterpreter::line_to_command(command_str), expected);
}

#[test]
fn check_calculate_infinite() {
    let expected = Some(Command::Calculate(CalculateOptions::Infinite));

    let command_str = "go infinite";
    assert_eq!(UciInterpreter::line_to_command(&command_str), expected);

    let command_str = "go infinite asd";
    assert_eq!(UciInterpreter::line_to_command(&command_str), expected);
}

#[test]
fn check_calculate_movetime() {
    let expected = Some(Command::Calculate(CalculateOptions::MoveTime(2000)));

    let command_str = "go movetime 2000";
    assert_eq!(UciInterpreter::line_to_command(&command_str), expected);
}

#[test]
#[should_panic]
fn assert_movetime_panic() {
    UciInterpreter::line_to_command("go movetime -1");
}