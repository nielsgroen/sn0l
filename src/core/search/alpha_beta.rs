use std::cmp::{max, min};
use chess::{Board, BoardStatus, ChessMove, Color, MoveGen};
use crate::core::evaluation::single_evaluation;
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::move_ordering::{order_captures, order_non_captures};
use crate::core::search::transposition::TranspositionTable;

/// The module for alpha-beta search;


pub fn search_depth_pruned(
    board: &Board,
    transposition_table: &mut TranspositionTable,
    depth: u32
) -> (ChessMove, BoardEvaluation) {
    // The base evaluation used for move ordering, and static board scoring
    let base_evaluation = single_evaluation(&board);

    let (chess_move, score) = search_alpha_beta(
        board,
        transposition_table,
        // base_evaluation,
        BoardEvaluation::BlackMate(0),
        BoardEvaluation::WhiteMate(0),
        0,
        depth,
    );

    (chess_move, score)
}

pub fn search_alpha_beta(
    board: &Board,
    transposition_table: &mut TranspositionTable,
    // current_evaluation: BoardEvaluation,
    mut alpha: BoardEvaluation,
    mut beta: BoardEvaluation,
    current_depth: u32,
    max_depth: u32,
) -> (ChessMove, BoardEvaluation) { // (_, eval)
    // dummy move, should always be overridden
    // unless the game is over
    let mut best_move = ChessMove::default();

    if board.status() == BoardStatus::Checkmate {
        return match board.side_to_move() {
            Color::White => (best_move, BoardEvaluation::BlackMate(0)), // black has checkmated white
            Color::Black => (best_move, BoardEvaluation::WhiteMate(0)),
        }
    }

    if board.status() == BoardStatus::Stalemate {
        return (best_move, BoardEvaluation::PieceScore(Centipawns::new(0)));
    }

    if current_depth >= max_depth {
        return quiescence_alpha_beta(
            board,
            transposition_table,
            // current_evaluation,
            alpha,
            beta,
        );
    }

    let mut move_gen = MoveGen::new_legal(board);
    let all_moves = [
        order_captures(
            board,
            // current_evaluation,
            transposition_table,
            &mut move_gen,
        ),
        order_non_captures(
            board,
            // current_evaluation,
            transposition_table,
            &mut move_gen,
        ),
    ];

    if board.side_to_move() == Color::White {
        let mut best_eval = BoardEvaluation::BlackMate(0);

        for moves in all_moves { // first capture moves, then non-capture moves
            for (chess_move, move_evaluation) in moves.iter() {
                let (_, _, eval) = search_alpha_beta(
                    &board.make_move_new(*chess_move),
                    transposition_table,
                    // *move_evaluation,
                    alpha,
                    beta,
                    current_depth + 1,
                    max_depth,
                );
                if eval >= best_eval {
                    best_eval = eval;
                    best_move = *chess_move;
                }

                alpha = max(alpha, eval);
                if beta <= alpha {
                    return (best_move, best_eval);
                }
            }
        }

        (best_move, best_eval)
    } else { // black to play
        let mut best_eval = BoardEvaluation::WhiteMate(0);

        for moves in all_moves {
            for (chess_move, move_evaluation) in moves.iter() {
                let (_, eval) = search_alpha_beta(
                    &board.make_move_new(*chess_move),
                    transposition_table,
                    // *move_evaluation,
                    alpha,
                    beta,
                    current_depth + 1,
                    max_depth,
                );
                if eval <= best_eval {
                    best_eval = eval;
                    best_move = *chess_move;
                }

                beta = min(beta, eval);
                if beta <= alpha {
                    return (best_move, best_eval);
                }
            }
        }

        (best_move, best_eval)
    }
}


pub fn quiescence_alpha_beta(
    board: &Board,
    transposition_table: &mut TranspositionTable,
    // current_evaluation: BoardEvaluation,
    mut alpha: BoardEvaluation,
    mut beta: BoardEvaluation,
) -> (ChessMove, BoardEvaluation) { // (_, eval)
    let mut best_move = ChessMove::default();

    if board.status() == BoardStatus::Checkmate {
        return match board.side_to_move() {
            Color::White => (best_move, BoardEvaluation::BlackMate(0)), // black has checkmated white
            Color::Black => (best_move, BoardEvaluation::WhiteMate(0)),
        }
    }

    if board.status() == BoardStatus::Stalemate {
        return (best_move, BoardEvaluation::PieceScore(Centipawns::new(0)));
    }

    let mut move_gen = MoveGen::new_legal(&board);

    let moves = order_captures(
        &board,
        transposition_table,
        &mut move_gen,
    );

    if board.side_to_move() == Color::White {
        let mut best_eval = BoardEvaluation::BlackMate(0);

        for (chess_move, move_evaluation) in moves.iter() {
            let (_, _, eval) = quiescence_alpha_beta(
                &board.make_move_new(*chess_move),
                transposition_table,
                alpha,
                beta,
            );
            if eval >= best_eval {
                best_eval = eval;
                best_move = *chess_move;
            }

            alpha = max(alpha, eval);
            if beta <= alpha {
                return (best_move, best_eval);
            }
        }

        (best_move, best_eval)
    } else { // black to play
        let mut best_eval = BoardEvaluation::WhiteMate(0);

        for (chess_move, move_evaluation) in moves.iter() {
            let (_, eval) = quiescence_alpha_beta(
                &board.make_move_new(*chess_move),
                transposition_table,
                alpha,
                beta,
            );
            if eval <= best_eval {
                best_eval = eval;
                best_move = *chess_move;
            }

            beta = min(beta, eval);
            if beta <= alpha {
                return (best_move, best_eval);
            }
        }

        (best_move, best_eval)
    }
}