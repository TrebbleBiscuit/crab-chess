# crab-chess

Chess engine made for fun! See some games it's played online here - https://lichess.org/@/CrabChess

- [Iterative deepening](https://www.chessprogramming.org/Iterative_Deepening) with [PV-Move ordering](https://www.chessprogramming.org/PV-Move) to find better moves faster
- [Negamax](https://en.wikipedia.org/wiki/Negamax) with [alpha-beta pruning](https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning) to search only what we need to
- [Quiescence search](https://www.chessprogramming.org/Quiescence_Search) to find stable board positions
- Other fun and exciting techniques


### How to build

You'll need rust https://rustup.rs/

Open a terminal in this repository and
```
cargo build --release
```


### How to run

This bot crudely implements UCI. I use it with [lichess-bot](https://github.com/lichess-bot-devs/lichess-bot) to play online and [cutechess](https://github.com/cutechess/cutechess) to test.

Here's an example of commands you might run (prefixed by "`> `")

```
> uci
uciok
> ucinewgame
> position startpos moves e2e4
> go movetime 2500
info depth 2 seldepth 0 score cp 12 time 13 pv g8f6 b1c3
info depth 3 seldepth 8 score cp 48 time 40 pv g8f6 b1c3
info depth 4 seldepth 12 score cp 12 time 215 pv g8f6 b1c3
info depth 5 seldepth 21 score cp 42 time 2500 pv g8f6 e4e5
bestmove g8f6
```

These commands tell the bot to
- Enable uci mode
- Create a new game
- Specify a position in which, from the starting position, the move `e2e4` has been made
- Tell the bot to think for 2.5 seconds 


### Logging

Log files are written to `crab_logs/` unless you use `--quiet` or `-q`

You can make these logs more verbose with `--verbose` or `-v`
