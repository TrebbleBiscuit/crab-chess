# crab-chess

Basic chess engine experiment with Rust

- Cheap pre-analysis move ordering
- Alpha-beta pruning
- Configurable iterative deepening
- Infinite depth for capture moves
- Piece-Square tables

### How to run

``` sh
cargo run
```

Or if you want the nice pretty logging messages:

```
cargo build
RUST_LOG=info target/debug/crab-chess
```

Replace `info` with one of these increasingly verbose logging levels: `error`, `warn`, `info`, `debug`, `trace`, 

### Specific positions

This program takes a fen as an argument, so you can do

``` sh
cargo run "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
```

or

``` sh
target/debug/crab-chess "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
```
