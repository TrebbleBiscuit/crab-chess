use crate::evaluator::EvaluatorBot2010;
use crate::speval::SinglePlayerEvaluator;
use chess::Color::{Black, White};
use chess::{Board, ChessMove, Game};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::env;
use std::io::stdin;
use std::io::{self, BufRead};
use std::str::FromStr;
use std::time::Duration;
use vampirc_uci::{parse_one, UciMessage};
pub mod evaluator;
pub mod precomputed;
pub mod speval;
use serde::{Deserialize, Serialize};

// #[derive(Serialize, Deserialize, Debug)]
// struct BookEntry {
//     fen: String,
//     move_map: std::collections::HashMap<String, i32>,
// }

fn program_file_name() -> String {
    match env::current_exe()
        .ok()
        .as_ref()
        .map(std::path::Path::new)
        .and_then(std::path::Path::file_name)
        .and_then(std::ffi::OsStr::to_str)
        .map(String::from)
    {
        Some(name) => name,
        None => "crab".to_string(),
    }
}

fn main() -> Result<(), ()> {
    let args: Vec<String> = env::args().collect();
    // human color provided by first argument, otherwise AI vs AI
    let (log_enabled, log_level) = if args.len() > 1 {
        match args[1].to_lowercase().as_str() {
            "--quiet" | "-q" => (false, "debug"),
            "--verbose" | "-v" => (true, "trace"),
            _ => (true, "debug"),
        }
    } else {
        (true, "debug")
    };

    // let env = Env::default().filter_or("CRAB_CHESS", "info");
    // env_logger::init_from_env(env);

    // let datetime_string = Utc::now().format("%Y-%m-%dT%H-%M-%S").to_string();
    // let log_trace_file_name = format!("crablog/crab-log-trace-{}.log", datetime_string);
    // simple_logging::log_to_file(log_trace_file_name, log::LevelFilter::Trace);

    // println!("calling book test...");
    // book_test();
    // return Ok(());

    if log_enabled {
        // log to file and also to stdout
        if let Ok(my_logger) = flexi_logger::Logger::try_with_str(log_level) {
            match my_logger
                .log_to_file(
                    flexi_logger::FileSpec::default()
                        .directory("crab_logs") // create files in this folder
                        .basename(program_file_name())
                        // .discriminant("log") // use in log file name
                        .suffix("log"),
                )
                .duplicate_to_stderr(flexi_logger::Duplicate::Info)
                .start()
            {
                Ok(x) => x,
                Err(y) => {
                    error!("{:?}", y);
                    panic!("couldn't set up logger! try --quiet")
                }
            };
        }
    }

    let mut input;
    loop {
        input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim(); // Remove the trailing newline character

        match input {
            "uci" => {
                println!("uciok");
                return wait_for_uci();
            }
            "crab" => {
                bot_vs_bot();
            }
            _ => {
                println!("Unknown command. Try `uci`");
            }
        }
    }

    // bot_vs_bot();
    // play_evaluator_bot_2010();
    // Ok(())
    // play_speval()
    // _debug_evaluate()

    // let result = wait_for_uci();
    return Ok(());
}

fn wait_for_uci() -> Result<(), ()> {
    let mut game = Game::new();
    let mut evaluator = EvaluatorBot2010::new();
    let move_depth = 9;
    let default_think_time: i32 = 4000;
    let mut think_time: i32 = default_think_time;
    for line in io::stdin().lock().lines() {
        let msg: UciMessage = parse_one(&line.unwrap());
        info!("Received message: {}", msg.to_string());
        // if msg.to_string() == "dumptt" {
        //     evaluator.dumptt();
        // }
        match msg {
            UciMessage::Uci => {
                // Initialize the UCI mode of the chess engine.
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
                // for mv in game.actions() {}
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
                            let my_increment;
                            match game.side_to_move() {
                                White => {
                                    remaining_time = white_time;
                                    my_increment = white_increment;
                                }
                                Black => {
                                    remaining_time = black_time;
                                    my_increment = black_increment;
                                }
                            }
                            if let Some(rem) = remaining_time {
                                // how much longer will this game last? 10-70 moves
                                let est_turns_remaining = 70 - game.actions().len().min(60);
                                // this is how much time we'll burn from our clock
                                let mut burn_time =
                                    rem.num_milliseconds() as i32 / est_turns_remaining as i32;
                                if let Some(inc) = my_increment {
                                    // we'll also burn most of our increment time
                                    burn_time += (inc.num_milliseconds() as f64 * 0.8) as i32;
                                }

                                // think for, at most, half of the remaining time
                                let max_think_time: i32 = (rem.num_milliseconds() / 2) as i32;
                                think_time = burn_time.min(max_think_time);
                                debug!("Think time set to {}", think_time);
                            }
                        }
                        vampirc_uci::UciTimeControl::MoveTime(time_ms) => {
                            // think_time = think_time.min(time_ms.num_milliseconds() as i32);
                            think_time = time_ms.num_milliseconds() as i32;
                        }
                    }
                }

                let (_value, mv) = evaluator.iterative_search_deepening(
                    &game.current_position(),
                    &game,
                    move_depth,
                    Duration::from_millis(think_time as u64),
                );
                println!("bestmove {mv}");
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
        if game.result().is_some() {
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
            let mv = sp_evaluator.top_level_search(&board, move_depth, Duration::from_secs(1));
            info!("{:?} AI Move: {}", to_move, mv);
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

fn bot_vs_bot() {
    // let mut white_evaluator: evaluator::EvaluatorBot2010 = EvaluatorBot2010::new();
    let mut white_evaluator = SinglePlayerEvaluator::new();
    let mut black_evaluator: evaluator::EvaluatorBot2010 = EvaluatorBot2010::new();
    let mut board: Board;
    let move_depth: usize = 12;
    let mut game = Game::new();
    let mut move_duration = Duration::from_millis(300);
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
        match to_move {
            White => {
                let mv = white_evaluator.top_level_search(&board, 5, move_duration);
                info!("{:?} AI Move: {}", to_move, mv);
                game.make_move(mv);
            }
            Black => {
                let (value, mv) = black_evaluator.iterative_search_deepening(
                    &board,
                    &game,
                    move_depth,
                    move_duration,
                );
                info!("{:?} AI Move: {} @ {}", to_move, mv, value);
                game.make_move(mv);
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
    let mut game: Game = if args.len() > 2 {
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
        12
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
            info!("{:?} AI Move: {} @ {}", to_move, mv, value);
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
