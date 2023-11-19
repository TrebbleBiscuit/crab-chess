use crate::crab_evaluate;
use crate::crab_transposition;
use chess::Piece::{Bishop, King, Knight, Pawn, Queen, Rook};
use chess::{Board, BoardStatus, ChessMove, Game, MoveGen, Piece, Square, EMPTY};
use crab_transposition::{NodeType, Transposition, TranspositionTable};
use log::{debug, trace};
use std::collections::HashMap;
use std::time::{Duration, Instant};

const MAXIMUM_SEARCH_DEPTH: usize = 40; // search will NEVER exceed this depth
const CHECK_MV_SEARCH_DEPTH: usize = 20; // search will only evaluate captures (not check) after this depth

const STALEMATE_SCORE: i32 = 0;
const CHECKMATE_SCORE: i32 = -999995;

#[derive(Debug, Default, Clone, Copy)]
pub struct SearchStats {
    nodes_searched: i32,
    boards_evaluated: i32,
    tt_pushed: i32,
    tt_hits: i32,
    tt_upper_hits: i32,
    tt_exact_hits: i32,
    tt_lower_hits: i32,
    depth_reduction_misses: i32,
    depth_reduction_hits: i32,
    max_ply: usize,
}

impl std::ops::AddAssign for SearchStats {
    // Define the behavior of the += operator for MyStruct
    fn add_assign(&mut self, other: Self) {
        // Modify the fields of self by adding the corresponding fields from other
        self.nodes_searched += other.nodes_searched;
        self.boards_evaluated += other.boards_evaluated;
        self.tt_pushed += other.tt_pushed;
        self.tt_hits += other.tt_hits;
        self.tt_upper_hits += other.tt_upper_hits;
        self.tt_exact_hits += other.tt_exact_hits;
        self.tt_lower_hits += other.tt_lower_hits;
        self.depth_reduction_misses += other.depth_reduction_misses;
        self.depth_reduction_hits += other.depth_reduction_hits;
        self.max_ply = self.max_ply.max(other.max_ply); // set ply to max instead of adding
    }
}

pub struct CrabChessSearch {
    piece_values: HashMap<Piece, i32>,
    transposition_table: TranspositionTable,
    trans_table_depth_threshold: usize,
    search_stats: SearchStats,
    cum_search_stats: SearchStats,
    current_search_depth: usize,
}

impl CrabChessSearch {
    pub fn new() -> CrabChessSearch {
        CrabChessSearch {
            // board: Board::default(),
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
            cum_search_stats: SearchStats::default(),
            current_search_depth: 0,
        }
    }

    fn get_moves_lazily_ordered(
        &self,
        board: &Board,
        move_iterator: MoveGen,
        suggested_moves: Option<Vec<&ChessMove>>,
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
            if all_suggested_moves.contains(&&mv) {
                guess_score += 10000
            }

            // promotions are good
            if let Some(piece) = mv.get_promotion() {
                if piece == Piece::Queen {
                    guess_score += 900
                }
            }

            let move_target = mv.get_dest();
            if let Some(piece) = board.piece_on(move_target) {
                // for captures, score is enemy piece value minus a fraction of our piece value
                // capturing cheap pieces with valuable pieces is likely a bad idea
                guess_score += self.piece_values.get(&piece).unwrap()
                    - (self
                        .piece_values
                        .get(&board.piece_on(mv.get_source()).unwrap())
                        .unwrap()
                        / 2);
                // a capture that can be recaptured by an enemy is worse
                // let pawn_attacks =
                //     chess::get_pawn_attacks(move_target, board.side_to_move(), BitBoard::new(0));
                // trace!("pawn attacks: {}", pawn_attacks)
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
        let mut best_resp;

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
        // iteratively search at multiple depths

        // generate seen_positions
        let mut seen_positions: HashMap<u64, u32> = HashMap::new();
        let mut replay_game = chess::Game::new();
        // replay all actions onto a new game to create seen_positions
        for replay_action in game.actions() {
            if let chess::Action::MakeMove(replay_mv) = replay_action {
                let replay_board = replay_game.current_position();
                if replay_board.piece_on(replay_mv.get_dest()).is_some() {
                    // a capture move means we'll never see any of the previous positions again
                    seen_positions.clear();
                }
                replay_game.make_move(*replay_mv);
                *seen_positions
                    .entry(replay_game.current_position().get_hash())
                    .or_insert(0) += 1;
            }
        }

        for n in 2.min(depth)..=depth {
            self.current_search_depth = n;
            // TODO: do i need to .step_by(2)?

            // execute a top level search
            (score, chosen_move, move_order, best_resp) =
                self.top_level_search(board, n, move_order, &kill_time, &seen_positions);
            debug!(
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
                self.cum_search_stats.max_ply,
                score,
                start_time.elapsed().as_millis(),
                chosen_move,
                if best_resp == ChessMove::new(Square::A1, Square::A1, None) {
                    "".to_string()
                } else {
                    best_resp.to_string()
                }
            );
            // if we have checkmate just go for it
            // for some reason this makes it play worse
            // which doesn't seem to make sense
            // if score > 999000 {
            //     return (score, chosen_move);
            // }
            // trace!("TT Size is now {}", self.transposition_table.0.len());
            // add to cumulative search stats then clear search_stats for next time
            self.cum_search_stats += self.search_stats;
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
        debug!("{:?}", self.cum_search_stats);
        trace!("---- End search ----");
        trace!("");
        return (score, chosen_move);
    }

    fn top_level_search(
        &mut self,
        board: &Board,
        depth: usize,
        move_order: Vec<(ChessMove, i32)>,
        kill_time: &Instant,
        seen_positions: &HashMap<u64, u32>,
    ) -> (i32, ChessMove, Vec<(ChessMove, i32)>, ChessMove) {
        let mut alpha = -999999777; // this must be worse than losing
        let beta = 999999777;
        // Search for the best move using alpha-beta pruning
        // assumes depth > 0

        // Do not get top level search from transposition table!
        // this is so that we can order all the moves, making iterative deepening blazing fast

        let mut best_move = ChessMove::new(Square::A1, Square::A1, None); // default;
        let mut best_response = ChessMove::new(Square::A1, Square::A1, None);
        // let mut moves_searched = Vec::new();

        // return move_values at the end, it'll be like the new version of move_order
        let mut move_values: Vec<(ChessMove, i32)> = Vec::new();
        // debug!("Searching {} moves at depth {}", move_order.len(), depth);
        for (mv_index, (mv, mv_naive_score)) in move_order.iter().enumerate() {
            let nboard = board.make_move_new(*mv);
            let default_move = ChessMove::new(Square::A1, Square::A1, None);

            // Check for threefold repetition
            let (new_seen_positions, is_draw) =
                match check_for_draw(seen_positions.clone(), &nboard) {
                    Ok(previous_map) => (previous_map, false),
                    Err(_) => (HashMap::new(), true),
                };

            let (evaluation, this_response) = if is_draw {
                (STALEMATE_SCORE, default_move)
            } else {
                alpha -= 1; // so that if mate in 2 is 9998 then mate in 3 is 9997

                // search capture moves deeper
                let mut depth_modifier: i32 = if board.piece_on(mv.get_dest()).is_some() {
                    1
                } else {
                    0
                };

                let mut needs_full_search = true;
                let mut move_search_score = 10101010; // this is ALWAYS overwritten
                let mut best_response_mv = default_move; // this is ALWAYS overwritten
                                                         // but if i don't initialize them the compiler has a fit

                if needs_full_search {
                    if depth_modifier < 0 {
                        // we already tried a shallow search but now need to
                        // perform a full search to get a more accurate result, make sure
                        // that this great looking position really is great looking
                        depth_modifier = 0;
                        // self.search_stats.depth_reduction_misses += 1;
                    };
                    (move_search_score, best_response_mv) = self.search(
                        &nboard,
                        // &hgame,
                        depth + depth_modifier as usize - 1,
                        1,
                        -beta,
                        -alpha,
                        kill_time,
                        Some(vec![&best_response]),
                        &new_seen_positions,
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
                trace!(
                    "top level alpha set -> {} @ {}",
                    best_move.to_string(),
                    alpha
                );
                // trace!("    ->{}!! ", evaluation);
            }
        }
        trace!("");
        move_values.sort_by(|a, b| b.1.cmp(&a.1));
        // trace!("Sored move values: {:#?}", move_values);
        let mut order_moves: Vec<(ChessMove, i32)> = Vec::new();
        // order moves from best to worst
        for (i, (mv, val)) in move_values.iter().enumerate() {
            if i > 0 && alpha > -900000 && *val < -900000 {
                // ignore losing moves in future searches
                // trace!("Avoiding a losing move - {}", mv.to_string());
                continue;
            }
            // let's also prune a move if we're reasonably deep and it looks absolutely terrible
            if depth >= 4 && val + 750 < alpha {
                continue;
            }
            order_moves.push((*mv, *val))
        }

        // alpha is the evaluation of the position since this is the top level search
        if depth >= self.trans_table_depth_threshold && kill_time.elapsed() == Duration::ZERO {
            // Push exact result to transposition table since this is top level node
            self.search_stats.tt_pushed += 1;
            self.transposition_table.insert(
                board.get_hash(),
                Transposition {
                    depth,
                    ply: 0,
                    score: alpha,
                    node_type: NodeType::Exact,
                    best_move,
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
        // game: &Game,
        mut depth: usize,
        ply: usize,
        mut alpha: i32,
        beta: i32,
        kill_time: &Instant,
        suggested_moves: Option<Vec<&ChessMove>>, // try this move first
        seen_positions: &HashMap<u64, u32>,
    ) -> (i32, ChessMove) {
        // Search for the best move using alpha-beta pruning
        let default_move = ChessMove::new(Square::A1, Square::A1, None);

        if board.status() == BoardStatus::Stalemate {
            return (STALEMATE_SCORE, default_move);
        }
        if board.status() == BoardStatus::Checkmate {
            return (CHECKMATE_SCORE, default_move);
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

        if depth == 0 {
            if board.checkers() != &EMPTY {
                // get out of check first
                depth += 1
            } else {
                // assumes depth > 0 when this fn is called for the first time
                // otherwise it will return default_move
                return (
                    self.quiescence_search(board, ply + 1, alpha, beta, kill_time, seen_positions),
                    default_move,
                );
            }
        }

        // for future transposition table
        let mut this_node_type = NodeType::UpperBound;

        let movegen: MoveGen = MoveGen::new_legal(&board);
        let mut best_response: ChessMove = default_move;

        // look at every possible move from this position
        for (mv, _) in self.get_moves_lazily_ordered(board, movegen, suggested_moves) {
            let nboard = board.make_move_new(mv);
            // add this position to the map of positions we've seen before
            let (new_seen_positions, is_draw) =
                match check_for_draw(seen_positions.clone(), &nboard) {
                    Ok(m) => (m, false),
                    Err(_) => (HashMap::new(), true),
                };
            let (move_search_score, sub_response) = if is_draw {
                (STALEMATE_SCORE, default_move)
            } else {
                self.search(
                    &nboard,
                    // &hyp_game,
                    depth - 1,
                    ply + 1,
                    -beta,
                    -alpha,
                    kill_time,
                    Some(vec![&best_response]),
                    &new_seen_positions,
                )
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
                // a beta cutoff means we've failed high; this is a lower bound
                self.search_stats.tt_pushed += 1;
                self.transposition_table.insert(
                    board.get_hash(),
                    Transposition {
                        depth,
                        ply,
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
                _ => {
                    self.search_stats.tt_pushed += 1;
                    self.transposition_table.insert(
                        board.get_hash(),
                        Transposition {
                            depth,
                            ply,
                            score: best_score,
                            node_type: this_node_type,
                            best_move,
                        },
                    )
                }
            }
        }

        return (best_score, best_move);
    }

    fn quiescence_search(
        &mut self,
        board: &Board,
        ply: usize,
        mut alpha: i32,
        beta: i32,
        kill_time: &Instant,
        seen_positions: &HashMap<u64, u32>,
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
                return CHECKMATE_SCORE;
            } else if board.status() == BoardStatus::Stalemate {
                return STALEMATE_SCORE;
            }
            // no attacking moves -> evaluate the board
        }

        // if we are currently in check, this next move is forced
        // i.e. we can't just take the board evaluation instead
        let forced_move = board.checkers().popcnt() != 0;

        // if the move isn't forced the player need not make it
        let evaluation = crab_evaluate::evaluate_material(board);
        self.search_stats.boards_evaluated += 1;

        let mut best_eval = -99999999;

        if !forced_move {
            // if this move isn't forced, then we don't have to capture anything
            if evaluation >= beta {
                // return beta;
                return evaluation;
            }
            if evaluation > alpha {
                alpha = evaluation;
            }
        }

        // if we're in too deep, bail out
        if ply >= (2 + self.current_search_depth * 6).min(MAXIMUM_SEARCH_DEPTH) {
            // debug!("Bailing out at max search depth {}", ply);
            return evaluation;
        }

        for (mv, _) in self.get_moves_lazily_ordered(board, movegen, None) {
            // Evaluate this move if ANY of these conditions is true
            // (1) this move captures a piece
            // (2) this move is a promotion
            // (3) this move puts our opponent in check
            // (4) we are in check right now - this move must get us out of it
            //     and we want to consider our opponent's immediate response
            let nboard = board.make_move_new(mv);

            // bouncer
            if nboard.checkers().to_size(0) == 0 {
                // this move does not put our opponent in check
                if !forced_move
                    && nboard.piece_on(mv.get_dest()).is_none()
                    && mv.get_promotion().is_none()
                {
                    // we were not in check before this move
                    // this move does not capture a piece
                    // it is not a promotion
                    // we do not check our opponent
                    continue; // skip this move
                }
            } else {
                // this move puts our opponent in check
                // to avoid super long sequences of moves we want to break out of here sometimes
                if ply >= (self.current_search_depth * 5).min(CHECK_MV_SEARCH_DEPTH) {
                    // debug!("Ignoring checks after check move search depth {}", ply);
                    return best_eval.max(evaluation);
                }
            }
            // check draw by repetition
            let (new_seen_positions, is_draw) =
                match check_for_draw(seen_positions.clone(), &nboard) {
                    Ok(m) => (m, false),
                    Err(_) => (HashMap::new(), true),
                };

            let move_search_score = if is_draw {
                0
            } else {
                self.quiescence_search(
                    &nboard,
                    ply + 1,
                    -beta,
                    -alpha,
                    kill_time,
                    &new_seen_positions,
                )
            };
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

        return best_eval.max(evaluation);
        // return alpha;
    }
}

fn check_for_draw(
    mut seen_positions: HashMap<u64, u32>,
    board: &Board,
) -> Result<HashMap<u64, u32>, ()> {
    let value = seen_positions.entry(board.get_hash()).or_insert(0);
    *value += 1;
    if *value >= 3 {
        Err(())
    } else {
        Ok(seen_positions)
    }
}
