use chess::Board;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SupportedProtocols {
    UCI,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CalculateOptions {
    Infinite,  // Keep on calculating in perpetuity
    MoveTime(u64),  // Time to calculate, in ms
    // Game {
    //     white_time: u64,
    //     black_time: u64,
    //     white_increment: u64,
    //     black_increment: u64,
    //     // moves_to_go: u64  // moves to next time control
    // }
}

impl Default for CalculateOptions {
    fn default() -> Self {
        CalculateOptions::Infinite
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DebugState {
    On,
    Off,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Command {
    DetermineProtocol,
    Identify,
    ToggleDebug(DebugState),
    InitFinished,  // queries whether the engine is finished initializing
    // SetOption {OptionValue},
    NewGame,
    SetPosition(Board),  // sets the board position for that game
    Calculate(CalculateOptions),  // `go` in UCI: Start calculating
    Stop,  // Stop Calculating, otherwise ignore
    // Ponder,  see UCI doc
    // PonderHit,
    Quit,  // exit the program
}


pub trait ProtocolInterpreter {
    fn line_to_command(line: &str) -> Option<Command>;
}
