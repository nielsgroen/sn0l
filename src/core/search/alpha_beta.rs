use std::cmp::{max, min};
use chess::{Board, BoardStatus, ChessMove, Color, MoveGen};
use crate::core::evaluation::{bubble_evaluation, SearchResult, single_evaluation};
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::draw_detection::detect_draw_incremental;
use crate::core::search::move_ordering::{order_captures, order_non_captures};
use crate::core::search::SearchDepth;
use crate::core::search::transposition::{get_transposition, TranspositionTable, update_transposition};

/// The module for alpha-beta search;


pub fn search_depth_pruned(
    board: &Board,
    transposition_table: &mut TranspositionTable,
    visited_boards: Vec<u64>,
    depth: u32,
    selective_depth: Option<u32>,
) -> SearchResult {
    // The base evaluation used for move ordering, and static board scoring
    let selective_depth = selective_depth.unwrap_or(depth);

    let search_result = search_alpha_beta(
        board,
        transposition_table,
        visited_boards,
        // base_evaluation,
        BoardEvaluation::BlackMate(0),
        BoardEvaluation::WhiteMate(0),
        0,
        depth,
        selective_depth,
    );

    search_result
}

pub fn search_alpha_beta(
    board: &Board,
    transposition_table: &mut TranspositionTable,
    mut visited_boards: Vec<u64>,
    // current_evaluation: BoardEvaluation,
    mut alpha: BoardEvaluation,
    mut beta: BoardEvaluation,
    current_depth: u32,
    max_depth: u32,
    max_selective_depth: u32,
) -> SearchResult { // (_, eval, nodes)
    let mut nodes_searched = 1;

    // dummy move, should always be overridden
    // unless the game is over
    let mut best_move = ChessMove::default();
    let current_evaluation = single_evaluation(board);

    if board.status() == BoardStatus::Checkmate {
        return SearchResult::new(
            best_move,
            current_evaluation,
            None,
            None,
        );
    }

    if board.status() == BoardStatus::Stalemate {
        return SearchResult::new(
            best_move,
            current_evaluation,
            None,
            None,
        );
    }

    // Do draw detection before quiescence search
    // => No draw detection necessary when only capturing
    // But still need draw detection on last move before quiescence search
    visited_boards.push(board.get_hash());
    let visited_boards = visited_boards;

    if detect_draw_incremental(&visited_boards) {
        return SearchResult::new(
            best_move,
            BoardEvaluation::PieceScore(Centipawns::new(0)),
            None,
            None,
        );
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

    let mut best_eval = BoardEvaluation::PieceScore(Centipawns::new(0));
    let mut best_path = Vec::new();
    let mut search_result = SearchResult::default();

    if board.side_to_move() == Color::White {
        best_eval = BoardEvaluation::BlackMate(0);

        'outer: for moves in all_moves { // first capture moves, then non-capture moves
            for (chess_move, move_evaluation) in moves.iter() {
                let new_board = &board.make_move_new(*chess_move);

                // if let Some(search_info) = get_transposition(
                //     transposition_table,
                //     &new_board,
                //     SearchDepth::Depth(max_depth - current_depth - 1),
                // ) {
                //     search_result = SearchResult::new(
                //         ChessMove::default(),
                //         search_info.evaluation,
                //         None,
                //         None,
                //     );
                // } else {
                //     search_result = search_alpha_beta(
                //         new_board,
                //         transposition_table,
                //         visited_boards.clone(),
                //         // *move_evaluation,
                //         alpha,
                //         beta,
                //         current_depth + 1,
                //         max_depth,
                //         max_selective_depth,
                //     );
                // }
                search_result = search_alpha_beta(
                    new_board,
                    transposition_table,
                    visited_boards.clone(),
                    // *move_evaluation,
                    alpha,
                    beta,
                    current_depth + 1,
                    max_depth,
                    max_selective_depth,
                );

                nodes_searched += search_result.nodes_searched;
                if search_result.board_evaluation >= best_eval {
                    best_eval = search_result.board_evaluation;
                    best_move = *chess_move;
                    best_path = search_result.critical_path;
                }

                alpha = max(alpha, search_result.board_evaluation);
                if beta < alpha {
                    break 'outer;
                }
            }
        }

    } else { // black to play
        best_eval = BoardEvaluation::WhiteMate(0);

        'outer: for moves in all_moves {
            for (chess_move, move_evaluation) in moves.iter() {
                let new_board = &board.make_move_new(*chess_move);

                // if let Some(search_info) = get_transposition(
                //     transposition_table,
                //     &new_board,
                //     SearchDepth::Depth(max_depth - current_depth - 1),
                // ) {
                //     search_result = SearchResult::new(
                //         ChessMove::default(),
                //         search_info.evaluation,
                //         None,
                //         None,
                //     );
                // } else {
                //     search_result = search_alpha_beta(
                //         new_board,
                //         transposition_table,
                //         visited_boards.clone(),
                //         // *move_evaluation,
                //         alpha,
                //         beta,
                //         current_depth + 1,
                //         max_depth,
                //         max_selective_depth,
                //     );
                // }
                search_result = search_alpha_beta(
                    new_board,
                    transposition_table,
                    visited_boards.clone(),
                    // *move_evaluation,
                    alpha,
                    beta,
                    current_depth + 1,
                    max_depth,
                    max_selective_depth,
                );

                nodes_searched += search_result.nodes_searched;

                if search_result.board_evaluation <= best_eval {
                    best_eval = search_result.board_evaluation;
                    best_move = *chess_move;
                    best_path = search_result.critical_path;
                }

                beta = min(beta, search_result.board_evaluation);
                if beta < alpha {
                    break 'outer;
                }
            }
        }
    }

    best_eval = bubble_evaluation(best_eval);
    update_transposition(
        transposition_table,
        board,
        SearchDepth::Depth(max_depth - current_depth),
        best_eval,
    );

    best_path.push(best_move);
    SearchResult::new(
        best_move,
        best_eval,
        Some(nodes_searched),
        Some(best_path),
    )
}


pub fn quiescence_alpha_beta(
    board: &Board,
    transposition_table: &mut TranspositionTable,
    // current_evaluation: BoardEvaluation,
    mut alpha: BoardEvaluation,
    mut beta: BoardEvaluation,
    current_depth: u32,
    max_selective_depth: u32,
) -> SearchResult { // (_, eval, nodes)
    let mut nodes_searched = 1;
    let mut best_move = ChessMove::default();
    let current_evaluation = single_evaluation(board);

    if current_depth >= max_selective_depth {
        return SearchResult::new(
            best_move,
            current_evaluation,
            None,
            None,
        );
    }

    let mut move_gen = MoveGen::new_legal(&board);

    let moves = order_captures(
        &board,
        current_evaluation,
        transposition_table,
        &mut move_gen,
    );

    if moves.is_empty() {
        return SearchResult::new(
            best_move,
            current_evaluation,
            None,
            None,
        );
    }

    if board.side_to_move() == Color::White {
        let mut best_eval = BoardEvaluation::BlackMate(0);
        let mut best_path = Vec::new();
        let mut search_result = SearchResult::default();

        for (chess_move, move_evaluation) in moves.iter() {
            search_result = quiescence_alpha_beta(
                &board.make_move_new(*chess_move),
                transposition_table,
                alpha,
                beta,
                current_depth + 1,
                max_selective_depth,
            );
            nodes_searched += search_result.nodes_searched;

            if search_result.board_evaluation >= best_eval {
                best_eval = search_result.board_evaluation;
                best_move = *chess_move;
                best_path = search_result.critical_path;
                best_path.push(best_move);
            }

            alpha = max(alpha, search_result.board_evaluation);
            if beta < alpha {
                best_eval = bubble_evaluation(best_eval);
                update_transposition(
                    transposition_table,
                    board,
                    SearchDepth::Quiescent,
                    best_eval,
                );
                best_path.push(best_move);
                return SearchResult::new(
                    best_move,
                    best_eval,
                    Some(nodes_searched),
                    Some(best_path),
                );
            }
        }

        // ADD NULL_MOVE FOR QUIESCENCE SEARCH:
        // Opponent can choose to not capture most of the time.
        // Can play non-captures, but those are not accounted for in quiescence search.
        // (Not forced to take pawn with queen if only capture available)
        if let Some(null_board) = board.null_move() {
            search_result = quiescence_alpha_beta(
                &null_board,
                transposition_table,
                alpha,
                beta,
                current_depth + 1,
                max_selective_depth - 1, // Quiescence should cut off at even depth, and we're skipping a move
            );
            nodes_searched += search_result.nodes_searched;

            if search_result.board_evaluation >= best_eval {
                best_eval = search_result.board_evaluation;
                best_move = ChessMove::default();
                best_path = Vec::new(); // Not an actual legal move => no actual line of play further
            }
        }

        // Or take the static evaluation with a penalty, to remove the amount of blunders
        // due to "having to capture, or give up the turn"
        match current_evaluation {
            BoardEvaluation::PieceScore(x) => {
                let penalized_score = BoardEvaluation::PieceScore(x - Centipawns::new(54));

                if penalized_score > best_eval {
                    best_eval = penalized_score;
                    best_move = ChessMove::default();
                    best_path = Vec::new();
                }
            },
            _ => (),
        };

        best_eval = bubble_evaluation(best_eval);
        update_transposition(
            transposition_table,
            board,
            SearchDepth::Quiescent,
            best_eval,
        );

        SearchResult::new(
            best_move,
            best_eval,
            Some(nodes_searched),
            Some(best_path),
        )
    } else { // black to play
        let mut best_eval = BoardEvaluation::WhiteMate(0);
        let mut best_path = Vec::new();
        let mut search_result = SearchResult::default();

        for (chess_move, move_evaluation) in moves.iter() {
            search_result = quiescence_alpha_beta(
                &board.make_move_new(*chess_move),
                transposition_table,
                alpha,
                beta,
                current_depth + 1,
                max_selective_depth,
            );
            nodes_searched += search_result.nodes_searched;

            if search_result.board_evaluation <= best_eval {
                best_eval = search_result.board_evaluation;
                best_move = *chess_move;
                best_path = search_result.critical_path;
                best_path.push(best_move);
            }

            beta = min(beta, search_result.board_evaluation);
            if beta < alpha {
                best_eval = bubble_evaluation(best_eval);
                update_transposition(
                    transposition_table,
                    board,
                    SearchDepth::Quiescent,
                    best_eval,
                );
                best_path.push(best_move);
                return SearchResult::new(
                    best_move,
                    best_eval,
                    Some(nodes_searched),
                    Some(best_path),
                );
            }
        }
        // ADD NULL_MOVE FOR QUIESCENCE SEARCH:
        // Opponent can choose to not capture most of the time.
        // Can play non-captures, but those are not accounted for in quiescence search.
        // (Not forced to take pawn with queen if only capture available)
        if let Some(null_board) = board.null_move() {
            search_result = quiescence_alpha_beta(
                &null_board,
                transposition_table,
                alpha,
                beta,
                current_depth + 1,
                max_selective_depth - 1, // Quiescence should cut off at even depth, and we're skipping a move
            );
            nodes_searched += search_result.nodes_searched;

            if search_result.board_evaluation <= best_eval {
                best_eval = search_result.board_evaluation;
                best_move = ChessMove::default();
                best_path = Vec::new(); // Not an actual legal move => no actual line of play further
            }
        }

        // Or take the static evaluation with a penalty, to remove the amount of blunders
        // due to "having to capture, or give up the turn"
        match current_evaluation {
            BoardEvaluation::PieceScore(x) => {
                let penalized_score = BoardEvaluation::PieceScore(x + Centipawns::new(54));

                if penalized_score < best_eval {
                    best_eval = penalized_score;
                    best_move = ChessMove::default();
                    best_path = Vec::new();
                }
            },
            _ => (),
        };

        best_eval = bubble_evaluation(best_eval);
        update_transposition(
            transposition_table,
            board,
            SearchDepth::Quiescent,
            best_eval,
        );

        SearchResult::new(
            best_move,
            best_eval,
            Some(nodes_searched),
            Some(best_path),
        )
    }
}