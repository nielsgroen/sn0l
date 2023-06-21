# sn0l
Chess Engine


## The Executables
- The main engine is at `main.rs`, can be run by using `cargo run --release`
- Another executable for writing lots of engine diagnostics to a DB can be found at `bin/store_analysis`.
This can be run using `cargo run --bin store_analysis --release`.
It stores all sorts of data gathered from letting the engine play the [Lichess Opening Database](https://github.com/lichess-org/chess-openings) against itself.
