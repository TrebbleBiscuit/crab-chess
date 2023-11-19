use crate::precomputed;
use chess::Color::{self, Black, White};
use chess::Piece::{Bishop, King, Knight, Pawn, Queen, Rook};
use chess::{BitBoard, Board, Piece, Square, ALL_PIECES};

// these are used for determining whether a pawn is supported or isolated
const FILE_MASKS: [u64; 8] = [
    0x101010101010101,
    0x202020202020202,
    0x404040404040404,
    0x808080808080808,
    0x1010101010101010,
    0x2020202020202020,
    0x4040404040404040,
    0x8080808080808080,
];

// passed pawn bonus depends on number of squares to promotion
const PASSED_PAWN_BONUS: [i32; 8] = [0, 150, 90, 50, 20, 15, 15, 15];

const DISTANCE_FROM_CENTER: [i32; 64] = [
    6, 5, 4, 3, 3, 4, 5, 6, 5, 4, 3, 2, 2, 3, 4, 5, 4, 3, 2, 1, 1, 2, 3, 4, 3, 2, 1, 0, 0, 1, 2, 3,
    3, 2, 1, 0, 0, 1, 2, 3, 4, 3, 2, 1, 1, 2, 3, 4, 5, 4, 3, 2, 2, 3, 4, 5, 6, 5, 4, 3, 3, 4, 5, 6,
];

const PAWN_PST_ENDGAME: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 80, 80, 80, 80, 80, 80, 80, 80, 50, 50, 50, 50, 50, 50, 50, 50, 30, 30,
    30, 30, 30, 30, 30, 30, 20, 20, 20, 20, 20, 20, 20, 20, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
    10, 10, 10, 10, 10, 10, 0, 0, 0, 0, 0, 0, 0, 0,
];
const PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 78, 83, 86, 73, 102, 82, 85, 90, 7, 29, 21, 44, 40, 31, 44, 7, -18, 16,
    -2, 15, 14, 0, 15, -13, -26, 3, 10, 9, 6, 1, 0, -23, -22, 9, 5, -11, -10, -2, 3, -19, -31, 8,
    -7, -37, -36, -14, 3, -31, 0, 0, 0, 0, 0, 0, 0, 0,
];
const KNIGHT_PST: [i32; 64] = [
    -66, -53, -75, -75, -10, -55, -58, -70, -3, -6, 100, -36, 4, 62, -4, -14, 10, 67, 1, 74, 73,
    27, 62, -2, 24, 24, 45, 37, 33, 41, 25, 17, -1, 5, 31, 21, 22, 35, 2, 0, -18, 10, 13, 22, 18,
    15, 11, -14, -23, -15, 2, 0, 2, 0, -23, -20, -74, -23, -26, -24, -19, -35, -22, -69,
];
const BISHOP_PST: [i32; 64] = [
    -59, -78, -82, -76, -23, -107, -37, -50, -11, 20, 35, -42, -39, 31, 2, -22, -9, 39, -32, 41,
    52, -10, 28, -14, 25, 17, 20, 34, 26, 25, 15, 10, 13, 10, 17, 23, 17, 16, 0, 7, 14, 25, 24, 15,
    8, 25, 20, 15, 19, 20, 11, 6, 7, 6, 20, 16, -7, 2, -15, -12, -14, -15, -10, -10,
];
const ROOK_PST: [i32; 64] = [
    35, 29, 33, 4, 37, 33, 56, 50, 55, 29, 56, 67, 55, 62, 34, 60, 19, 35, 28, 33, 45, 27, 25, 15,
    0, 5, 16, 13, 18, -4, -9, -6, -28, -35, -16, -21, -13, -29, -46, -30, -42, -28, -42, -25, -25,
    -35, -26, -46, -53, -38, -31, -26, -29, -43, -44, -53, -30, -24, -18, 5, -2, -18, -31, -32,
];
const QUEEN_PST: [i32; 64] = [
    6, 1, -8, -104, 69, 24, 88, 26, 14, 32, 60, -10, 20, 76, 57, 24, -2, 43, 32, 60, 72, 63, 43, 2,
    1, -16, 22, 17, 25, 20, -13, -6, -14, -15, -2, -5, -1, -10, -20, -22, -30, -6, -13, -11, -16,
    -11, -16, -27, -36, -18, 0, -19, -15, -15, -21, -38, -39, -30, -31, -13, -31, -36, -34, -42,
];
const KING_PST: [i32; 64] = [
    4, 54, 47, -99, -99, 60, 83, -62, -32, 10, 55, 56, 56, 55, 10, 3, -62, 12, -57, 44, -67, 28,
    37, -31, -55, 50, 11, -4, -19, 13, 0, -49, -55, -43, -52, -28, -51, -47, -8, -50, -47, -42,
    -43, -79, -64, -32, -29, -32, -4, 3, -14, -50, -57, -18, 13, 4, 17, 30, -3, -14, 6, -1, 40, 18,
];

pub fn evaluate_material(board: &Board) -> i32 {
    // Returns a positive value if the player whose turn it is is winning
    let mut total_score: i32 = 0;
    // we'll use this bitboard to calculate pawn bonus value
    let white_pieces = board.color_combined(Color::White);
    let black_pieces = board.color_combined(Color::Black);
    let white_pawns = white_pieces & board.pieces(Piece::Pawn);
    let black_pawns = black_pieces & board.pieces(Piece::Pawn);
    let white_major_pieces = white_pieces & !white_pawns;
    let black_major_pieces = black_pieces & !black_pawns;
    let all_major_pieces = white_major_pieces | black_major_pieces;
    let endgame_factor: u32 = 10 - all_major_pieces.popcnt().clamp(4, 10); // 0 to 6

    for piece in ALL_PIECES {
        for square in *board.pieces(piece) {
            let index = square.to_index();
            if board.color_on(square) == Some(White) {
                total_score += match piece {
                    Pawn => {
                        interpolated_pawn_pst(endgame_factor, 63 - index)
                            + 100
                            + pawn_bonus_value(square, White, &black_pawns, &white_pawns)
                    }
                    Knight => KNIGHT_PST[63 - index] + 320,
                    Bishop => BISHOP_PST[63 - index] + 330,
                    Rook => ROOK_PST[63 - index] + 500,
                    Queen => QUEEN_PST[63 - index] + 900,
                    King => {
                        evaluate_king_position(63 - index, board, square, White, endgame_factor)
                    }
                };
            } else {
                total_score -= match piece {
                    Pawn => {
                        interpolated_pawn_pst(endgame_factor, index)
                            + 100
                            + pawn_bonus_value(square, Black, &white_pawns, &black_pawns)
                    }
                    Knight => KNIGHT_PST[index] + 320,
                    Bishop => BISHOP_PST[index] + 330,
                    Rook => ROOK_PST[index] + 500,
                    Queen => QUEEN_PST[index] + 900,
                    King => evaluate_king_position(index, board, square, Black, endgame_factor),
                };
            }
        }
    }

    if all_major_pieces.popcnt() < 5 {
        // Mop up when there are few pieces left
        // it's good for the player who's winning if they get near the enemy king
        let white_king = white_pieces & board.pieces(Piece::King);
        let black_king = black_pieces & board.pieces(Piece::King);
        let dist = precomputed::DISTANCE_BETWEEN_SQUARES[white_king.to_square().to_index()]
            [black_king.to_square().to_index()];
        // if this is good for white add score
        // if this is good for black remove score
        if total_score > 300 {
            // white is winning
            total_score += 5 * (10 - dist as i32);
        } else if total_score < -300 {
            // black is winning
            total_score -= 5 * (10 - dist as i32);
        } else {
            // it's quite even?
        };
    };

    // also calculate mobility
    // this doesn't seem to be worth the cost
    // let our_mobility = (MoveGen::new_legal(&board).len() * 2) as i32;
    // let their_mobility = if let Some(new_board) = &board.null_move() {
    //     (MoveGen::new_legal(&new_board).len() * 2) as i32
    // } else {
    //     // we're in check, so can't tell how many moves our opponent has - let's guess
    //     20
    // };
    // total_score += our_mobility - their_mobility;

    match board.side_to_move() {
        White => return total_score,
        Black => return -total_score,
    }
}

fn interpolated_pawn_pst(endgame_factor: u32, color_specific_index: usize) -> i32 {
    // endgame factor is from 0 to 6
    match endgame_factor {
        0 => PAWN_PST[color_specific_index],
        1..=5 => {
            (((6 - endgame_factor) as i32 * PAWN_PST[color_specific_index])
                + (endgame_factor as i32 * PAWN_PST_ENDGAME[color_specific_index]))
                / 6
        }
        6 => PAWN_PST_ENDGAME[color_specific_index],
        _ => unreachable!(),
    }
}

fn king_safety(board: &Board, square: Square, king_color: Color) -> i32 {
    // king safety

    // this is important near the beginning of the game
    // but in the endgame when there are fewer enemy pieces you want to open up
    // start w 20 pieces, start scaling down safety factor at 15, hits zero at 5
    let enemy_pieces_remaining = board.color_combined(!king_color).popcnt();
    // safety is multiplied by the clamped # of enemy pieces remaining
    // more threats around means safety is more important
    let safety_factor = enemy_pieces_remaining.clamp(5, 15) - 5; // 0 to 10

    if safety_factor == 0 {
        return 0;
    }

    // pretend there's a bishop, then a rook, where the king is
    // more moves is bad because the king is vulnerable to many attacks
    let mut safety = 5;
    let blockers = board.combined();
    safety -= chess::get_rook_moves(square, *blockers).popcnt() as i32;
    safety -= chess::get_bishop_moves(square, *blockers).popcnt() as i32;

    // TODO: tune
    // ends up being worth like 0-2 pawns
    let safety = safety * safety_factor as i32;
    return safety;
}

fn is_passed_pawn(enemy_pawns: &BitBoard, pawn_square: Square, pawn_color: Color) -> bool {
    // If an enemy pawn is not in the passed pawn mask, it is indeed a passed pawn
    let passed_pawn_mask = passed_pawn_mask_from_square(pawn_square, pawn_color);
    (enemy_pawns.0 & passed_pawn_mask) == 0
}

fn pawn_bonus_value(
    square: Square,
    pawn_color: Color,
    enemy_pawns: &BitBoard,
    friendly_pawns: &BitBoard,
) -> i32 {
    let mut bonus_value: i32 = 0;
    // passed pawns are good, even better with less material on the board
    // only look for passed pawns after the opponent has lost 6 pieces
    if is_passed_pawn(enemy_pawns, square, pawn_color) {
        let squares_to_promotion = {
            // white is 1 square away at rank 7
            // black is 6 squares away at rank 7
            let rank = square.get_rank().to_index(); // 0 to 7
            match pawn_color {
                White => 7 - rank,
                Black => rank,
            }
        };
        bonus_value += PASSED_PAWN_BONUS[squares_to_promotion];
    };

    // let's do some bitboard stuff to figure out if this pawn is supported

    let file_index = square.get_file().to_index();
    let file_mask_center = FILE_MASKS[file_index];
    let file_mask_left = FILE_MASKS[(file_index).max(1) - 1];
    let file_mask_right = FILE_MASKS[(file_index + 1).min(7)];

    // if more than one friendly pawn is in the same file, that's not ideal
    bonus_value += match (friendly_pawns & BitBoard::new(file_mask_center)).popcnt() {
        // this bonus will be applied to each pawn
        0 | 1 => 0,
        2 => -10,
        _ => -20,
    };
    // if a pawn is isolated, that's not ideal
    bonus_value += match (friendly_pawns & BitBoard::new(file_mask_left | file_mask_right)).popcnt()
    {
        0 => -20,
        1 => -6,
        _ => 0,
    };
    return bonus_value;
}

fn endgame_king_modifier(king_square: Square, endgame_factor: u32) -> i32 {
    // being near the center is good
    // being away from the center is bad
    // endgame_factor is between 0 and 6
    if endgame_factor == 0 {
        return 0;
    }
    // at max endgame factor (6) this is between +18 and -18
    // feel like it should be more so i'll scale it by 3
    (3 - DISTANCE_FROM_CENTER[king_square.to_index()]) * 3 * endgame_factor as i32
}

fn evaluate_king_position(
    color_specific_index: usize,
    board: &Board,
    square: Square,
    king_color: Color,
    endgame_factor: u32,
) -> i32 {
    match endgame_factor {
        0 => KING_PST[color_specific_index] + king_safety(board, square, king_color),
        1..=5 => {
            (KING_PST[color_specific_index] / (endgame_factor as i32 - 1).max(1))
                + king_safety(board, square, king_color)
                + endgame_king_modifier(square, endgame_factor)
        }
        6 => endgame_king_modifier(square, endgame_factor),
        _ => unreachable!(),
    }
}

fn passed_pawn_mask_from_square(pawn_square: Square, pawn_color: Color) -> u64 {
    // Check if there are no enemy pawns in the same file or adjacent files
    let file_index = pawn_square.get_file().to_index();
    let total_file_mask = precomputed::TRIPLE_FILE_MASKS[file_index];

    // Generate masks for the rows below or above the pawn
    let rank_index = pawn_square.get_rank().to_index();
    match pawn_color {
        White => {
            let rank_mask_above = u64::MAX << (8 * (7 - rank_index));
            rank_mask_above & total_file_mask
        }
        Black => {
            let rank_mask_below = u64::MAX >> (8 * (8 - rank_index));
            rank_mask_below & total_file_mask
        }
    }
}
