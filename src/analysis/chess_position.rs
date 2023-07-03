pub trait ChessPosition {
    /// Returns the opening name, and the UCI string.
    fn uci_position(&self) -> (Option<String>, String);
}
