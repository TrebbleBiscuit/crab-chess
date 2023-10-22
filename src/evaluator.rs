use chess::Color::{Black, White};
use chess::Piece::{Bishop, King, Knight, Pawn, Queen, Rook};
use chess::{Board, BoardStatus, ChessMove, Game, MoveGen, Piece, Square, ALL_PIECES};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info, trace};
use std::collections::HashMap;
use std::time::{Duration, Instant};

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
    transposition_table: HashMap<String, (i32, ChessMove)>,
    trans_table_depth_threshold: usize,
}

impl EvaluatorBot2010 {
    pub fn new() -> EvaluatorBot2010 {
        let pawn_pst = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 78, 83, 86, 73, 102, 82, 85, 90, 7, 29, 21, 44, 40, 31, 44, 7,
            -17, 16, -2, 15, 14, 0, 15, -13, -26, 3, 10, 9, 6, 1, 0, -23, -22, 9, 5, -11, -10, -2,
            3, -19, -31, 8, -7, -37, -36, -14, 3, -31, 0, 0, 0, 0, 0, 0, 0, 0,
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
            transposition_table: HashMap::new(),
            trans_table_depth_threshold: 5,
        }
    }

    fn evaluate_material(&self, board: &Board) -> i32 {
        // Returns a positive value if the player whose turn it is is winning
        let mut total_score: i32 = 0;
        for piece in ALL_PIECES {
            for square in *board.pieces(piece) {
                let index = square.to_index();
                if board.color_on(square) == Some(White) {
                    total_score += match piece {
                        Pawn => self.pawn_w_pst[index] + 100,
                        Knight => self.knight_w_pst[index] + 320,
                        Bishop => self.bishop_w_pst[index] + 330,
                        Rook => self.rook_w_pst[index] + 500,
                        Queen => self.queen_w_pst[index] + 900,
                        King => self.king_w_pst[index],
                    };
                } else {
                    total_score -= match piece {
                        Pawn => self.pawn_pst[index] + 100,
                        Knight => self.knight_pst[index] + 320,
                        Bishop => self.bishop_pst[index] + 330,
                        Rook => self.rook_pst[index] + 500,
                        Queen => self.queen_pst[index] + 900,
                        King => self.king_pst[index],
                    };
                }
            }
        }
        match board.side_to_move() {
            White => return total_score,
            Black => return -total_score,
        }
    }

    fn push_to_transposition_table(&mut self, hash: u64, depth: usize, score: i32, mv: ChessMove) {
        trace!("Pushing {} to transposition table at depth {}", hash, depth);
        self.transposition_table
            .insert(String::from(format!("{}-{}", hash, depth)), (score, mv));
    }

    fn get_from_transposition_table(&self, hash: u64, depth: usize) -> Option<&(i32, ChessMove)> {
        // get value from the transposition table
        let mut val = None;
        for try_depth in depth..=10 {
            match self
                .transposition_table
                .get(&String::from(format!("{}-{}", hash, try_depth)))
            {
                Some(cache_val) => {
                    debug!("got {:?} from transpo table", cache_val);
                    val = Some(cache_val)
                }
                None => (),
            }
        }
        return val;
    }

    fn get_move_order(&self, board: &Board, move_iterator: MoveGen) -> Vec<ChessMove> {
        // Pass in a MoveGen to grab moves from
        // returns a vector of moves lazily ordered by guess of which is best
        let mut guess_values: Vec<(ChessMove, i32)> = Vec::new();
        let mut guess_score: i32;
        for mv in move_iterator {
            guess_score = 0;
            match mv.get_promotion() {
                // promotions are good
                Some(piece) => guess_score += self.piece_values.get(&piece).unwrap(),
                _ => (),
            }
            match board.piece_on(mv.get_dest()) {
                // for captures, score is enemy piece value minus a fraction of our piece value
                // capturing cheap pieces with valuable pieces is likely a bad idea
                Some(piece) => {
                    guess_score += self.piece_values.get(&piece).unwrap()
                        - (self
                            .piece_values
                            .get(&board.piece_on(mv.get_source()).unwrap())
                            .unwrap()
                            / 2)
                }
                None => (),
            }
            guess_values.push((mv, guess_score))
        }

        guess_values.sort_by(|a, b| b.1.cmp(&a.1));
        let mut ordered_moves: Vec<ChessMove> = Vec::new();
        // order moves from best to worst
        for (mv, _) in guess_values.iter() {
            ordered_moves.push(*mv)
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
        let mut move_order: Vec<ChessMove> = Vec::new();
        let movegen: MoveGen = MoveGen::new_legal(&board);
        let mut chosen_move: ChessMove = ChessMove::new(Square::A1, Square::A1, None);

        let kill_time = Instant::now() + time_low_bar;
        for mv in self.get_move_order(board, movegen) {
            move_order.push(mv)
        }
        if depth < 3 {
            panic!("depth must be >= 3");
        }
        for n in 3..=depth {
            // print!("move rankings: ");
            // for mv in move_order.iter() {
            //     print!("{} ", mv);
            // }
            // println!("");
            // println!("Move order: {:?}", move_order);
            (score, chosen_move, move_order) =
                self.top_level_search(board, game, n, move_order, kill_time);
            // println!("Move order: {:?}", move_order);
            info!(
                "Depth: {} - {} @ {} - {:?}",
                n,
                chosen_move,
                score,
                start_time.elapsed()
            );
            // println!("Depth: {} - {:?}, Move order: {:?}", n, start_time.elapsed(), move_order);
            if n >= depth {
                debug!("Reached maximum depth...");
                return (score, chosen_move);
            } else if kill_time.elapsed() > Duration::ZERO {
                debug!("Too much time elapsed to continue search...");
                return (score, chosen_move);
            }
        }
        return (score, chosen_move);
    }

    fn top_level_search(
        &mut self,
        board: &Board,
        game: &Game,
        depth: usize,
        move_order: Vec<ChessMove>,
        kill_time: Instant,
    ) -> (i32, ChessMove, Vec<ChessMove>) {
        let mut alpha = -9999999; // this must be worse than losing
        let beta = 9999999;
        // Search for the best move using alpha-beta pruning
        // assumes depth > 0

        // match self.get_from_transposition_table(board.get_hash(), depth) {
        //     Some(cache_info) => {
        //         trace!("Returning info from cache lvl {}: {:?}", depth, cache_info);
        //         let (score, best_move) = *cache_info;
        //         return (score, best_move, move_order)
        //     }
        //     None => ()
        // }

        let mut best_move = ChessMove::new(Square::A1, Square::A1, None); // default;
                                                                          // let mut moves_searched = Vec::new();
        let mut move_values: Vec<(ChessMove, i32)> = Vec::new();
        // debug!("Searching {} moves at depth {}", move_order.len(), depth);
        let bar = ProgressBar::new(move_order.len() as u64);
        bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed} - ETA: {eta}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
        );
        for mv in move_order {
            bar.inc(1);
            // trace!("  {}", mv.to_string());
            let nboard = board.make_move_new(mv);

            let (move_search_score, _best_response_mv) =
                self.search(&nboard, game, depth - 1, -beta, -alpha);

            // move_search_score is the score of the best response move from our opponent
            // invert it; we'll pick the move with the highest score - gives our opponent the worst best response
            let evaluation = if game.can_declare_draw() {
                0
            } else {
                -move_search_score
            };
            move_values.push((mv, evaluation));

            if evaluation > alpha {
                alpha = evaluation;
                best_move = mv;
                // trace!("    ->{}!! ", evaluation);
            } else {
                // trace!("    ->{}  ", evaluation);
                // print!(".. ");
                // io::stdout().flush().unwrap();
            }

            // cancel search if we're out of time
            if kill_time.elapsed() > Duration::ZERO {
                info!("Killing search because we're out of time");
                break;
            }
        }
        trace!("");
        move_values.sort_by(|a, b| b.1.cmp(&a.1));
        // trace!("Sored move values: {:#?}", move_values);
        let mut order_moves: Vec<ChessMove> = Vec::new();
        // order moves from best to worst
        for (mv, _) in move_values.iter() {
            order_moves.push(*mv)
        }

        // if depth >= self.trans_table_depth_threshold {
        //     self.push_to_transposition_table(board.get_hash(), depth, alpha, best_move)
        // }

        // println!("order_moves: {:?}", order_moves);
        return (alpha, best_move, order_moves);
    }

    fn search(
        &mut self,
        board: &Board,
        game: &Game,
        depth: usize,
        mut alpha: i32,
        beta: i32,
    ) -> (i32, ChessMove) {
        // Search for the best move using alpha-beta pruning
        let default_move = ChessMove::new(Square::A1, Square::A1, None);
        if game.can_declare_draw() {
            return (0, default_move);
        }
        if depth == 0 {
            // assumes depth > 0 when this fn is called for the first time
            // otherwise it will return default_move
            return (
                self.search_only_captures(board, game, alpha, beta),
                default_move,
            );
        }

        // match self.get_from_transposition_table(board.get_hash(), depth) {
        //     Some(cache_info) => {
        //         trace!("Returning info from cache lvl {}: {:?}", depth, cache_info);
        //         return *cache_info
        //     }
        //     None => ()
        // }

        let movegen: MoveGen = MoveGen::new_legal(&board);
        if movegen.len() == 0 {
            if board.status() == BoardStatus::Checkmate {
                return (i32::from(-999999), default_move);
            } else {
                return (i32::from(0), default_move);
            }
        }
        let mut best_move: ChessMove = default_move;
        let mut best_score = -999999; // this is distinct from alpha; it may be smaller if no moves are better
        for mv in self.get_move_order(board, movegen) {
            let nboard = board.make_move_new(mv);
            let (move_search_score, _next_move) =
                self.search(&nboard, game, depth - 1, -beta, -alpha);
            let evaluation = -move_search_score;
            if evaluation >= beta {
                // position is too good; opponent would never let us get here
                return (evaluation, mv);
            }
            if evaluation > alpha {
                alpha = evaluation;
                best_score = evaluation;
                best_move = mv;
            } else if evaluation > best_score {
                // so that if no moves are better, this fn will return its own best result instead of the one given to it
                best_score = evaluation;
                best_move = mv;
            }
        }

        // if depth >= self.trans_table_depth_threshold {
        //     self.push_to_transposition_table(board.get_hash(), depth, best_score, best_move)
        // }

        return (best_score, best_move);
    }

    fn search_only_captures(&self, board: &Board, game: &Game, mut alpha: i32, beta: i32) -> i32 {
        // Search for the best move using alpha-beta pruning
        // This time, only look for captures at infinite depth
        let evaluation = self.evaluate_material(board);
        if evaluation >= beta {
            return evaluation;
        }
        if evaluation > alpha {
            alpha = evaluation;
        }
        // filter targets
        let targets = board.color_combined(!board.side_to_move());
        let mut movegen: MoveGen = MoveGen::new_legal(&board);
        movegen.set_iterator_mask(*targets);
        if movegen.len() == 0 {
            if board.status() == BoardStatus::Checkmate {
                return i32::from(-999999);
            } else if board.status() == BoardStatus::Stalemate {
                return i32::from(0);
            }
            // no attacking moves does not mean stalemate here
        }
        let mut best_score = evaluation;
        for mv in self.get_move_order(board, movegen) {
            let nboard = board.make_move_new(mv);
            let move_search_score = self.search_only_captures(&nboard, game, -beta, -alpha);
            let evaluation = -move_search_score;
            if evaluation >= beta {
                return evaluation;
            }
            if evaluation > alpha {
                alpha = evaluation;
                best_score = evaluation;
            } else if evaluation > best_score {
                best_score = evaluation;
            }
        }
        return best_score;
    }
}
