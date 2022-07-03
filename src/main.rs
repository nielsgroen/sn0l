use std::io;
use std::io::BufRead;

use input::uci_interpreter::UciInterpreter;
use input::protocol_interpreter::ProtocolInterpreter;


mod input;


fn main() {

    for _ in 0..20 {
        let mut buffer = String::new();
        io::stdin().lock().read_line(&mut buffer);
        println!("{:?}", UciInterpreter::line_to_command(&buffer));
    }
}
