# Executables

## \[main executable\]
The main chess engine
- stored in `src/main.rs`
- can be run after compiling
- can also be run using `cargo run --release`

## store_analysis
For storing all kinds of search metrics to an sqlite DB.
- stored in `src/bin/store_analysis.rs`
- can be run after compiling
- can also be run using `cargo run --bin store_analysis --release`
- for viewing arg options: `cargo run --bin store_analysis --release -- --help`
- for running with options: `cargo run --bin store_analysis --release -- [OPTIONS]`
