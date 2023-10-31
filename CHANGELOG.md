
### 0.0.14-rc9

- More array lookups instead of calculations

### 0.0.14-rc8

- seen_positions is now a slice

### 0.0.14-rc7

- Replaced many calculations with array lookups

### 0.0.14-rc6

- Fixed a bug where isolated and stacked pawns were not properly being considered during evaluation

### 0.0.14-rc5

- Properly implement more efficient repetition detection
- Some other optimizations

### 0.0.14-rc4

- Pass slices instead of move values around
- Testing more efficient repetition detection

### 0.0.14-rc3

- Changed endgame board evaluation in several ways:
    - Tripled the effect of king positioning
    - Increased the bonus for pawns near promotion (200, 120, 70, 30)
    - Decreased effect of king piece-square table

### 0.0.14-rc2

Evaluate a board as stronger if pawns have support and are not doubled up

### 0.0.14-rc1

- The player who is winning in the endgame should try and hug the enemy king

### 0.0.13-rc9

- In the endgame, it's good to have your king in the center

### 0.0.13-rc8

- Add endgame PST for pawns, start using it as the number of major pieces on the board decreases

### 0.0.13-rc7

- Tuned maximum sub-search depth (should not affect most positions)
- Small refactor to use array instead of vector lookups for piece-square tables

### 0.0.13-rc6

- Implement position repetition detection in deep searches
- Limit maximum search depth by a factor of minimum search depth to keep from looking extremely deep into lines from low depth searches

### 0.0.13-rc5

- Removed all search depth reductions, but kept extensions

### 0.0.13-rc4

- Added further reduction in search depth of later ordered moves at higher depths

### 0.0.13-rc3 

- Added `--quiet` argument, will disable writing to log
- Fix return early during searches that encounter stalemate

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
