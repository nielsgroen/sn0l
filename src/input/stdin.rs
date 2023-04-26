use std::io;
use std::io::BufRead;
use std::sync::mpsc::Sender;

use super::protocol_interpreter::Command;
use super::uci_interpreter::UciInterpreter;
use super::protocol_interpreter::ProtocolInterpreter;

pub fn listen_to_stdin(input_tx: Sender<Command>) -> ! {
    loop {
        let mut buffer = String::new();
        io::stdin().lock().read_line(&mut buffer).expect("Failed stdin read");

        let command = UciInterpreter::line_to_command(&buffer);

        if let Some(val) = command.clone() {
            input_tx.send(val).unwrap();
        }

        println!("{:?}", command);
    }
}