use crate::evaluator::EvaluatorBot2010;
use crate::speval::SinglePlayerEvaluator;
use chess::Color::{Black, White};
use chess::{Board, ChessMove, Game};
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
    // let env = Env::default().filter_or("CRAB_CHESS", "info");
    // env_logger::init_from_env(env);

    // let datetime_string = Utc::now().format("%Y-%m-%dT%H-%M-%S").to_string();
    // let log_trace_file_name = format!("crablog/crab-log-trace-{}.log", datetime_string);
    // simple_logging::log_to_file(log_trace_file_name, log::LevelFilter::Trace);

    match flexi_logger::Logger::try_with_str("trace") {
        Ok(my_logger) => {
            my_logger
                .log_to_file(
                    flexi_logger::FileSpec::default()
                        .directory("crab_logs") // create files in folder ./log_files
                        .basename("crab")
                        // .discriminant("log") // use infix in log file name
                        .suffix("log"), // use suffix .trc instead of .log
                ) // write logs to file
                .duplicate_to_stderr(flexi_logger::Duplicate::Info) // print info also to the console
                .start();
        }
        Err(_) => {}
    }

    // play_evaluator_bot_2010();
    // play_speval()
    // _debug_evaluate()

    let result = wait_for_uci();
    return result;
}

fn wait_for_uci() -> Result<(), ()> {
    let mut uci_ok: bool = false;
    let mut game = Game::new();
    let mut evaluator = EvaluatorBot2010::new();
    let move_depth = 9;
    let default_think_time: i32 = 4000;
    let mut think_time: i32 = default_think_time;
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
                think_time = default_think_time;
            }
            UciMessage::IsReady => {
                println!("readyok");
            }
            UciMessage::Position {
                startpos,
                fen,
                moves,
            } => {
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
                if let Some(tc) = time_control {
                    match tc {
                        vampirc_uci::UciTimeControl::Ponder => {
                            // pondering not implemented
                            warn!("pondering not implemented!");
                        }
                        vampirc_uci::UciTimeControl::Infinite => {
                            think_time = default_think_time;
                        }
                        vampirc_uci::UciTimeControl::TimeLeft {
                            white_time,
                            black_time,
                            white_increment,
                            black_increment,
                            moves_to_go,
                        } => {
                            let remaining_time;
                            match game.side_to_move() {
                                White => remaining_time = white_time,
                                Black => remaining_time = black_time,
                            }
                            if let Some(rem) = remaining_time {
                                // think for, at most, half of the remaining time
                                let max_think_time: i32 = (rem.num_milliseconds() / 2) as i32;
                                think_time = think_time.min(max_think_time);
                                debug!("Think time set to {}", think_time);
                            }
                        }
                        vampirc_uci::UciTimeControl::MoveTime(time_ms) => {
                            think_time = think_time.min(time_ms.num_milliseconds() as i32);
                            // think_time = time_ms.num_milliseconds() as i32;
                        }
                    }
                }

                let (value, mv) = evaluator.iterative_search_deepening(
                    &game.current_position(),
                    &game,
                    move_depth,
                    Duration::from_millis(think_time as u64),
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
    let mut bot_evaluator = EvaluatorBot2010::new();
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

    let mut evaluator: evaluator::EvaluatorBot2010 = EvaluatorBot2010::new();
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
