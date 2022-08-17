# crab-chess

Basic chess engine experiment with Rust

- Alpha-beta pruning
- Iterative deepening
- Infinite depth for capture moves
- Tiny transitory transposition table
- Piece-Square tables


### How to build

You'll need rust https://rustup.rs/

Open a terminal in this repository and
```
cargo build --release
```


### How to run

Supply up to two command-line arguments
```sh
target/debug/crab-chess black "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
```
- `black` - Human plays as this color against the AI
- `"rnbqkb...` - FEN string to start from a particular board position

Debug logging messages include FEN strings so that you can ~~savescum~~ return to previous positions


### Logging

Configure logging verbosity with the `CRAB_CHESS` environment variable

Replace `info` (default) with one of these increasingly verbose logging levels: `error`, `warn`, `info`, `debug`, `trace`
