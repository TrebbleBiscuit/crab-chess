use chess::Color::{self, Black, White};
use chess::Piece::{Bishop, King, Knight, Pawn, Queen, Rook};
use chess::{BitBoard, Board, BoardStatus, ChessMove, Game, MoveGen, Piece, Square, ALL_PIECES};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info, trace};
use std::collections::HashMap;
use std::time::{Duration, Instant};

const MAXIMUM_SEARCH_DEPTH: usize = 40; // search will NEVER exceed this depth
const CHECK_MV_SEARCH_DEPTH: usize = 20; // search will only evaluate captures (not check) after this depth

#[derive(Clone, Copy, PartialEq, PartialOrd)]

enum NodeType {
    UpperBound,
    Exact,
    LowerBound,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct Transposition {
    depth: usize,
    ply: usize,
    score: i32,
    node_type: NodeType,
    best_move: ChessMove,
}

impl Transposition {
    fn empty() -> Self {
        Self {
            depth: 0,
            ply: 0,
            score: 0,
            node_type: NodeType::Exact,
            best_move: ChessMove::new(Square::A1, Square::A1, None),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SearchStats {
    nodes_searched: i32,
    boards_evaluated: i32,
    tt_hits: i32,
    tt_upper_hits: i32,
    tt_exact_hits: i32,
    tt_lower_hits: i32,
    depth_reduction_misses: i32,
    max_ply: usize,
}

impl std::ops::AddAssign for SearchStats {
    // Define the behavior of the += operator for MyStruct
    fn add_assign(&mut self, other: Self) {
        // Modify the fields of self by adding the corresponding fields from other
        self.nodes_searched += other.nodes_searched;
        self.boards_evaluated += other.boards_evaluated;
        self.tt_hits += other.tt_hits;
        self.tt_upper_hits += other.tt_upper_hits;
        self.tt_exact_hits += other.tt_exact_hits;
        self.tt_lower_hits += other.tt_lower_hits;
        self.depth_reduction_misses += other.depth_reduction_misses;
        self.max_ply = self.max_ply.max(other.max_ply); // set ply to max instead of adding
    }
}

struct TranspositionTable(chess::CacheTable<Transposition>);

impl TranspositionTable {
    fn new() -> Self {
        Self(
            // 2^18 is 262,144
            // at ~40b each that's around 10.5 megabytes
            chess::CacheTable::new(1 << 18, Transposition::empty()),
        )
    }

    fn insert(&mut self, key: u64, value: Transposition) {
        self.0.add(key, value);
    }

    fn get(&self, key: u64, depth: usize) -> Option<Transposition> {
        if let Some(val) = self.0.get(key) {
            if val.depth < depth {
                return None;
            }
        } else {
            return None;
        }
        self.0.get(key)
    }
}

pub fn debug_pp() {
    let start_time = Instant::now();
    let mut count = 0;
    for n in 0..10000000 {
        for square in BitBoard::new(u64::MAX) {
            passed_pawn_mask_from_square(square, White);
            passed_pawn_mask_from_square(square, Black);
            count += 2;
        }
    }
    println!("Created {} pawn masks in {:?}", count, start_time.elapsed())
}

fn adjacent_files_mask() {
    // :/
}

fn passed_pawn_mask_from_square(pawn_square: Square, pawn_color: Color) -> u64 {
    // Check if there are no enemy pawns in the same file or adjacent files
    let file_index = pawn_square.get_file().to_index();

    // Generate masks for the 2-3 columns adjacent to the pawn
    let file_a = 0x0101010101010101u64;
    let file_mask_center = file_a << file_index;
    let file_mask_left = file_a << (file_index).max(1) - 1;
    let file_mask_right = file_a << (file_index + 1).min(7);
    let total_file_mask = file_mask_center | file_mask_left | file_mask_right;

    // Generate masks for the rows below or above the pawn
    let rank_index = pawn_square.get_rank().to_index();
    let passed_pawn_mask = match pawn_color {
        White => {
            let rank_mask_above = u64::MAX << 8 * (7 - rank_index);
            rank_mask_above & total_file_mask
        }
        Black => {
            let rank_mask_below = u64::MAX >> 8 * (8 - rank_index);
            rank_mask_below & total_file_mask
        }
    };
    passed_pawn_mask
}

pub struct EvaluatorBot2010 {
    pawn_pst: Vec<i32>,
    knight_pst: Vec<i32>,
    bishop_pst: Vec<i32>,
    rook_pst: Vec<i32>,
    queen_pst: Vec<i32>,
    king_pst: Vec<i32>,
    pawn_w_pst: Vec<i32>,
    knight_w_pst: Vec<i32>,
    bishop_w_pst: Vec<i32>,
    rook_w_pst: Vec<i32>,
    queen_w_pst: Vec<i32>,
    king_w_pst: Vec<i32>,
    piece_values: HashMap<Piece, i32>,
    transposition_table: TranspositionTable,
    trans_table_depth_threshold: usize,
    search_stats: SearchStats,
}

impl EvaluatorBot2010 {
    pub fn new() -> EvaluatorBot2010 {
        let pawn_pst = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 278, 283, 286, 273, 302, 282, 285, 290, 107, 129, 121, 144,
            140, 131, 144, 107, 2, 36, 18, 35, 34, 20, 35, 7, -26, 3, 10, 9, 6, 1, 0, -23, -22, 9,
            5, -11, -10, -2, 3, -19, -31, 8, -7, -37, -36, -14, 3, -31, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let knight_pst = vec![
            -66, -53, -75, -75, -10, -55, -58, -70, -3, -6, 100, -36, 4, 62, -4, -14, 10, 67, 1,
            74, 73, 27, 62, -2, 24, 24, 45, 37, 33, 41, 25, 17, -1, 5, 31, 21, 22, 35, 2, 0, -18,
            10, 13, 22, 18, 15, 11, -14, -23, -15, 2, 0, 2, 0, -23, -20, -74, -23, -26, -24, -19,
            -35, -22, -69,
        ];
        let bishop_pst = vec![
            -59, -78, -82, -76, -23, -107, -37, -50, -11, 20, 35, -42, -39, 31, 2, -22, -9, 39,
            -32, 41, 52, -10, 28, -14, 25, 17, 20, 34, 26, 25, 15, 10, 13, 10, 17, 23, 17, 16, 0,
            7, 14, 25, 24, 15, 8, 25, 20, 15, 19, 20, 11, 6, 7, 6, 20, 16, -7, 2, -15, -12, -14,
            -15, -10, -10,
        ];
        let rook_pst = vec![
            35, 29, 33, 4, 37, 33, 56, 50, 55, 29, 56, 67, 55, 62, 34, 60, 19, 35, 28, 33, 45, 27,
            25, 15, 0, 5, 16, 13, 18, -4, -9, -6, -28, -35, -16, -21, -13, -29, -46, -30, -42, -28,
            -42, -25, -25, -35, -26, -46, -53, -38, -31, -26, -29, -43, -44, -53, -30, -24, -18, 5,
            -2, -18, -31, -32,
        ];
        let queen_pst = vec![
            6, 1, -8, -104, 69, 24, 88, 26, 14, 32, 60, -10, 20, 76, 57, 24, -2, 43, 32, 60, 72,
            63, 43, 2, 1, -16, 22, 17, 25, 20, -13, -6, -14, -15, -2, -5, -1, -10, -20, -22, -30,
            -6, -13, -11, -16, -11, -16, -27, -36, -18, 0, -19, -15, -15, -21, -38, -39, -30, -31,
            -13, -31, -36, -34, -42,
        ];
        let king_pst = vec![
            4, 54, 47, -99, -99, 60, 83, -62, -32, 10, 55, 56, 56, 55, 10, 3, -62, 12, -57, 44,
            -67, 28, 37, -31, -55, 50, 11, -4, -19, 13, 0, -49, -55, -43, -52, -28, -51, -47, -8,
            -50, -47, -42, -43, -79, -64, -32, -29, -32, -4, 3, -14, -50, -57, -18, 13, 4, 17, 30,
            -3, -14, 6, -1, 40, 18,
        ];
        EvaluatorBot2010 {
            // board: Board::default(),
            pawn_pst: pawn_pst.to_vec(),
            knight_pst: knight_pst.to_vec(),
            bishop_pst: bishop_pst.to_vec(),
            rook_pst: rook_pst.to_vec(),
            queen_pst: queen_pst.to_vec(),
            king_pst: king_pst.to_vec(),
            pawn_w_pst: pawn_pst.iter().copied().rev().collect(),
            knight_w_pst: knight_pst.iter().copied().rev().collect(),
            bishop_w_pst: bishop_pst.iter().copied().rev().collect(),
            rook_w_pst: rook_pst.iter().copied().rev().collect(),
            queen_w_pst: queen_pst.iter().copied().rev().collect(),
            king_w_pst: king_pst.iter().copied().rev().collect(),
            piece_values: [
                (Pawn, 100),
                (Knight, 330),
                (Bishop, 330),
                (Rook, 500),
                (Queen, 900),
                (King, 5000),
            ]
            .iter()
            .cloned()
            .collect(),
            transposition_table: TranspositionTable::new(),
            trans_table_depth_threshold: 2,
            search_stats: SearchStats::default(),
        }
    }

    fn king_safety(&self, board: &Board, square: Square, king_color: Color) -> i32 {
        // king safety

        // this is important near the beginning of the game
        // but in the endgame when there are fewer enemy pieces you want to open up
        // start w 20 pieces, start scaling down safety factor at 15, hits zero at 5
        let enemy_pieces_remaining = board.color_combined(!king_color).popcnt();
        // safety is multiplied by the clamped # of enemy pieces remaining
        // more threats around means safety is more important
        let safety_factor = (enemy_pieces_remaining.clamp(5, 15) - 5); // 0 to 10

        if safety_factor == 0 {
            return 0;
        }

        // pretend there's a bishop, then a rook, where the king is
        // more moves is bad because the king is vulnerable to many attacks
        let mut safety = 5;
        let blockers = board.combined();
        safety -= chess::get_rook_moves(square, *blockers).popcnt() as i32;
        safety -= chess::get_bishop_moves(square, *blockers).popcnt() as i32;

        // TODO: tune
        // ends up being worth like 0-2 pawns
        let safety = safety * safety_factor as i32;
        return safety;
    }

    fn is_passed_pawn(
        &self,
        enemy_pawns: &BitBoard,
        pawn_square: Square,
        pawn_color: Color,
    ) -> bool {
        // If an enemy pawn is not in the passed pawn mask, it is indeed a passed pawn
        let passed_pawn_mask = passed_pawn_mask_from_square(pawn_square, pawn_color);
        (enemy_pawns.0 & passed_pawn_mask) == 0

        // debug messages
        // if pawn_square == Square::F4 || pawn_square == Square::B3 {
        //     info!("{:?}", file_mask_left);
        //     info!("{:?}", file_mask_center);
        //     info!("{:?}", file_mask_right);
        //     info!("{:?}", total_file_mask);
        //     info!("{:?}", passed_pawn_mask);
        //     for sq in BitBoard::new(passed_pawn_mask) {
        //         debug!("{:?}", sq)
        //     }
        //     debug!("now here are pawns:");
        //     for sq in *enemy_pawns {
        //         debug!("{:?}", sq)
        //     }
        // }
    }

    fn pawn_bonus_value(&self, square: Square, pawn_color: Color, enemy_pawns: &BitBoard) -> i32 {
        let mut bonus_value = 0;
        // passed pawns are good, even better with less material on the board
        // only look for passed pawns after the opponent has lost 6 pieces
        if self.is_passed_pawn(enemy_pawns, square, pawn_color) {
            let squares_to_promotion = {
                // white is 1 square away at rank 7
                // black is 6 squares away at rank 7
                let rank = square.get_rank().to_index(); // 0 to 7
                match pawn_color {
                    White => 7 - rank,
                    Black => rank,
                }
            };
            bonus_value += match squares_to_promotion {
                1 => 150,
                2 => 90,
                3 => 50,
                4 => 20,
                _ => 15,
            };
        };
        return bonus_value as i32;
    }

    fn evaluate_material(&self, board: &Board) -> i32 {
        // Returns a positive value if the player whose turn it is is winning
        let mut total_score: i32 = 0;
        // we'll use this bitboard to calculate pawn bonus value
        let white_pawns = board.color_combined(Color::White) & board.pieces(Piece::Pawn);
        let black_pawns = board.color_combined(Color::Black) & board.pieces(Piece::Pawn);
        for piece in ALL_PIECES {
            for square in *board.pieces(piece) {
                let index = square.to_index();
                if board.color_on(square) == Some(White) {
                    total_score += match piece {
                        Pawn => {
                            self.pawn_w_pst[index]
                                + 100
                                + self.pawn_bonus_value(square, White, &black_pawns)
                        }
                        Knight => self.knight_w_pst[index] + 320,
                        Bishop => self.bishop_w_pst[index] + 330,
                        Rook => self.rook_w_pst[index] + 500,
                        Queen => self.queen_w_pst[index] + 900,
                        King => self.king_w_pst[index] + self.king_safety(board, square, White),
                    };
                } else {
                    total_score -= match piece {
                        Pawn => {
                            self.pawn_pst[index]
                                + 100
                                + self.pawn_bonus_value(square, Black, &white_pawns)
                        }
                        Knight => self.knight_pst[index] + 320,
                        Bishop => self.bishop_pst[index] + 330,
                        Rook => self.rook_pst[index] + 500,
                        Queen => self.queen_pst[index] + 900,
                        King => self.king_pst[index] + self.king_safety(board, square, Black),
                    };
                }
            }
        }
        // also calculate mobility
        // actually this should be the *difference* in mobility
        // this overflows the stack
        // total_score += (MoveGen::new_legal(&board).len() * 2) as i32;

        match board.side_to_move() {
            White => return total_score,
            Black => return -total_score,
        }
    }

    fn get_moves_lazily_ordered(
        &self,
        board: &Board,
        move_iterator: MoveGen,
        suggested_moves: Option<Vec<ChessMove>>,
    ) -> Vec<(ChessMove, i32)> {
        // Pass in a MoveGen to grab moves from
        // returns a vector of moves lazily ordered by guess of which is best
        let mut guess_values: Vec<(ChessMove, i32)> = Vec::new();
        let mut guess_score: i32;
        let all_suggested_moves = match suggested_moves {
            Some(suggested_move) => suggested_move,
            None => vec![],
        };
        for mv in move_iterator {
            // evaluating material here is too expensive
            guess_score = 0;

            // push suggested moves up
            if all_suggested_moves.contains(&mv) {
                guess_score += 10000
            }

            match mv.get_promotion() {
                // promotions are good
                Some(piece) => guess_score += self.piece_values.get(&piece).unwrap(),
                _ => (),
            }
            let move_target = mv.get_dest();
            match board.piece_on(move_target) {
                // for captures, score is enemy piece value minus a fraction of our piece value
                // capturing cheap pieces with valuable pieces is likely a bad idea
                Some(piece) => {
                    guess_score += self.piece_values.get(&piece).unwrap()
                        - (self
                            .piece_values
                            .get(&board.piece_on(mv.get_source()).unwrap())
                            .unwrap()
                            / 2);
                    // a capture that can be recaptured by an enemy is worse
                    let pawn_attacks = chess::get_pawn_attacks(
                        move_target,
                        board.side_to_move(),
                        BitBoard::new(0),
                    );
                    // trace!("pawn attacks: {}", pawn_attacks)
                }
                None => (),
            }
            guess_values.push((mv, guess_score))
        }

        guess_values.sort_by(|a, b| b.1.cmp(&a.1));
        let mut ordered_moves: Vec<(ChessMove, i32)> = Vec::new();
        // order moves from best to worst
        for (mv, score) in guess_values.iter() {
            ordered_moves.push((*mv, *score))
        }
        ordered_moves
    }

    pub fn iterative_search_deepening(
        &mut self,
        board: &Board,
        game: &Game,
        depth: usize,
        time_low_bar: Duration,
    ) -> (i32, ChessMove) {
        let start_time = Instant::now();
        let mut score: i32 = 111111;
        let mut move_order: Vec<(ChessMove, i32)> = Vec::new();
        let movegen: MoveGen = MoveGen::new_legal(&board);
        let mut chosen_move: ChessMove = ChessMove::new(Square::A1, Square::A1, None);
        let mut best_resp: ChessMove = ChessMove::new(Square::A1, Square::A1, None);

        trace!("---- start search ----");
        debug!("Selecting a move from position {}", board.to_string());

        // debug!("DEBUG: clearing transpo table");
        // self.transposition_table = HashMap::new();

        let kill_time = Instant::now() + time_low_bar;
        for mv in self.get_moves_lazily_ordered(board, movegen, None) {
            move_order.push(mv)
        }
        // if depth < 3 {
        //     panic!("depth must be >= 3");
        // }
        let mut cum_search_stats = SearchStats::default();
        // iteratively search at multiple depths
        for n in 2.min(depth)..=depth {
            // TODO: do i need to .step_by(2)?

            // execute a top level search
            (score, chosen_move, move_order, best_resp) =
                self.top_level_search(board, game, n, move_order, kill_time);
            info!(
                "Depth: {} - {} -> {} @ {} - {:?}",
                n,
                chosen_move,
                best_resp,
                score,
                start_time.elapsed()
            );
            // for uci
            println!(
                "info depth {} seldepth {} score cp {} time {} pv {} {}",
                n,
                cum_search_stats.max_ply,
                score,
                start_time.elapsed().as_millis(),
                chosen_move,
                best_resp
            );
            // trace!("TT Size is now {}", self.transposition_table.0.len());
            // add to cumulative search stats then clear search_stats for next time
            cum_search_stats += self.search_stats;
            self.search_stats = SearchStats::default();

            // debug!("best response: {}", best_resp);
            let mut move_scores_output = "Move scores: ".to_string();
            for (mv, score) in move_order.iter() {
                move_scores_output += format!(" {} @ {} ", mv, score).as_str();
            }
            debug!("{}", move_scores_output);

            if n >= depth {
                debug!("Reached maximum depth...");
                break;
            } else if kill_time.elapsed() > Duration::ZERO {
                debug!("Too much time elapsed to continue search...");
                break;
            }
        }
        debug!("Cumulative search stats:");
        debug!("{:?}", cum_search_stats);
        trace!("---- End search ----");
        trace!("");
        return (score, chosen_move);
    }

    fn top_level_search(
        &mut self,
        board: &Board,
        game: &Game,
        depth: usize,
        move_order: Vec<(ChessMove, i32)>,
        kill_time: Instant,
    ) -> (i32, ChessMove, Vec<(ChessMove, i32)>, ChessMove) {
        let mut alpha = -999999777; // this must be worse than losing
        let beta = 999999777;
        // Search for the best move using alpha-beta pruning
        // assumes depth > 0

        // TODO: EXPERIMENT - DO NOT GET TOP LEVEL SEARCH FROM TRANSPOSITION TABLE
        // this is so that we can preserve move order, making iterative deepening viable
        // match self.get_from_transposition_table(board.get_hash(), depth) {
        //     Some(cache_info) => {
        //         let (score, best_move) = cache_info;
        //         // make sure that the move order we return has the best move first
        //         // so that if we do another search we start with the best move
        //         let mut new_moves = vec![(best_move, score)];
        //         for entry in move_order.iter() {
        //             if entry != &(best_move, score) {
        //                 new_moves.push(*entry)
        //             }
        //         }
        //         // if let Some(mv_index) = move_order.iter().position(|x| *x == (best_move, score)) {
        //         //     move_order.remove(mv_index);
        //         //     move_order.insert(0, (best_move, score));
        //         // }
        //         let response_unavailable = ChessMove::new(Square::A2, Square::A2, None);
        //         return (score, best_move, new_moves, response_unavailable);
        //     }
        //     None => (),
        // }

        let mut best_move = ChessMove::new(Square::A1, Square::A1, None); // default;
        let mut best_response = ChessMove::new(Square::A1, Square::A1, None);
        // let mut moves_searched = Vec::new();

        // return move_values at the end, it'll be like the new version of move_order
        let mut move_values: Vec<(ChessMove, i32)> = Vec::new();
        // debug!("Searching {} moves at depth {}", move_order.len(), depth);
        let bar = ProgressBar::new(move_order.len() as u64);
        bar.set_message(format!("{}", alpha));
        bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed} - ETA: {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
        );
        for (mv_index, (mv, _)) in move_order.iter().enumerate() {
            // search extensions! look deeper for capture moves
            bar.inc(1);
            // if depth > 5 {
            //     trace!("  now evaluating: {}", mv.to_string());
            // }
            let nboard = board.make_move_new(*mv);
            let default_move = ChessMove::new(Square::A1, Square::A1, None);
            let mut hgame = game.clone();
            hgame.make_move(*mv);

            let (evaluation, this_response) = {
                alpha -= 1; // so that if mate in 2 is 9998 then mate in 3 is 9997

                // search capture moves deeper
                let more_depth = if board.piece_on(mv.get_dest()).is_some() {
                    1
                } else {
                    0
                };

                // first few moves are full depth, but search remaining non-capture moves at shallower depth
                let depth_reduction;
                // no depth reduction when depth <= 3
                // will enable better ordered moves during later searches
                if more_depth == 0
                    && depth > 3
                    && mv_index >= 3
                    && board.piece_on(mv.get_dest()).is_none()
                {
                    depth_reduction = 1;
                } else {
                    depth_reduction = 0;
                };
                // perform a possibly shallower search
                let (mut move_search_score, mut best_response_mv) = self.search(
                    &nboard,
                    &hgame,
                    (depth - 1 - depth_reduction).max(0),
                    1,
                    -beta,
                    -alpha,
                    kill_time,
                    Some(vec![best_response]),
                );
                if depth_reduction > 0 && -move_search_score > alpha {
                    // perform a full search to get a more accurate result, make sure
                    // that this great looking position really is great looking
                    self.search_stats.depth_reduction_misses += 1;
                    (move_search_score, best_response_mv) = self.search(
                        &nboard,
                        &hgame,
                        depth + more_depth - 1,
                        1,
                        -beta,
                        -alpha,
                        kill_time,
                        Some(vec![best_response]),
                    );
                }
                alpha += 1;
                // move_search_score is the score of the best response move from our opponent
                // invert it; we'll pick the move with the highest score - gives our opponent the worst best response
                (-move_search_score, best_response_mv)
            };
            move_values.push((*mv, evaluation));
            self.search_stats.nodes_searched += 1;

            if kill_time.elapsed() > Duration::ZERO {
                // the result we got in this search may not be accurate
                debug!("Out of time");
                // we can use this move if it's the first/only one we've looked at
                // otherwise we discard this result
                if best_move != default_move {
                    break;
                }
            }

            if evaluation > alpha {
                alpha = evaluation;
                best_move = *mv;
                best_response = this_response;
                bar.set_message(format!("{}", alpha));
                trace!(
                    "top level alpha set -> {} @ {}",
                    best_move.to_string(),
                    alpha
                );
                // trace!("    ->{}!! ", evaluation);
            }
            // else if evaluation < -999000 {
            //     trace!(
            //         "found forced checkmate in top level move {} -> {}",
            //         mv.to_string(),
            //         this_response.to_string()
            //     );
            // } else {
            //     // trace!("    ->{}  ", evaluation);
            //     // print!(".. ");
            //     // io::stdout().flush().unwrap();
            // }

            // // cancel search if we're out of time
            // MOVED UP
            // if kill_time.elapsed() > Duration::ZERO {
            //     info!("Killing search because we're out of time");
            //     break;
            // }
        }
        trace!("");
        move_values.sort_by(|a, b| b.1.cmp(&a.1));
        // trace!("Sored move values: {:#?}", move_values);
        let mut order_moves: Vec<(ChessMove, i32)> = Vec::new();
        // order moves from best to worst
        for (i, (mv, val)) in move_values.iter().enumerate() {
            // pruning these moves may be covering the root of the problem
            // but i can't imagine this doing anything but helping so
            if alpha > -900000 {
                if *val < -900000 {
                    // ignore losing moves in future searches
                    debug!("Avoiding a losing move - {}", mv.to_string());
                    continue;
                }
            }
            order_moves.push((*mv, *val))
        }

        // alpha is the evaluation of the position since this is the top level search
        if depth >= self.trans_table_depth_threshold && kill_time.elapsed() == Duration::ZERO {
            // Push exact result to transposition table since this is top level node
            self.transposition_table.insert(
                board.get_hash(),
                Transposition {
                    depth: depth,
                    ply: 0,
                    score: alpha,
                    node_type: NodeType::Exact,
                    best_move: best_move,
                },
            );
        }

        debug!("Finished top level search! evaluation: {}", alpha);

        // println!("order_moves: {:?}", order_moves);
        return (alpha, best_move, order_moves, best_response);
    }

    fn search(
        &mut self,
        board: &Board,
        game: &Game,
        depth: usize,
        ply: usize,
        mut alpha: i32,
        beta: i32,
        kill_time: Instant,
        suggested_moves: Option<Vec<ChessMove>>, // try this move first
    ) -> (i32, ChessMove) {
        // Search for the best move using alpha-beta pruning
        let default_move = ChessMove::new(Square::A1, Square::A1, None);

        if game.can_declare_draw() || board.status() == BoardStatus::Stalemate {
            return (0, default_move);
        }

        // trace!("search board {} is {:?}", board.to_string(), board.status());
        if board.status() == BoardStatus::Checkmate {
            // trace!(
            //     "checkmate against {:?} as evaluated by search() depth {} alpha {} beta {}",
            //     board.side_to_move(),
            //     depth,
            //     alpha,
            //     beta
            // );

            return (i32::from(-999995), default_move);
        }
        if depth == 0 {
            // assumes depth > 0 when this fn is called for the first time
            // otherwise it will return default_move
            return (
                self.search_only_captures(board, game, ply + 1, alpha, beta),
                default_move,
            );
        }

        let mut best_move: ChessMove = default_move;
        let mut best_score = -9999998; // this is distinct from alpha; it may be smaller if no moves are better

        if let Some(transpo) = self.transposition_table.get(board.get_hash(), depth) {
            // get the move from the transposition table
            self.search_stats.tt_hits += 1;
            match transpo.node_type {
                // if upper bound, check if eval < beta; perahps we can immediately prune
                NodeType::UpperBound => {
                    self.search_stats.tt_upper_hits += 1;
                    if transpo.score < beta {
                        return (transpo.score, transpo.best_move);
                    }
                }
                // if exact match, return that result
                NodeType::Exact => {
                    self.search_stats.tt_exact_hits += 1;
                    return (transpo.score, transpo.best_move);
                }
                // if lower bound, check if eval > alpha; perhaps this is the best move
                NodeType::LowerBound => {
                    self.search_stats.tt_lower_hits += 1;
                    if transpo.score > alpha {
                        // this could be a good move
                        alpha = transpo.score;
                        best_move = transpo.best_move;
                        best_score = transpo.score;
                    }
                }
            }
        }

        // for future transposition table
        let mut this_node_type = NodeType::UpperBound;

        let movegen: MoveGen = MoveGen::new_legal(&board);
        let mut best_response: ChessMove = default_move;
        for (mv, _) in self.get_moves_lazily_ordered(board, movegen, suggested_moves) {
            let nboard = board.make_move_new(mv);
            // the only way i could find to detect draws
            // make a new game, make the move, then check
            let mut hyp_game = game.clone();
            hyp_game.make_move(mv);
            let (move_search_score, sub_response) = match hyp_game.can_declare_draw() {
                true => return (0, default_move),
                false => self.search(
                    &nboard,
                    &hyp_game,
                    depth - 1,
                    ply + 1,
                    -beta,
                    -alpha,
                    kill_time,
                    Some(vec![best_response]),
                ),
            };
            // we don't have all the nodes on this tree yet

            // reduce checkmate moves in subsearches so that sooner mates are more valuable
            let evaluation = if move_search_score <= -999000 {
                -move_search_score - 2
            } else {
                -move_search_score
            };
            self.search_stats.nodes_searched += 1;

            if evaluation >= beta {
                // position is too good; opponent would never let us get here
                // if depth > 3 {
                //     trace!(
                //         "beta cutoff {} - {} -> {} @ {}",
                //         depth,
                //         mv.to_string(),
                //         resp.to_string(),
                //         evaluation
                //     );
                // }
                // a beta cutoff means we've failed high; this is a lower bound
                self.transposition_table.insert(
                    board.get_hash(),
                    Transposition {
                        depth: depth,
                        ply: ply,
                        score: evaluation,
                        node_type: NodeType::LowerBound,
                        best_move: mv,
                    },
                );
                return (evaluation, mv);
                // return (beta, mv);
            }
            if evaluation > alpha {
                alpha = evaluation;
                best_score = evaluation;
                best_move = mv;
                best_response = sub_response;
                // since at least one search exceeded alpha, we know it's exact
                this_node_type = NodeType::Exact
            } else if evaluation > best_score {
                // so that if no moves are better, this fn will return its own best result instead of the one given to it
                best_score = evaluation;
                best_move = mv;
                best_response = sub_response;
            }
            // cancel search if we're out of time
            if kill_time.elapsed() > Duration::ZERO {
                // trace!("breaking from subsearch at kill time");
                break;
            }
        }

        if depth >= self.trans_table_depth_threshold && kill_time.elapsed() == Duration::ZERO {
            // if no move scores exceeded alpha, this is an upper bound and the true score may be less
            // otherwise it's the true score
            match this_node_type {
                NodeType::LowerBound => unreachable!(),
                _ => self.transposition_table.insert(
                    board.get_hash(),
                    Transposition {
                        depth: depth,
                        ply: ply,
                        score: best_score,
                        node_type: this_node_type,
                        best_move: best_move,
                    },
                ),
            }
        }

        return (best_score, best_move);
    }

    fn search_only_captures(
        &mut self,
        board: &Board,
        game: &Game,
        ply: usize,
        mut alpha: i32,
        beta: i32,
    ) -> i32 {
        if ply > self.search_stats.max_ply {
            self.search_stats.max_ply = ply
        }
        // filter targets
        let targets = board.color_combined(!board.side_to_move());
        let mut movegen: MoveGen = MoveGen::new_legal(&board);
        movegen.set_iterator_mask(*targets);
        if movegen.len() == 0 {
            if board.status() == BoardStatus::Checkmate {
                // trace!(
                //     "found checkmate for {:?} in search_only_captures",
                //     board.side_to_move()
                // );
                return i32::from(-999995);
            } else if board.status() == BoardStatus::Stalemate {
                return i32::from(0);
            }
            // no attacking moves -> evaluate the board
        }
        // assuming that this player can always take the evaluation on the board as is
        // instead of making a capture - because many capture moves are bad
        let evaluation = self.evaluate_material(board);
        self.search_stats.boards_evaluated += 1;
        if evaluation >= beta {
            // return beta;
            return evaluation;
        }
        if evaluation > alpha {
            alpha = evaluation;
        }
        // if we're in too deep, bail out
        if ply >= MAXIMUM_SEARCH_DEPTH {
            debug!("Bailing out at max search depth {}", ply);
            return evaluation;
        }
        // first look through attacking moves
        // for (mv, _) in self.get_moves_lazily_ordered(board, movegen) {
        let mut best_eval = -99999999;
        for mv in &mut movegen {
            let nboard = board.make_move_new(mv);
            let move_search_score =
                self.search_only_captures(&nboard, game, ply + 1, -beta, -alpha);
            let score = -move_search_score;
            self.search_stats.nodes_searched += 1;
            if score >= beta {
                // opponent would never let us get here
                // return beta;
                return score;
            }
            if score > best_eval {
                best_eval = score;
            }
            if score > alpha {
                // wow a great result!
                alpha = score;
            }
        }
        // then look through other moves
        // unless we're too deep
        if ply >= CHECK_MV_SEARCH_DEPTH {
            debug!("Ignoring checks after check move search depth {}", ply);
            return best_eval.max(evaluation);
        }
        movegen.set_iterator_mask(!chess::EMPTY);
        for (mv, _) in self.get_moves_lazily_ordered(board, movegen, None) {
            // we only want to continue evaluating if EITHER of these conditions is true
            // (1) this move puts our opponent in check
            // (2) we are in check right now - this move must get us out of it
            //     and we want to consider our opponent's immediate response
            let nboard = board.make_move_new(mv);
            // bouncer
            if (nboard.checkers().to_size(0) == 0) & (board.checkers().to_size(0) == 0) {
                // this particular move is not interesting
                // but there may be other interesting moves
                continue;
                // score = evaluation;
            }
            // keep searching deeper
            let score;
            let move_search_score =
                self.search_only_captures(&nboard, game, ply + 1, -beta, -alpha);
            score = -move_search_score;
            self.search_stats.nodes_searched += 1;
            if score >= beta {
                // opponent would never let us get here
                // return beta;
                return score;
            }
            if score > best_eval {
                best_eval = score;
            }
            if score > alpha {
                // wow a great result!
                alpha = score;
            }
        }
        return best_eval.max(evaluation);
        // return alpha;
    }
}
