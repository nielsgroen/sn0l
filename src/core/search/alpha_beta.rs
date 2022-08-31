use std::cmp::{max, min};
use chess::{Board, BoardStatus, ChessMove, Color, MoveGen};
use crate::core::evaluation::{bubble_evaluation, single_evaluation};
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::move_ordering::{order_captures, order_non_captures};
use crate::core::search::SearchDepth;
use crate::core::search::transposition::{TranspositionTable, update_transpositions};

/// The module for alpha-beta search;


pub fn search_depth_pruned(
    board: &Board,
    transposition_table: &mut TranspositionTable,
    depth: u32,
    selective_depth: Option<u32>,
) -> (ChessMove, BoardEvaluation, u32) {
    // The base evaluation used for move ordering, and static board scoring
    let selective_depth = selective_depth.unwrap_or(depth);

    let (chess_move, score, nodes) = search_alpha_beta(
        board,
        transposition_table,
        // base_evaluation,
        BoardEvaluation::BlackMate(0),
        BoardEvaluation::WhiteMate(0),
        0,
        depth,
        selective_depth,
    );

    (chess_move, score, nodes)
}

pub fn search_alpha_beta(
    board: &Board,
    transposition_table: &mut TranspositionTable,
    // current_evaluation: BoardEvaluation,
    mut alpha: BoardEvaluation,
    mut beta: BoardEvaluation,
    current_depth: u32,
    max_depth: u32,
    max_selective_depth: u32,
) -> (ChessMove, BoardEvaluation, u32) { // (_, eval, nodes)
    let mut nodes_searched = 1;

    // dummy move, should always be overridden
    // unless the game is over
    let mut best_move = ChessMove::default();
    let current_evaluation = single_evaluation(board);

    if board.status() == BoardStatus::Checkmate {
        return match board.side_to_move() {
            Color::White => (best_move, BoardEvaluation::BlackMate(0), nodes_searched), // black has checkmated white
            Color::Black => (best_move, BoardEvaluation::WhiteMate(0), nodes_searched),
        }
    }

    if board.status() == BoardStatus::Stalemate {
        return (best_move, BoardEvaluation::PieceScore(Centipawns::new(0)), nodes_searched);
    }

    if current_depth >= max_depth {
        return quiescence_alpha_beta(
            board,
            transposition_table,
            // current_evaluation,
            alpha,
            beta,
            current_depth + 1,
            max_selective_depth
        );
    }

    let mut move_gen = MoveGen::new_legal(board);
    let all_moves = [
        order_captures(
            board,
            current_evaluation,
            transposition_table,
            &mut move_gen,
        ),
        order_non_captures(
            board,
            current_evaluation,
            transposition_table,
            &mut move_gen,
        ),
    ];

    if board.side_to_move() == Color::White {
        let mut best_eval = BoardEvaluation::BlackMate(0);

        for moves in all_moves { // first capture moves, then non-capture moves
            for (chess_move, move_evaluation) in moves.iter() {
                nodes_searched += 1;

                let (_, eval, sub_nodes) = search_alpha_beta(
                    &board.make_move_new(*chess_move),
                    transposition_table,
                    // *move_evaluation,
                    alpha,
                    beta,
                    current_depth + 1,
                    max_depth,
                    max_selective_depth,
                );

                nodes_searched += sub_nodes;
                if eval >= best_eval {
                    best_eval = eval;
                    best_move = *chess_move;
                }

                alpha = max(alpha, eval);
                if beta <= alpha {
                    best_eval = bubble_evaluation(best_eval);
                    update_transpositions(
                        transposition_table,
                        board,
                        SearchDepth::Depth(max_depth - current_depth),
                        best_eval,
                    );
                    return (best_move, best_eval, nodes_searched);
                }
            }
        }

        best_eval = bubble_evaluation(best_eval);
        update_transpositions(
            transposition_table,
            board,
            SearchDepth::Depth(max_depth - current_depth),
            best_eval,
        );

        (best_move, best_eval, nodes_searched)
    } else { // black to play
        let mut best_eval = BoardEvaluation::WhiteMate(0);

        for moves in all_moves {
            for (chess_move, move_evaluation) in moves.iter() {
                nodes_searched += 1;

                let (_, eval, sub_nodes) = search_alpha_beta(
                    &board.make_move_new(*chess_move),
                    transposition_table,
                    // *move_evaluation,
                    alpha,
                    beta,
                    current_depth + 1,
                    max_depth,
                    max_selective_depth,
                );
                nodes_searched += sub_nodes;

                if eval <= best_eval {
                    best_eval = eval;
                    best_move = *chess_move;
                }

                beta = min(beta, eval);
                if beta <= alpha {
                    best_eval = bubble_evaluation(best_eval);
                    update_transpositions(
                        transposition_table,
                        board,
                        SearchDepth::Depth(max_depth - current_depth),
                        best_eval,
                    );
                    return (best_move, best_eval, nodes_searched);
                }
            }
        }

        best_eval = bubble_evaluation(best_eval);
        update_transpositions(
            transposition_table,
            board,
            SearchDepth::Depth(max_depth - current_depth),
            best_eval,
        );
        (best_move, best_eval, nodes_searched)
    }
}


pub fn quiescence_alpha_beta(
    board: &Board,
    transposition_table: &mut TranspositionTable,
    // current_evaluation: BoardEvaluation,
    mut alpha: BoardEvaluation,
    mut beta: BoardEvaluation,
    current_depth: u32,
    max_selective_depth: u32,
) -> (ChessMove, BoardEvaluation, u32) { // (_, eval, nodes)
    let mut nodes_searched = 1;
    let mut best_move = ChessMove::default();
    let current_evaluation = single_evaluation(board);

    if current_depth >= max_selective_depth {
        return (best_move, current_evaluation, nodes_searched);
    }

    let mut move_gen = MoveGen::new_legal(&board);

    let moves = order_captures(
        &board,
        current_evaluation,
        transposition_table,
        &mut move_gen,
    );

    if moves.is_empty() {
        return (best_move, single_evaluation(board), nodes_searched);
    }

    if board.side_to_move() == Color::White {
        let mut best_eval = BoardEvaluation::BlackMate(0);

        for (chess_move, move_evaluation) in moves.iter() {
            nodes_searched += 1;

            let (_, eval, sub_nodes) = quiescence_alpha_beta(
                &board.make_move_new(*chess_move),
                transposition_table,
                alpha,
                beta,
                current_depth + 1,
                max_selective_depth,
            );
            nodes_searched += sub_nodes;

            if eval >= best_eval {
                best_eval = eval;
                best_move = *chess_move;
            }

            alpha = max(alpha, eval);
            if beta <= alpha {
                best_eval = bubble_evaluation(best_eval);
                update_transpositions(
                    transposition_table,
                    board,
                    SearchDepth::Quiescent,
                    best_eval,
                );
                return (best_move, best_eval, nodes_searched);
            }
        }

        best_eval = bubble_evaluation(best_eval);
        update_transpositions(
            transposition_table,
            board,
            SearchDepth::Quiescent,
            best_eval,
        );
        (best_move, best_eval, nodes_searched)
    } else { // black to play
        let mut best_eval = BoardEvaluation::WhiteMate(0);

        for (chess_move, move_evaluation) in moves.iter() {
            nodes_searched += 1;

            let (_, eval, sub_nodes) = quiescence_alpha_beta(
                &board.make_move_new(*chess_move),
                transposition_table,
                alpha,
                beta,
                current_depth + 1,
                max_selective_depth,
            );
            nodes_searched += sub_nodes;

            if eval <= best_eval {
                best_eval = eval;
                best_move = *chess_move;
            }

            beta = min(beta, eval);
            if beta <= alpha {
                best_eval = bubble_evaluation(best_eval);
                update_transpositions(
                    transposition_table,
                    board,
                    SearchDepth::Quiescent,
                    best_eval,
                );
                return (best_move, best_eval, nodes_searched);
            }
        }

        best_eval = bubble_evaluation(best_eval);
        update_transpositions(
            transposition_table,
            board,
            SearchDepth::Quiescent,
            best_eval,
        );
        (best_move, best_eval, nodes_searched)
    }
}