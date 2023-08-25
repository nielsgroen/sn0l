# sn0l
A Chess Engine for investigating the MTD-H algorithm.

## The Executables
- The main engine is at `main.rs`, can be run by using `cargo run --release`
- Another executable for writing lots of engine diagnostics to a DB can be found at `bin/store_analysis`.
This can be run using `cargo run --bin store_analysis --release`.
It stores all sorts of data gathered from letting the engine play the [Win at Chess](https://www.chessprogramming.org/Win_at_Chess) dataset, or the [Lichess Opening Database](https://github.com/lichess-org/chess-openings) against itself.


To get all the options for collecting engine metrics, you can run:
```shell
cargo run --bin store_analysis --release -- -h
```
For more info on running the binaries, there is another README at the /src/bin/ directory.
