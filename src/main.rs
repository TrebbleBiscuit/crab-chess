use chess::{MoveGen, Game, Board, ALL_PIECES, Piece, BoardStatus, ChessMove, Square, Color};
use chess::Piece::{Pawn, Knight, Bishop, Rook, Queen, King};
// use chess::EMPTY;
use chess::Color::{White, Black};
// use std::collections::HashMap;
use std::str::FromStr;
use std::env;
use std::time::Instant;
use std::cmp::Ordering;

// Piece-Square Tables
const PAWN_PST: [i32; 64] = [   0,   0,   0,   0,   0,   0,   0,   0,
            78,  83,  86,  73, 102,  82,  85,  90,
             7,  29,  21,  44,  40,  31,  44,   7,
           -17,  16,  -2,  15,  14,   0,  15, -13,
           -26,   3,  10,   9,   6,   1,   0, -23,
           -22,   9,   5, -11, -10,  -2,   3, -19,
           -31,   8,  -7, -37, -36, -14,   3, -31,
             0,   0,   0,   0,   0,   0,   0,   0
];

const KNIGHT_PST: [i32; 64] = [ -66, -53, -75, -75, -10, -55, -58, -70,
            -3,  -6, 100, -36,   4,  62,  -4, -14,
            10,  67,   1,  74,  73,  27,  62,  -2,
            24,  24,  45,  37,  33,  41,  25,  17,
            -1,   5,  31,  21,  22,  35,   2,   0,
           -18,  10,  13,  22,  18,  15,  11, -14,
           -23, -15,   2,   0,   2,   0, -23, -20,
           -74, -23, -26, -24, -19, -35, -22, -69
];

const BISHOP_PST: [i32; 64] = [ -59, -78, -82, -76, -23,-107, -37, -50,
           -11,  20,  35, -42, -39,  31,   2, -22,
            -9,  39, -32,  41,  52, -10,  28, -14,
            25,  17,  20,  34,  26,  25,  15,  10,
            13,  10,  17,  23,  17,  16,   0,   7,
            14,  25,  24,  15,   8,  25,  20,  15,
            19,  20,  11,   6,   7,   6,  20,  16,
            -7,   2, -15, -12, -14, -15, -10, -10
];

const ROOK_PST: [i32; 64] = [  35,  29,  33,   4,  37,  33,  56,  50,
            55,  29,  56,  67,  55,  62,  34,  60,
            19,  35,  28,  33,  45,  27,  25,  15,
             0,   5,  16,  13,  18,  -4,  -9,  -6,
           -28, -35, -16, -21, -13, -29, -46, -30,
           -42, -28, -42, -25, -25, -35, -26, -46,
           -53, -38, -31, -26, -29, -43, -44, -53,
           -30, -24, -18,   5,  -2, -18, -31, -32
];

const QUEEN_PST: [i32; 64] = [   6,   1,  -8,-104,  69,  24,  88,  26,
            14,  32,  60, -10,  20,  76,  57,  24,
            -2,  43,  32,  60,  72,  63,  43,   2,
             1, -16,  22,  17,  25,  20, -13,  -6,
           -14, -15,  -2,  -5,  -1, -10, -20, -22,
           -30,  -6, -13, -11, -16, -11, -16, -27,
           -36, -18,   0, -19, -15, -15, -21, -38,
           -39, -30, -31, -13, -31, -36, -34, -42
];

const KING_PST: [i32; 64] = [   4,  54,  47, -99, -99,  60,  83, -62,
           -32,  10,  55,  56,  56,  55,  10,   3,
           -62,  12, -57,  44, -67,  28,  37, -31,
           -55,  50,  11,  -4, -19,  13,   0, -49,
           -55, -43, -52, -28, -51, -47,  -8, -50,
           -47, -42, -43, -79, -64, -32, -29, -32,
            -4,   3, -14, -50, -57, -18,  13,   4,
            17,  30,  -3, -14,   6,  -1,  40,  18
];

// TODO: arrays are not slower so just use them instead so we can do this
// const KING_WHITE_PST: [i32; 64] = KING_PST.iter().copied().rev().collect();



// scores.insert(String::from("Blue"), 10);
// scores.insert(String::from("Yellow"), 50);

// fn print_type_of<T>(_: &T) {
//     println!("{}", std::any::type_name::<T>())
// }

// fn piece_value(piece:Piece) -> i32 {
//     match piece {
//         Pawn => 100,
//         Knight => 320,
//         Bishop => 330,
//         Rook => 500,
//         Queen => 900,
//         King => 10000
//     }
// }

fn evaluate_material(board: &Board) -> i32{
    let side = board.side_to_move();
    // Returns a positive value if the player whose turn it is is winning 
    fn value_at_square(piece: Piece, square: usize, side: Color) -> i32 {
        // SUPER ABSURDLY SLOW
        // let index: usize;
        // match side {
        //     White => index = 63-square,
        //     Black => index = square
        // }
        match piece {
            Pawn => PAWN_PST[index] + 100,
            Knight => KNIGHT_PST[index] + 320,
            Bishop => BISHOP_PST[index] + 330,
            Rook => ROOK_PST[index] + 500,
            Queen => QUEEN_PST[index] + 900,
            King => KING_PST[index]
        }
    }

    let mut total_score: i32 = 0;

    for piece in ALL_PIECES {
        for square in *board.pieces(piece) {
            let color = board.color_on(square);
            let value = value_at_square(piece, square.to_index(), side);
            match color {
                Some(White) => total_score += value,
                Some(Black) => total_score -= value,
                None => panic!("no piece here"),
            }
        }
    }
    match board.side_to_move() {
        White => return total_score,
        Black => return -total_score
    }
}

fn search(board: &Board, depth: usize, mut alpha: i32, beta: i32) -> (i32, ChessMove){
    // Search for the best move using alpha-beta pruning
    let default_move = ChessMove::new(Square::A1, Square::A1, None);
    if depth == 0 {
        // assumes depth > 0 when this fn is called for the first time
        // otherwise it will return default_move
        // return (evaluate_material(board), default_move)
        return (search_only_captures(board, alpha, beta), default_move)
    }

    let mut movegen: MoveGen = MoveGen::new_legal(&board);
    if movegen.len() == 0 {
        if board.status() == BoardStatus::Checkmate {
            return (i32::from(-999999), default_move)
        } else {
            return (i32::from(0), default_move)
        }
    }
    let mut best_move: ChessMove = default_move;
    // let mut moves_searched = Vec::new();
    // let mut move_values = Vec::new();
    for mv in &mut movegen {
        let nboard = board.make_move_new(mv);
        let (move_search_score, _next_move) = search(&nboard, depth-1, -beta, -alpha);
        // if depth == 4 {
        //     moves_searched.push(mv);
        //     move_values.push(move_search_score);
        // }
        let evaluation = -move_search_score;
        if evaluation >= beta { 
            return (beta, default_move);
        }
        if evaluation > alpha {
            alpha = evaluation;
            best_move = mv;
        }
    }
    // if depth == 4 {
    //     let index_of_max: Option<usize> = move_values
    //         .iter()
    //         .enumerate()
    //         .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
    //         .map(|(index, _)| index);
    //     println!("{:?}, {:?}, {:?}", index_of_max, moves_searched, move_values);
    // }
    return (alpha, best_move)

}

// fn list_moves_in_order(moves: MoveGen) {
//     for mv in moves {
//         let mut est_value: i32 = 0;
//         match mv.get_promotion() {
//             Some(piece) => est_value += piece_value(piece),
//             _ => (),
//         }
//     }
// }



fn search_only_captures(board: &Board, mut alpha: i32, beta: i32) -> i32{
    // Search for the best move using alpha-beta pruning
    // This time, only look for captures at infinite depth
    let evaluation = evaluate_material(board);
    if evaluation >= beta {
        return beta;
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
            return i32::from(-999999)
        } else if board.status() == BoardStatus::Stalemate {
            return i32::from(0)
        }
        // no attacking moves does not mean stalemate here
    }
    for mv in &mut movegen {
        let nboard = board.make_move_new(mv);
        let move_search_score = search_only_captures(&nboard, -beta, -alpha);
        let evaluation = -move_search_score;
        if evaluation >= beta { 
            return beta;
        }
        if evaluation > alpha {
            alpha = evaluation;
        }
    }
    return alpha

}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    
    // let piece_values: HashMap<&Piece, i32> = [(&Pawn, 100), (&Knight, 300), (&Bishop, 300), (&Rook, 300), (&Queen, 300), (&King, 300)].iter().cloned().collect();
    println!("Hello, world!");
    // create a game
    // let mut board: Board;
    let mut game: Game = Game::new();
    // let mut game: Game = Game::from_str("rnbqkb1r/ppp2ppp/4pn2/8/3P4/2Np1N2/PP2PPPP/R1B1KB1R w KQkq - 0 6").expect("Valid FEN");

    let mut board = if args.len() > 1 {
        Board::from_str(&args[1]).expect("Valid FEN")
    } else {
        game.current_position()
    };
    println!("{:?} to move", board.side_to_move());
    //let mut board = game.current_position();
    println!("Evaluation: {:?}", evaluate_material(&board));
    let start_time = Instant::now();
    let (value, mv) = search(&board, 2, -9999999, 9999999);
    let elapsed_time = start_time.elapsed();
    println!("Search took {} seconds", (elapsed_time.as_secs()));
    println!("Search: {:?} @ {}", mv, value);
    
    // loop {
    //     board = game.current_position();
    //     if !game.result().is_none() {
    //         println!("Game Over");
    //         break
    //     } else if game.can_declare_draw() {
    //         println!("Draw");
    //         break
    //     }
    //     let mut movegen: MoveGen = MoveGen::new_legal(&board);
    //     let targets = board.color_combined(!board.side_to_move());
    //     // look for targets first
    //     movegen.set_iterator_mask(*targets);
    //     if movegen.len() == 0 {
    //         // if there are no targets to capture, make a non-capture move instead
    //         movegen.set_iterator_mask(!EMPTY);
    //     }
    //     println!("{} possible moves", movegen.len());
    //     for mv in &mut movegen {
    //         println!("Making move: {}", mv);
    //         game.make_move(mv);
    //         break

    //     }
    // }
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
