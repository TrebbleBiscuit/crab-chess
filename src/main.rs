use crate::evaluator::{create_evaluator, EvaluatorBot2010};
use crate::speval::SinglePlayerEvaluator;
use chess::Color::{Black, White};
use chess::{Board, ChessMove, Game};
use env_logger::{self, Env};
use log::{debug, error, info, warn};
use std::env;
use std::io::stdin;
use std::io::{self, BufRead};
use std::str::FromStr;
use std::time::Duration;
use vampirc_uci::{parse_one, UciMessage};

pub mod evaluator;
pub mod speval;

fn main() -> Result<(), ()> {
    let env = Env::default().filter_or("CRAB_CHESS", "info");
    env_logger::init_from_env(env);

    // play_evaluator_bot_2010();
    // play_speval()
    // _debug_evaluate()

    let result = wait_for_uci();
    return result;
}

fn wait_for_uci() -> Result<(), ()> {
    let mut uci_ok: bool = false;
    let mut game = Game::new();
    let mut evaluator = create_evaluator();
    let move_depth = 5;
    for line in io::stdin().lock().lines() {
        let msg: UciMessage = parse_one(&line.unwrap());
        info!("Received message: {}", msg);
        match msg {
            UciMessage::Uci => {
                // Initialize the UCI mode of the chess engine.
                uci_ok = true;
                println!("uciok")
            }
            UciMessage::UciNewGame => {
                game = Game::new();
            }
            UciMessage::IsReady => {
                println!("readyok");
            }
            UciMessage::Position {
                startpos,
                fen,
                moves,
            } => {
                // Set up the starting position in the engine and play the moves e2-e4 and e7-e5
                if startpos {
                    game = Game::new();
                }
                if let Some(input_fen) = fen {
                    if let Ok(new_game) = Game::from_str(input_fen.as_str()) {
                        game = new_game;
                    }
                }
                for each_move in moves.iter() {
                    game.make_move(*each_move);
                }
            }
            UciMessage::Go {
                time_control,
                search_control,
            } => {
                // if let Some(tc) = time_control {
                //     ...
                // }

                let (value, mv) = evaluator.iterative_search_deepening(
                    &game.current_position(),
                    &game,
                    move_depth,
                    Duration::new(10, 0),
                );
                println!("bestmove {}", mv.to_string());
                game.make_move(mv);

                // singleplayer
                // let mv = evaluator.top_level_search(&game.current_position(), move_depth);
                // game.make_move(mv);
                // println!("bestmove {}", mv.to_string());
            }
            UciMessage::Quit => return Ok(()),
            _ => {
                // info!("{:?}", msg)
            }
        }
    }
    Ok(())
}

fn play_speval() {
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
    let move_depth: usize = if args.len() > 3 {
        args[3].parse::<usize>().unwrap()
    } else {
        5
    };
    // use fen if provided as an argument, otherwise new game
    let mut game = if args.len() > 2 {
        info!("Creating game from FEN");
        Game::from_str(args[2].as_str()).expect("Valid FEN")
    } else {
        Game::new()
    };

    let mut board: Board;
    let sp_evaluator = SinglePlayerEvaluator::new();
    let mut bot_evaluator = create_evaluator();
    loop {
        board = game.current_position();
        debug!(
            "Board hash: {}; fen: {}",
            board.get_hash(),
            board.to_string()
        );
        let to_move = board.side_to_move();
        info!("Move {} - {:?} to move", game.actions().len() + 1, to_move);
        if !game.result().is_none() {
            info!("Game Over");
            break;
        } else if game.can_declare_draw() {
            info!("Draw");
            break;
        }
        // pick move
        // match to_move {
        //     White => {
        //         let mv = sp_evaluator.top_level_search(&board, move_depth);
        //         info!("{:?} SP Move: {}", to_move, mv.to_string());
        //         game.make_move(mv);
        //     }
        //     Black => {
        //         let (value, mv) = bot_evaluator.iterative_search_deepening(
        //             &board,
        //             &game,
        //             move_depth,
        //             Duration::new(10, 0),
        //         );
        //         info!("{:?} BOT Move: {} @ {}", to_move, mv.to_string(), value);
        //         game.make_move(mv);
        //     }
        // }
        if player_color != to_move.to_index() {
            // AI's turn
            let mv = sp_evaluator.top_level_search(&board, move_depth);
            info!("{:?} AI Move: {}", to_move, mv.to_string());
            game.make_move(mv);
        } else {
            // Human's turn
            info!("Enter your move");
            let mut input = String::new();
            stdin()
                .read_line(&mut input)
                .expect("error: unable to read user input");
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

fn play_evaluator_bot_2010() {
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

    let mut evaluator: evaluator::EvaluatorBot2010 = create_evaluator();
    let mut board: Board;
    let move_depth: usize = if args.len() > 3 {
        args[3].parse::<usize>().unwrap()
    } else {
        6
    };
    loop {
        board = game.current_position();
        debug!(
            "Board hash: {}; fen: {}",
            board.get_hash(),
            board.to_string()
        );
        // println!("{:?}", game.actions());
        let to_move = board.side_to_move();
        info!("Move {} - {:?} to move", game.actions().len() + 1, to_move);
        if !game.result().is_none() {
            info!("Game Over");
            break;
        } else if game.can_declare_draw() {
            info!("Draw");
            break;
        }
        if player_color != to_move.to_index() {
            // AI's turn
            let (value, mv) = evaluator.iterative_search_deepening(
                &board,
                &game,
                move_depth,
                Duration::new(1, 0),
            );
            info!("{:?} AI Move: {} @ {}", to_move, mv.to_string(), value);
            game.make_move(mv);
        } else {
            // Human's turn
            info!("Enter your move");
            let mut input = String::new();
            stdin()
                .read_line(&mut input)
                .expect("error: unable to read user input");
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
