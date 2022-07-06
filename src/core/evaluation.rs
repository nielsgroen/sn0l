use std::cmp::{max, min};
use std::hash::Hash;
use std::str::FromStr;
use chess::{Board, BoardStatus, ChessMove, Color, MoveGen, NUM_PIECES, Piece};

use super::score;
use super::score::Centipawns;

pub fn best_move_depth(board: Board, depth: u64) -> Option<ChessMove> {
    match board.status() {
        BoardStatus::Ongoing => {
            let mut legal_moves = MoveGen::new_legal(&board);

            let mut best_move: Option<ChessMove> = None;
            let mut best_score: Centipawns = Centipawns::new(i64::MIN);

            for legal_move in legal_moves {
                // the minus is because, position is evaluated for other player
                let score = -eval_depth(board.make_move_new(legal_move), depth);

                println!("{}", legal_move);
                println!("{}", score);

                if score > best_score {
                    best_move = Some(legal_move);
                    best_score = score;
                }
            }

            best_move
        },
        _ => None,
    }
}



/// Determines the score of a `Board` without looking at deeper board states
/// following from possible moves
pub fn eval_single(board: Board) -> Centipawns {
    if board.status() == BoardStatus::Checkmate {
        return Centipawns::new(-2 * score::KING_COST.0);
    } else if board.status() == BoardStatus::Stalemate {
        return Centipawns::new(0);
    }

    let mut score: Centipawns = Centipawns::new(0);
    let our_color: Color = board.side_to_move();

    // add up the scores of all our rooks, pawns etc.
    for index in 0..NUM_PIECES {
        score.0 += &score::PIECE_EVALUATIONS[index].0 * (
                board.pieces(chess::ALL_PIECES[index])
                    & board.color_combined(our_color))
                    .popcnt() as i64;
    }

    let mut their_score = Centipawns::new(0);
    let their_color = match our_color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };

    // add up the scores of all their rooks, pawns etc.
    for index in 0..NUM_PIECES {
        their_score.0 += &score::PIECE_EVALUATIONS[index].0 * (
                board.pieces(chess::ALL_PIECES[index])
                    & board.color_combined(their_color))
                    .popcnt() as i64;
    }

    score - their_score
}

/// Will evaluate the board by 'minimaxing' over single board evaluations at a given depth
pub fn eval_depth(board: Board, max_depth: u64) -> Centipawns {
    max_eval(board, max_depth)
}

pub fn max_eval(board: Board, max_depth: u64) -> Centipawns {
    if max_depth == 0
        || board.status() == BoardStatus::Checkmate
        || board.status() == BoardStatus::Stalemate
    {
        return eval_single(board);
    }

    let mut legal_moves = MoveGen::new_legal(&board);
    let mut best_val = Centipawns::new(i64::MIN);
    for legal_move in legal_moves {
        let val = min_eval(board.make_move_new(legal_move), max_depth - 1);
        if val > best_val {
            best_val = val;
        }
    }

    best_val
}

pub fn min_eval(board: Board, max_depth: u64) -> Centipawns {
    if max_depth == 0
        || board.status() == BoardStatus::Checkmate
        || board.status() == BoardStatus::Stalemate
    {
        return eval_single(board);
    }

    let mut legal_moves = MoveGen::new_legal(&board);
    let mut best_val = Centipawns::new(i64::MAX);
    for legal_move in legal_moves {
        let val = min_eval(board.make_move_new(legal_move), max_depth - 1);
        if val < best_val {
            best_val = val;
        }
    }

    best_val
}

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

    assert_eq!(eval_single(board), Centipawns::new(0));
}

#[test]
fn check_single_eval_missing_rook() {
    // only king and rooks, white misses a1 rook.
    let board = Board::from_str("r3k2r/8/8/8/8/8/8/4K2R w Kkq - 0 1").unwrap();

    assert_eq!(eval_single(board), Centipawns::new(-500));
}

// #[test]
// fn check_depth_eval_missing_rook() {
//     // only king and rooks, white misses a1 rook.
//     let board = Board::from_str("r3k2r/8/8/8/8/8/8/4K2R w Kkq - 0 1").unwrap();
//
//     // Note: +500 instead of -500 for eval_single
//     assert_eq!(eval_depth(board, 4), Centipawns::new(500));
// }

#[test]
fn check_best_move_missing_rook() {
    // only king and rooks, white misses a1 rook.
    let board = Board::from_str("r3k2r/8/8/8/8/8/8/4K2R w Kkq - 0 1").unwrap();

    assert_eq!(best_move_depth(board, 2), Some(ChessMove::from_str("h1h8").unwrap()));
}