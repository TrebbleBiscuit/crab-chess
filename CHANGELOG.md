
### 0.0.12

- Apply a bonus to passed pawns close to promotion
- Search extension for top-level capture moves
- Fixed a bug with move selection
- Reduced maximum check move depth

### 0.0.11

Modified pawn piece square table - +200 to row 7, +100 to row 6, +20 to row 5

### 0.0.10

- Added king safety to board evaluation, scaled by enemy pieces remaining
- Added a maximum search depth to truncate evaluation of extremely long chains of checking moves

### 0.0.9

- Transposition table bugfixes and improvements

### 0.0.8

- Added depth reduction for later-ordered moves in top level searches
- Fixed stalemate recognition

### 0.0.7

- Fixed a huge bug with cancelled searches causing blunders

### 0.0.6

- When iterating moves, move generation will first consider the opponent's best response to the current best move, significantly improving the number of pruned branches
- Transposition table lookup disabled for top level search to enable better move ordering for iterative deepening

### 0.0.5

- Fixed and improved search and evaluation, checkmate evaluation  
- Searches from transposition table evaluate the best move first

### 0.0.4

- Transposition table stores moves up to depth 3, much better time control awareness

### 0.0.3

- Searches for less than 4 seconds up to depth 9, can stop mid search

### 0.0.2

- Searches for at least five seconds at a high depth

### 0.0.1

- Searches at depth 5
