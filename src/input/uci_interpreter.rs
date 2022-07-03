use std::fmt::format;
use std::str::{FromStr, SplitWhitespace};
use itertools::Itertools;
use chess::{Board, BoardBuilder, ChessMove, Square};

use crate::input::protocol_interpreter::{CalculateOptions, DebugState};
use super::protocol_interpreter::{Command, ProtocolInterpreter};

// Interpreter for the Universal Chess Interface protocol
pub struct UciInterpreter;

impl UciInterpreter {
    fn determine_board<'a>(mut args: impl Iterator<Item=&'a str>) -> Board {
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
            Some(other) => CalculateOptions::Infinite,  // TODO
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
            "position" => Some(Command::SetPosition(UciInterpreter::determine_board(split.into_iter()))),
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
    let expected = Some(Command::SetPosition(expected_board));

    let command_str = format!("{} {}", "position fen", fen);

    assert_eq!(UciInterpreter::line_to_command(&command_str), expected);
}

#[test]
fn check_position_start() {
    let expected = Some(Command::SetPosition(Board::default()));

    let command_str = "position startpos";
    assert_eq!(UciInterpreter::line_to_command(&command_str), expected);
}

#[test]
fn check_position_start_with_moves() {
    let expected = Some(Command::SetPosition(Board::from_str("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq - 0 1").unwrap()));
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