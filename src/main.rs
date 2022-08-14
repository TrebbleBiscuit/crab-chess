use chess::{Game, Board, ChessMove};
use chess::Color::{White, Black};
use std::str::FromStr;
use std::env;
use std::time::Duration;
// use std::collections::HashMap;
use std::io::{stdin, stdout, Read, Write};
use crate::evaluator::create_evaluator;
use log::{debug, error, warn, info};
use env_logger;

pub mod evaluator;

fn pause() {
    let mut stdout = stdout();
    stdout.write(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
}


fn main() {

    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let mut board = if args.len() > 1 {
        Board::from_str(args[1].as_str()).expect("Valid FEN")
    } else {
        Board::default()
    };
    let evaluator = create_evaluator();
    let (value, mv) = evaluator.iterative_search_deepening(&board, 7, Duration::new(10, 0));
    println!("{} @ {}", mv.to_string(), value);
}

fn play_game() {
    let args: Vec<String> = env::args().collect();
    debug!("{:?}", args);
    // create a game
    // let mut board: Board;
    // let mut game: Game = Game::from_str("rnbqkb1r/ppp2ppp/4pn2/8/3P4/2Np1N2/PP2PPPP/R1B1KB1R w KQkq - 0 6").expect("Valid FEN");

    // set up the board
    let mut game = if args.len() > 1 {
        Game::from_str(args[1].as_str()).expect("Valid FEN")
    } else {
        Game::new()
    };

    let evaluator = create_evaluator();

    //let mut board = game.current_position();
    // println!("Evaluation: {:?}", evaluator.evaluate_material(&board));
    // let start_time = Instant::now();
    // let (value, mv) = evaluator.iterative_search_deepening(&board, 6, Duration::new(40, 0));
    // let elapsed_time = start_time.elapsed();
    // println!("Search took {} seconds", (elapsed_time.as_secs()));
    // println!("Search: {:?} @ {}", mv, value);
    let mut board: Board;
    loop {
        board = game.current_position();
        info!("Board hash: {}; fen: {}", board.get_hash(), board.to_string());
        // println!("{:?}", game.actions());
        let to_move = board.side_to_move();
        info!("{:?} to move", to_move);
        if !game.result().is_none() {
            info!("Game Over");
            break
        } else if game.can_declare_draw() {
            info!("Draw");
            break
        }
        if to_move == White {
            let (value, mv) = evaluator.iterative_search_deepening(&board, 7, Duration::new(18, 0));
            println!("Search: {} @ {}", mv.to_string(), value);
            game.make_move(mv);
        } else {
            let mut input = String::new();
            stdin().read_line(&mut input).expect("error: unable to read user input");
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
        pause();
    }
}

// fn example() {
//     let board = Board::default();
//     // create an iterable
//     let mut movegen = MoveGen::new_legal(&board);

//     // make sure .len() works.
//     assert_eq!(movegen.len(), 20); // the .len() function does *not* consume the iterator

//     // lets iterate over targets.
//     let targets = board.color_combined(!board.side_to_move());
//     movegen.set_iterator_mask(*targets);

//     // count the number of targets
//     let mut count = 0;
//     for mv in &mut movegen {
//         count += 1;
//         println!("Capture move: {}", mv)
//         // This move captures one of my opponents pieces (with the exception of en passant)
//     }

//     // now, iterate over the rest of the moves
//     movegen.set_iterator_mask(!EMPTY);
//     for mv in &mut movegen {
//         count += 1;
//         println!("Non-capture move: {}", mv)
//         // This move does not capture anything
//     }

//     // make sure it works
//     assert_eq!(count, 20);
// }


// fn evaluate_board() {}
