use chess::{Game, Board, ChessMove};
use chess::Color::{White, Black};
use std::str::FromStr;
use std::env;
use std::time::Duration;
// use std::collections::HashMap;
use std::io::stdin;
use crate::evaluator::create_evaluator;
use log::{debug, error, warn, info};
use env_logger::{self, Env};

pub mod evaluator;


fn main() {
    let env = Env::default()
        .filter_or("CRAB_CHESS", "info");
    env_logger::init_from_env(env);
    

    play_game()
    // _debug_evaluate()
}

fn play_game() {
    let args: Vec<String> = env::args().collect();
    debug!("{:?}", args);
    // human color provided by first argument, otherwise AI vs AI
    let player_color = if args.len() > 1 {
        match args[1].to_lowercase().as_str() {
            "white" => White.to_index(),
            "black" => Black.to_index(),
            _ => {
                error!("Provided color not recognized, starting AI vs AI");
                3
            }
        }
    } else {
        warn!("Provide the human's color as the first argument or it will be AI vs AI!");
        3
    };
    // use fen if provided as an argument, otherwise new game
    let mut game = if args.len() > 2 {
        info!("Creating game from FEN");
        Game::from_str(args[2].as_str()).expect("Valid FEN")
    } else {
        Game::new()
    };

    let mut evaluator = create_evaluator();
    let mut board: Board;
    loop {
        board = game.current_position();
        debug!("Board hash: {}; fen: {}", board.get_hash(), board.to_string());
        // println!("{:?}", game.actions());
        let to_move = board.side_to_move();
        info!("Move {} - {:?} to move", game.actions().len(), to_move);
        if !game.result().is_none() {
            info!("Game Over");
            break
        } else if game.can_declare_draw() {
            info!("Draw");
            break
        }
        if player_color != to_move.to_index() {
            // AI's turn
            let (value, mv) = evaluator.iterative_search_deepening(&board, 7, Duration::new(10, 0));
            info!("{:?} AI Move: {} @ {}", to_move, mv.to_string(), value);
            // TODO: make_move will return default_move if it can't find a non-losing move
            // if it does so, may as well resign
            // but rn it keeps infinitely looping
            game.make_move(mv);
        } else {
            // Human's turn
            info!("Enter your move");
            let mut input = String::new();
            stdin().read_line(&mut input).expect("error: unable to read user input");
            if input == String::from("new game") {
                game = Game::new()
            }
            let outcome = match ChessMove::from_san(&board, input.as_str()) {
                Ok(mv) => game.make_move(mv),
                Err(er) => {
                    warn!("{:?}", er);
                    false
                }
            };
            if !outcome {
                error!("Move failed")
            }
            
        }
    }
}

fn _debug_evaluate() {
    let args: Vec<String> = env::args().collect();
    let board = if args.len() > 1 {
        Board::from_str(args[1].as_str()).expect("Valid FEN")
    } else {
        Board::default()
    };
    let mut evaluator = create_evaluator();
    let (value, mv) = evaluator.iterative_search_deepening(&board, 6, Duration::new(8, 0));
    println!("{} @ {}", mv.to_string(), value);
}