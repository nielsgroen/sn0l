use std::cmp::{max, min};
use std::hash::Hash;
use std::ops::BitAnd;
use std::str::FromStr;
use std::iter::ExactSizeIterator;
use chess::{BitBoard, Board, BoardStatus, ChessMove, Color, EMPTY, MoveGen, NUM_PIECES, Piece};

use super::score;
use super::score::Centipawns;

pub fn best_move_depth(board: &Board, depth: u64) -> Option<ChessMove> {
    match board.status() {
        BoardStatus::Ongoing => {
            let masks = determine_masks(&board);

            let mut legal_moves = MoveGen::new_legal(&board);

            let mut best_move: Option<ChessMove> = None;
            let mut best_val = Centipawns::new(i64::MIN) + Centipawns::new(1);  // avoids overflow for -best_val

            for mask in masks {
                legal_moves.set_iterator_mask(mask);
                for legal_move in &mut legal_moves {
                    // the minus is because, position is evaluated for other player
                    // let score = -eval_depth(board.make_move_new(legal_move), depth - 1);
                    let score = -max_eval_pruned(board.make_move_new(legal_move), depth - 1, 0, -best_val);

                    // println!("{}", legal_move);
                    // println!("{}", score);

                    if score > best_val {
                        best_move = Some(legal_move);
                        best_val = score;
                    }
                }
            }

            best_move
        },
        _ => None,
    }
}



/// Determines the score of a `Board` without looking at deeper board states
/// following from possible moves
pub fn eval_single(board: &Board) -> Centipawns {
    if board.status() == BoardStatus::Checkmate {
        return Centipawns::new(-2 * score::KING_COST.0);
    } else if board.status() == BoardStatus::Stalemate {
        return Centipawns::new(0);
    }

    let mut score: Centipawns = Centipawns::new(0);
    let our_color: Color = board.side_to_move();

    // add up the scores of all our rooks, pawns etc.

    // Determine piece scores
    // Iterate over all bits in bitboard
    for piece in chess::ALL_PIECES {
        let BitBoard(mut piece_positions) = board.pieces(piece).bitand(board.color_combined(our_color));
        'inner: for index in 0..64 {
            score += Centipawns::new(score::score_tables::piece_table(our_color, piece)[index] as i64 * (piece_positions & 1) as i64);
            piece_positions >>= 1;  // Iterate over all set bits on bitboard

            if piece_positions == 0 {
                break 'inner;
            }
        }
    }

    let mut their_score = Centipawns::new(0);
    let their_color = match our_color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };

    // Determine piece scores
    // Iterate over all bits in bitboard
    for piece in chess::ALL_PIECES {
        let BitBoard(mut piece_positions) = board.pieces(piece).bitand(board.color_combined(their_color));
        'inner: for index in 0..64 {
            their_score += Centipawns::new(score::score_tables::piece_table(their_color, piece)[index] as i64 * (piece_positions & 1) as i64);
            piece_positions >>= 1;

            if piece_positions == 0 {
                break 'inner;
            }
        }
    }

    score - their_score
}

/// Will evaluate the board by 'minimaxing' over single board evaluations at a given depth
pub fn eval_depth(board: Board, max_depth: u64) -> Centipawns {
    return max_eval_pruned(board, max_depth, 0, Centipawns::new(i64::MAX));
}

#[deprecated]
pub fn max_eval(board: Board, max_depth: u64, current_depth: u64) -> Centipawns {
    if max_depth == current_depth
        || board.status() == BoardStatus::Checkmate
        || board.status() == BoardStatus::Stalemate
    {
        return eval_single(&board);
        // let ev = eval_single(board);
        // if current_depth % 2 == 0 {
        //     return ev;
        // } else {
        //     return -ev;
        // }
    }

    let mut legal_moves = MoveGen::new_legal(&board);
    let mut best_move = ChessMove::from_str("a1a2").unwrap();
    let mut best_val = Centipawns::new(i64::MIN);

    for legal_move in legal_moves {
        let val = -max_eval(board.make_move_new(legal_move), max_depth, current_depth + 1);
        if val > best_val {
            best_val = val;
            best_move = legal_move;
        }
    }

    // println!("{:?}", best_move);
    // println!("{:?}", best_val);
    best_val
}

pub fn max_eval_pruned(board: Board, max_depth: u64, current_depth: u64, prune_cutoff: Centipawns) -> Centipawns {
    if board.status() == BoardStatus::Checkmate
        || board.status() == BoardStatus::Stalemate
    {
        return eval_single(&board);
        // let ev = eval_single(board);
        // if current_depth % 2 == 0 {
        //     return ev;
        // } else {
        //     return -ev;
        // }
    }

    if max_depth <= current_depth {
        return quiescence_search(board, prune_cutoff);
    }

    let masks = determine_masks(&board);

    let mut legal_moves = MoveGen::new_legal(&board);
    // let mut best_move = ChessMove::from_str("a1a2").unwrap();
    let mut best_val = Centipawns::new(i64::MIN) + Centipawns::new(1);  // avoids overflow for -best_val

    for mask in masks {
        legal_moves.set_iterator_mask(mask);
        for legal_move in &mut legal_moves {
            let val = -max_eval_pruned(board.make_move_new(legal_move), max_depth, current_depth + 1, -best_val);
            if val > best_val {
                best_val = val;
                // best_move = legal_move;
            }
            if best_val > prune_cutoff {
                return best_val;
            }
        }
    }

    // println!("{:?}", best_move);
    // println!("{:?}", best_val);
    best_val
}

pub fn quiescence_search(board: Board, prune_cutoff: Centipawns) -> Centipawns {
    let their_color = match board.side_to_move() {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };

    // println!("Hello");
    let capture_mask = *board.color_combined(their_color);
    let mut legal_captures = MoveGen::new_legal(&board);
    legal_captures.set_iterator_mask(capture_mask);
    let mut best_val = Centipawns::new(i64::MIN) + Centipawns::new(1);

    let mut checked_one = false;

    for capture_move in &mut legal_captures {
        // a bit dirty, but the most reliable way to check if we checked 'at least one move' is by setting a var in this loop
        // MoveGen.len() is for all moves w/ all masks
        checked_one = true;

        let val = -quiescence_search(board.make_move_new(capture_move), -best_val);
        if val > best_val {
            best_val = val;
        }
        if best_val > prune_cutoff {
            return best_val;
        }
    }

    if !checked_one {
        return eval_single(&board);
    }

    best_val
}

fn determine_masks(board: &Board) -> [BitBoard; 6] {
    let opponent_piece_locations = board.color_combined(!(&board).side_to_move());

    [
        score::VERY_CENTER.bitand(opponent_piece_locations),
        score::CENTER.bitand(opponent_piece_locations),
        *opponent_piece_locations,
        score::VERY_CENTER,
        score::CENTER,
        !chess::EMPTY,
    ]
}
// pub fn min_eval(board: Board, max_depth: u64, current_depth: u64) -> Centipawns {
//     if max_depth == current_depth
//         || board.status() == BoardStatus::Checkmate
//         || board.status() == BoardStatus::Stalemate
//     {
//         let ev = eval_single(board);
//         if current_depth % 2 == 0 {
//             return ev;
//         } else {
//             return -ev;
//         }
//     }
//
//     let mut legal_moves = MoveGen::new_legal(&board);
//     let mut best_move = ChessMove::from_str("a1a2").unwrap();
//     let mut best_val = Centipawns::new(i64::MAX);
//     for legal_move in legal_moves {
//         let val = min_eval(board.make_move_new(legal_move), max_depth, current_depth + 1);
//         if val < best_val {
//             best_val = val;
//         }
//     }
//     // println!("{:?}", best_move);
//     // println!("{:?}", best_val);
//
//     best_val
// }

// pub fn eval_depth(board: Board, max_depth: u64) -> Centipawns {
//     // DFS over board states
//     let mut current_board = board;
//     let mut current_eval: Option<Centipawns> = None;
//     let mut current_depth: u64 = 0;
//     let mut work_stack: Vec<Board> = Vec::new();
//     work_stack.insert(0, board);
//     let mut parents: Vec<Board> = Vec::new();
//     let mut parent_evals: Vec<Option<Centipawns>> = Vec::new();
//     let mut children_left: Vec<usize> = Vec::new();
//
//     while work_stack.len() > 0 {
//         // println!("parents {:?}", parents.iter().map(|x| {x.get_hash()}).collect::<Vec<_>>());
//         // println!("work_stack {:?}", work_stack.iter().map(|x| {x.get_hash()}).collect::<Vec<u64>>());
//         // println!("parent_evals {:?}", parent_evals);
//         current_board = work_stack.pop().unwrap();
//
//         // determine score based on if board is terminal
//         let board_status = current_board.status();
//         current_eval = match board_status {
//             BoardStatus::Stalemate => Some(Centipawns::new(0)),
//             BoardStatus::Checkmate => Some(Centipawns::new(-2 * score::KING_COST.0)),
//             BoardStatus::Ongoing => {
//                 let mut result: Option<Centipawns> = None;
//
//                 if current_depth == max_depth {
//                     result = Some(eval_single(board));
//                 }
//                 result
//             },
//         };
//
//         // println!("{:?}", current_eval);
//
//         // if this already node has some eval => node is terminal
//         if let Some(cur_eval) = current_eval {
//             // bubble up board eval to parents
//             // bubble_up_parent_evals(rooted_eval, &mut parent_evals, current_depth);
//             let cur_eval = match current_depth {
//                 x if x % 2 == 0 => cur_eval,
//                 _ => -cur_eval,
//             };
//
//             // check if can move directly sideways, or need to move up first
//             loop {
//                 let children_left_len = children_left.len();
//                 children_left[children_left_len - 1] -= 1;
//
//                 let parent_evals_len = parent_evals.len();
//                 let current_parent_val = parent_evals[parent_evals_len - 1];
//                 parent_evals[parent_evals_len - 1] = match current_parent_val {
//                     None => Some(cur_eval),
//                     Some(x) => match current_depth {
//                         d if d % 2 == 0 => Some(Centipawns::new(max(x.0, cur_eval.0))),
//                         _ => Some(Centipawns::new(min(x.0, cur_eval.0))),
//                     }
//                 };
//                 if children_left[children_left_len - 1] > 0 {  // move sideways
//                     break;
//                 } else {  // move up
//                     let parent_evals_len = parent_evals.len();
//                     let current_parent_val = parent_evals[parent_evals_len - 2];
//                     parent_evals[parent_evals_len - 2] = match current_parent_val {
//                         None => Some(parent_evals[parent_evals_len - 1].unwrap()),
//                         Some(x) => match current_depth {
//                             d if d % 2 == 0 => Some(Centipawns::new(max(x.0, parent_evals[parent_evals_len - 1].unwrap().0))),
//                             _ => Some(Centipawns::new(min(x.0, parent_evals[parent_evals_len - 1].unwrap().0))),
//                         }
//                     };
//
//
//
//                     if work_stack.len() == 0 {  // children_left is empty, but want to keep final result
//                         break;
//                     }
//                     children_left.pop();
//                     parents.pop();
//                     parent_evals.pop();
//                     current_depth -= 1;
//                     if children_left.len() == 0 {
//                         break;
//                     }
//                 }
//             }
//         } else {  // Node is not terminal => go deeper
//             let new_legal = MoveGen::new_legal(&current_board);
//             let new_legal_len = new_legal.len();
//             work_stack.extend(new_legal.map(|x| -> Board {current_board.make_move_new(x)}));
//             parents.insert(parents.len(), current_board);
//             parent_evals.insert(parent_evals.len(), current_eval);
//             children_left.insert(children_left.len(), new_legal_len);
//             current_depth += 1;
//         }
//     }
//
//     parent_evals.pop().unwrap().unwrap()
// }

fn bubble_up_parent_evals(current_eval: Centipawns, parent_evals: &mut Vec<Option<Centipawns>>, current_depth: u64) {
    let mut eval = current_eval;
    let mut depth = current_depth;
    for index in 1..(parent_evals.len() + 1) {
        let new_eval = match parent_evals[parent_evals.len() - index] {
            None => Centipawns::new(eval.0),
            Some(x) => match depth {
                d if d % 2 == 0 => Centipawns::new(max(x.0, eval.0)),
                _ => Centipawns::new(min(x.0, eval.0))
            },
        };
        let parent_evals_len = parent_evals.len();
        parent_evals[parent_evals_len - index] = Some(new_eval);
        eval = new_eval;
        depth -= 1;
    }
}

#[test]
fn check_single_eval_startpos() {
    let board = Board::default();

    assert_eq!(eval_single(&board), Centipawns::new(0));
}

#[test]
fn check_single_eval_missing_rook() {
    // only king and rooks, white misses a1 rook.
    let board = Board::from_str("r3k2r/8/8/8/8/8/8/4K2R w Kkq - 0 1").unwrap();

    assert_eq!(eval_single(&board), Centipawns::new(-500));
}

#[test]
fn check_depth_eval_missing_rook() {
    // only king and rooks, white misses a1 rook.
    let board = Board::from_str("r3k2r/8/8/8/8/8/8/4K2R w Kkq - 0 1").unwrap();

    // Note: +500 instead of -500 for eval_single
    assert_eq!(eval_depth(board, 4), Centipawns::new(500));
}

#[test]
fn check_best_move_missing_rook() {
    // only king and rooks, white misses a1 rook.
    let board = Board::from_str("r3k2r/8/8/8/8/8/8/4K2R w Kkq - 0 1").unwrap();

    assert_eq!(best_move_depth(&board, 4), Some(ChessMove::from_str("h1h8").unwrap()));
}

#[test]
fn check_pruning_correctness() {
    let boards = [
        Board::from_str("r1b1k3/1p3p1B/p3p3/2bpP2p/7q/2N2K2/PPP4P/RNB4n b q - 1 20").unwrap(),
        Board::from_str("r1bqk3/1p3p1p/p3p2Q/2bpP3/6n1/2NB1n2/PPP2PrP/RNBK3R w q - 3 14").unwrap(),
        Board::from_str("2kr1b1r/B1pqp2p/1p2p3/5np1/4NP2/Q2P1P2/PPP4P/2KR3R b - - 1 16").unwrap(),
        Board::from_str("r3k2r/8/8/8/8/8/8/4K2R w Kkq - 0 1").unwrap(),
        Board::from_str("r3k1r1/8/8/8/8/8/8/4K2R b Kq - 0 1").unwrap(),
        Board::from_str("r3k1r1/8/8/8/8/8/8/4K2R w Kq - 0 1").unwrap(),
    ];

    for board in boards {
        assert_eq!(max_eval(board, 3, 0), max_eval_pruned(board, 3, 0, Centipawns::new(i64::MAX)))
    }
}