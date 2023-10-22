use chess::Color::{Black, White};
use chess::Piece::{Bishop, King, Knight, Pawn, Queen, Rook};
use chess::{Board, BoardStatus, ChessMove, Game, MoveGen, Piece, Square, ALL_PIECES};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info, trace};
use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::time::{Duration, Instant};
pub struct SinglePlayerEvaluator {}

impl SinglePlayerEvaluator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn top_level_search(&self, board: &Board, depth: usize) -> ChessMove {
        // search at increasing depths up to the maximum or until we find mate
        for current_depth in 0..=depth {
            if let Some(path_to_mate) = self.search(board, current_depth) {
                info!("Found mate in {}", current_depth);
                return path_to_mate;
            }
        }

        // couldn't find a path to checkmate at this
        info!("Couldn't find a path to checkmate! Selecting a random move.");
        let move_gen = MoveGen::new_legal(&board);
        return move_gen
            .into_iter()
            .choose(&mut rand::thread_rng())
            .unwrap();
        // println!(
        //     "{:?}",
        //     move_gen.choose(&mut rand::thread_rng())
        // );
    }

    fn search(&self, board: &Board, depth: usize) -> Option<ChessMove> {
        if depth == 0 {
            return None;
        }
        // it's our turn, look through all legal moves
        let legal_moves = MoveGen::new_legal(&board);
        let num_legal_moves = legal_moves.len();
        // in random order to spice it up
        let mut legal_moves_vec = legal_moves
            .into_iter()
            .choose_multiple(&mut rand::thread_rng(), num_legal_moves);
        legal_moves_vec.shuffle(&mut rand::thread_rng());
        for possible_move in legal_moves_vec {
            // make a new board and make a move
            let possible_board = board.make_move_new(possible_move);
            if possible_board.status() == BoardStatus::Checkmate {
                // this move checkmates our opponent!
                return Some(possible_move);
            }
            // didn't checkmate our opponent right away and it's their turn
            // pretend it's our turn again and then look deeper
            let new_board = match possible_board.null_move() {
                Some(fantasy_board) => fantasy_board,
                None => {
                    // the opponent is in check, and thus can not skip their turn
                    // instead let's have them make a totally random legal move
                    possible_board.make_move_new(
                        MoveGen::new_legal(&possible_board)
                            .into_iter()
                            .choose(&mut rand::thread_rng())
                            .unwrap(),
                    )
                }
            };
            let future_result = self.search(&new_board, depth - 1);
            if let Some(_) = future_result {
                // found a checkmate in our deeper search
                return Some(possible_move);
            }
        }
        // couldn't find a path to checkmate
        return None;
    }
}
