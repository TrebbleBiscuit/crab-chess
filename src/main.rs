use chess::{MoveGen, Game, Board};
use chess::EMPTY;




fn main() {
    println!("Hello, world!");
    // create a game
    let mut board: Board;
    let mut game: Game = Game::new();
    loop {
        board = game.current_position();
        if !game.result().is_none() {
            println!("Game Over");
            break
        } else if game.can_declare_draw() {
            println!("Draw");
            break
        }
        let mut movegen: MoveGen = MoveGen::new_legal(&board);
        let targets = board.color_combined(!board.side_to_move());
        // look for targets first
        movegen.set_iterator_mask(*targets);
        if movegen.len() == 0 {
            // if there are no targets to capture, make a non-capture move instead
            movegen.set_iterator_mask(!EMPTY);
        }
        println!("{} possible moves", movegen.len());
        for mv in &mut movegen {
            println!("Making move: {}", mv);
            game.make_move(mv);
            break

        }
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
