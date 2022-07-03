use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub mod protocol_interpreter;
pub mod uci_interpreter;
pub mod stdin;

#[derive(Debug)]
pub struct ProtocolSupportError;


impl Display for ProtocolSupportError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Protocol not supported")
    }
}

impl Error for ProtocolSupportError {

}
