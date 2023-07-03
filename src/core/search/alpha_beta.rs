use std::time::{Duration, SystemTime, UNIX_EPOCH};
use chess::{Board, BoardStatus, ChessMove, Color, MoveGen};
use crate::analysis::database::rows::PositionSearchRow;
use crate::core::evaluation::{bubble_evaluation, game_status, single_evaluation};
use crate::core::evaluation::incremental::incremental_evaluation;
use crate::core::score::{BoardEvaluation, Centipawns};
use crate::core::search::common::check_game_over;
use crate::core::search::draw_detection::detect_draw_incremental;
use crate::core::search::move_ordering::order_moves;
use crate::core::search::search_result::SearchResult;
use crate::core::search::SearchDepth;
use crate::core::search::transpositions::{EvalBound, TranspositionTable};

/// The module for alpha-beta search;


pub fn search_depth_pruned<T: SearchResult + Default>(
    board: &Board,
    // transposition_table: &mut impl TranspositionTable,
    transposition_table: &mut Box<dyn TranspositionTable>,
    visited_boards: Vec<u64>,
    depth: u32,
    selective_depth: Option<u32>,
) -> (T, PositionSearchRow) {
    // The base evaluation used for move ordering, and static board scoring
    let selective_depth = selective_depth.unwrap_or(depth);
    let simple_eval = single_evaluation(&board, board.status());

    let simple_score;
    match simple_eval {
        BoardEvaluation::PieceScore(x) => simple_score = x,
        _ => panic!("searching finished position"),
    }

    let total_search_time = SystemTime::now();

    let search_result: T = search_alpha_beta(
        board,
        transposition_table,
        visited_boards,
        simple_score,
        EvalBound::Exact(BoardEvaluation::BlackMate(0)),
        EvalBound::Exact(BoardEvaluation::WhiteMate(0)),
        0,
        depth,
        selective_depth,
    );

    let position_search = PositionSearchRow {
        run_id: 0, // NEEDS TO BE CHANGED HIGHER UP
        uci_position: "".to_string(), // NEEDS TO BE CHANGED HIGHER UP
        depth,
        time_taken: total_search_time.elapsed().unwrap_or(Duration::from_secs(0)).as_millis() as u32,
        nodes_evaluated: search_result.nodes_searched().unwrap_or(0),
        evaluation: search_result.eval_bound().board_evaluation(),
        conspiracy_counter: None,
        move_num: 0, // NEEDS TO BE CHANGED HIGHER UP
        timestamp: total_search_time.duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0)).as_secs() as i64,
    };

    (
        search_result,
        position_search,
    )
}

pub fn search_alpha_beta<T: SearchResult + Default>(
    board: &Board,
    // transposition_table: &mut impl TranspositionTable,
    transposition_table: &mut Box<dyn TranspositionTable>,
    mut visited_boards: Vec<u64>,
    simple_evaluation: Centipawns,
    alpha: EvalBound,
    beta: EvalBound,
    current_depth: u32,
    max_depth: u32,
    max_selective_depth: u32,
) -> T { // (_, eval, nodes)
    let mut nodes_searched = 1;

    let mut alpha = alpha;
    let mut beta = beta;

    // dummy move, should always be overridden
    // unless the game is over
    let mut best_move = ChessMove::default();

    let mut move_gen = MoveGen::new_legal(board);
    let board_status = game_status(&board, move_gen.len() != 0);

    // let current_evaluation = BoardEvaluation::PieceScore(simple_evaluation);

    // Do draw detection before quiescence search
    // => No draw detection necessary when only capturing
    // But still need draw detection on last move before quiescence search

    let been_here_before = visited_boards.contains(&board.get_hash());
    visited_boards.push(board.get_hash());
    let visited_boards = visited_boards;
    if let Some(search_result) = check_game_over(board, board_status, &visited_boards) {
        return search_result;
    }

    // Check if already in transposition table
    let mut already_found_move = None;
    if let Some(solution) = transposition_table.get_transposition(
        board,
        None,
    ) {
        already_found_move = Some(solution.best_move); // register best move for re-use in move ordering

        if solution.depth_searched >= SearchDepth::Depth(max_depth - current_depth) && !been_here_before {
            // Already found something deep enough, so no need to recalculate
            match board.side_to_move() {
                Color::White => {
                    match solution.evaluation {
                        EvalBound::UpperBound(_) => (), // TODO: check if less than alpha
                        EvalBound::Exact(_) => { // Not an upper bound so re-usable
                            return T::make_search_result(
                                solution.best_move,
                                solution.evaluation,
                                None,
                                solution.prime_variation.clone(),
                            )
                        },
                        EvalBound::LowerBound(x) => {
                            if solution.evaluation > beta {
                                return T::make_search_result(
                                    solution.best_move,
                                    solution.evaluation,
                                    None,
                                    solution.prime_variation.clone(),
                                )
                            }
                            if solution.evaluation > alpha {
                                alpha = EvalBound::Exact(x);
                            }
                        }
                    }
                },
                Color::Black => {
                    match solution.evaluation {
                        EvalBound::LowerBound(_) => (), // TODO: check if more than beta
                        EvalBound::Exact(_) => { // Not a lower bound, so re-usable for black
                            return T::make_search_result(
                                solution.best_move,
                                solution.evaluation,
                                None,
                                solution.prime_variation.clone(),
                            )
                        },
                        EvalBound::UpperBound(x) => {
                            if solution.evaluation < alpha {
                                return T::make_search_result(
                                    solution.best_move,
                                    solution.evaluation,
                                    None,
                                    solution.prime_variation.clone(),
                                );
                            }
                            if solution.evaluation < beta {
                                beta = EvalBound::Exact(x);
                            }
                        }
                    }
                },
            }
        }
    }


    if current_depth >= max_depth {
        return quiescence_alpha_beta(
            board,
            transposition_table,
            simple_evaluation,
            alpha,
            beta,
            current_depth + 1,
            max_selective_depth
        );
    }

    let all_moves: Vec<ChessMove> = order_moves(
        board,
        already_found_move,
        &mut move_gen,
        false,
    );

    let mut best_eval;
    let mut best_search_result = T::default();

    if all_moves.len() == 0 {
        panic!("WARNING continuing with empty all_moves");
    }
    if board.side_to_move() == Color::White {
        best_eval = EvalBound::UpperBound(BoardEvaluation::BlackMate(0));

        for chess_move in all_moves.into_iter() {
            let new_board = &board.make_move_new(chess_move);
            let improvement = incremental_evaluation(
                &board,
                &chess_move,
                board.side_to_move(),
            );

            let search_result: T = search_alpha_beta(
                new_board,
                transposition_table,
                visited_boards.clone(),
                simple_evaluation + improvement,  // + because white
                alpha,
                beta,
                current_depth + 1,
                max_depth,
                max_selective_depth,
            );

            nodes_searched += search_result.nodes_searched().unwrap_or(1);
            if search_result.eval_bound() >= best_eval {
                best_eval = search_result.eval_bound();
                best_move = chess_move;
                best_search_result = search_result;
            }

            // alpha = max(alpha, best_eval);
            if best_eval > alpha {
                alpha = EvalBound::Exact(best_eval.board_evaluation());
            }
            if beta < alpha {
                break;
            }
        }
    } else { // black to play
        best_eval = EvalBound::LowerBound(BoardEvaluation::WhiteMate(0));

        for chess_move in all_moves.into_iter() {
            let new_board = &board.make_move_new(chess_move);
            let improvement = incremental_evaluation(
                &board,
                &chess_move,
                board.side_to_move(),
            );

            let search_result: T = search_alpha_beta(
                new_board,
                transposition_table,
                visited_boards.clone(),
                simple_evaluation - improvement,  // - because black
                alpha,
                beta,
                current_depth + 1,
                max_depth,
                max_selective_depth,
            );

            nodes_searched += search_result.nodes_searched().unwrap_or(1);

            if search_result.eval_bound() <= best_eval {
                best_eval = search_result.eval_bound();
                best_move = chess_move;
                best_search_result = search_result;
            }

            // beta = min(beta, best_eval);
            if best_eval < beta {
                beta = EvalBound::Exact(best_eval.board_evaluation());
            }
            if beta < alpha {
                break;
            }
        }
    }

    best_eval.set_board_evaluation(bubble_evaluation(best_eval.board_evaluation()));

    let eval_bound = match (board.side_to_move(), beta < alpha) {
        (_, false) => EvalBound::Exact(best_eval.board_evaluation()),
        (Color::White, true) => EvalBound::LowerBound(best_eval.board_evaluation()),
        (Color::Black, true) => EvalBound::UpperBound(best_eval.board_evaluation()),
    };

    best_search_result.prepend_move(best_move);
    transposition_table.update(
        board,
        SearchDepth::Depth(max_depth - current_depth),
        eval_bound,
        best_move,
        best_search_result.critical_path(),
    );
    best_search_result.set_nodes_searched(Some(nodes_searched));
    best_search_result.set_best_move(best_move);
    best_search_result.set_eval_bound(eval_bound);
    best_search_result
}


pub fn quiescence_alpha_beta<T: SearchResult + Default>(
    board: &Board,
    // transposition_table: &mut impl TranspositionTable,
    transposition_table: &mut Box<dyn TranspositionTable>,
    simple_evaluation: Centipawns,
    alpha: EvalBound,
    beta: EvalBound,
    current_depth: u32,
    max_selective_depth: u32,
) -> T { // (_, eval, nodes)
    let mut alpha = alpha;
    let mut beta = beta;

    let mut nodes_searched = 1;
    let mut best_move = ChessMove::default();

    let mut move_gen = MoveGen::new_legal(&board);
    let board_status = game_status(&board, move_gen.len() != 0);

    let current_evaluation = BoardEvaluation::PieceScore(simple_evaluation);

    if board_status == BoardStatus::Checkmate {
        return T::make_search_result(
            best_move,
            {
                match board.side_to_move() {
                    Color::White => EvalBound::Exact(BoardEvaluation::BlackMate(1)), // black has checkmated white
                    Color::Black => EvalBound::Exact(BoardEvaluation::WhiteMate(1)),
                }
            },
            None,
            None,
        );
    }

    if board_status == BoardStatus::Stalemate {
        return T::make_search_result(
            best_move,
            EvalBound::Exact(BoardEvaluation::PieceScore(Centipawns::new(0))),
            None,
            None,
        );
    }

    // HERE: can check if already in TT:
    // but quiescent won't be put in TT.
    // IF WANT TT: CHECK TT HERE

    if current_depth >= max_selective_depth {
        return T::make_search_result(
            best_move,
            EvalBound::Exact(current_evaluation),
            None,
            None,
        );
    }

    // let moves = order_captures(
    //     &board,
    //     current_evaluation,
    //     transposition_table,
    //     &mut move_gen,
    // );
    let already_found_move = None; // TODO: get from TT
    let moves = order_moves(
        &board,
        // transposition_table,
        already_found_move,
        &mut move_gen,
        true,
    );

    if moves.is_empty() {
        return T::make_search_result(
            best_move,
            EvalBound::Exact(current_evaluation),
            None,
            None,
        );
    }

    let mut best_eval = EvalBound::Exact(BoardEvaluation::PieceScore(Centipawns::new(0)));
    // let mut search_result = T::default();
    let mut best_search_result = T::default();
    if board.side_to_move() == Color::White {
        best_eval = EvalBound::Exact(BoardEvaluation::BlackMate(0));
        // let mut best_path = Vec::new();
        // let mut search_result = T::default();

        for chess_move in moves.into_iter() {
            let improvement = incremental_evaluation(
                &board,
                &chess_move,
                board.side_to_move(),
            );
            let search_result: T = quiescence_alpha_beta(
                &board.make_move_new(chess_move),
                transposition_table,
                simple_evaluation + improvement, // + because white
                alpha,
                beta,
                current_depth + 1,
                max_selective_depth,
            );
            nodes_searched += search_result.nodes_searched().unwrap_or(1);

            if search_result.eval_bound() >= best_eval {
                best_eval = search_result.eval_bound();
                best_move = chess_move;
                best_search_result = search_result;
                best_search_result.prepend_move(best_move);

                // best_path = search_result.critical_path;
                // best_path.push(best_move);
            }

            // alpha = max(alpha, best_eval);
            if best_eval > alpha {
                alpha = best_eval;
            }
            if beta < alpha {
                break;
                // best_eval = bubble_evaluation(best_eval);
                // update_transposition(
                //     transposition_table,
                //     board,
                //     SearchDepth::Quiescent,
                //     best_eval,
                // );
                // best_path.push(best_move);
                // return T::make_search_result(
                //     best_move,
                //     best_eval,
                //     Some(nodes_searched),
                //     Some(best_path),
                // );
            }
        }

        // ADD NULL_MOVE FOR QUIESCENCE SEARCH:
        // Opponent can choose to not capture most of the time.
        // Can play non-captures, but those are not accounted for in quiescence search.
        // (Not forced to take pawn with queen if only capture available)
        if let Some(null_board) = board.null_move() {
            let search_result: T = quiescence_alpha_beta(
                &null_board,
                transposition_table,
                simple_evaluation,
                alpha,
                beta,
                current_depth + 1,
                max_selective_depth - 1, // Quiescence should cut off at even depth, and we're skipping a move
            );
            nodes_searched += search_result.nodes_searched().unwrap_or(1);

            if search_result.eval_bound() >= best_eval {
                best_eval = search_result.eval_bound();
                best_move = ChessMove::default();
                best_search_result = T::make_search_result(
                    best_move,
                    best_eval,
                    Some(nodes_searched),
                    None,  // Not an actual legal move => no actual line of play further
                );
            }
        }

        // Or take the static evaluation with a penalty, to remove the amount of blunders
        // due to "having to capture, or give up the turn"
        match current_evaluation {
            BoardEvaluation::PieceScore(x) => {
                let penalized_score = EvalBound::Exact(BoardEvaluation::PieceScore(x - Centipawns::new(54)));

                if penalized_score > best_eval {
                    best_eval = penalized_score;
                    best_move = ChessMove::default();
                    best_search_result = T::make_search_result(
                        best_move,
                        best_eval,
                        Some(nodes_searched),
                        None,
                    );
                }
            },
            _ => (),
        };
    } else { // black to play
        best_eval = EvalBound::Exact(BoardEvaluation::WhiteMate(0));

        for chess_move in moves.into_iter() {
            let improvement = incremental_evaluation(
                &board,
                &chess_move,
                board.side_to_move(),
            );
            let search_result: T = quiescence_alpha_beta(
                &board.make_move_new(chess_move),
                transposition_table,
                simple_evaluation - improvement,  // - because black
                alpha,
                beta,
                current_depth + 1,
                max_selective_depth,
            );
            nodes_searched += search_result.nodes_searched().unwrap_or(1);

            if search_result.eval_bound() <= best_eval {
                best_eval = search_result.eval_bound();
                best_move = chess_move;
                best_search_result = search_result;
                best_search_result.prepend_move(best_move);
                // best_path.push(best_move);
            }

            // beta = min(beta, best_eval);
            if best_eval < beta {
                beta = best_eval;
            }
            if beta < alpha {
                break;
                // best_eval = bubble_evaluation(best_eval);
                // update_transposition(
                //     transposition_table,
                //     board,
                //     SearchDepth::Quiescent,
                //     best_eval,
                // );
                // best_path.push(best_move);
                // return T::make_search_result(
                //     best_move,
                //     best_eval,
                //     Some(nodes_searched),
                //     Some(best_path),
                // );
            }
        }
        // ADD NULL_MOVE FOR QUIESCENCE SEARCH:
        // Opponent can choose to not capture most of the time.
        // Can play non-captures, but those are not accounted for in quiescence search.
        // (Not forced to take pawn with queen if only capture available)
        if let Some(null_board) = board.null_move() {
            let search_result: T = quiescence_alpha_beta(
                &null_board,
                transposition_table,
                simple_evaluation,
                alpha,
                beta,
                current_depth + 1,
                max_selective_depth - 1, // Quiescence should cut off at even depth, and we're skipping a move
            );
            nodes_searched += search_result.nodes_searched().unwrap_or(1);

            if search_result.eval_bound() <= best_eval {
                best_eval = search_result.eval_bound();
                best_move = ChessMove::default();
                best_search_result = T::make_search_result(
                    best_move,
                    best_eval,
                    Some(nodes_searched),
                    None
                );
                // best_path = Vec::new(); // Not an actual legal move => no actual line of play further
            }
        }

        // Or take the static evaluation with a penalty, to remove the amount of blunders
        // due to "having to capture, or give up the turn"
        match current_evaluation {
            BoardEvaluation::PieceScore(x) => {
                let penalized_score = EvalBound::Exact(BoardEvaluation::PieceScore(x + Centipawns::new(54)));

                if penalized_score < best_eval {
                    best_eval = penalized_score;
                    best_move = ChessMove::default();
                    best_search_result = T::make_search_result(
                        best_move,
                        best_eval,
                        Some(nodes_searched),
                        None
                    );
                }
            },
            _ => (),
        };
    }

    best_eval.set_board_evaluation(bubble_evaluation(best_eval.board_evaluation()));

    let eval_bound = match (board.side_to_move(), beta < alpha) {
        (_, false) => EvalBound::Exact(best_eval.board_evaluation()),
        (Color::White, true) => EvalBound::LowerBound(best_eval.board_evaluation()),
        (Color::Black, true) => EvalBound::UpperBound(best_eval.board_evaluation()),
    };
    // No need to update TT: quiescent too low depth
    // transposition_table.update(
    //     board,
    //     SearchDepth::Quiescent,
    //     best_eval,
    //     best_move,
    // );
    best_search_result.set_nodes_searched(Some(nodes_searched));
    best_search_result.set_best_move(best_move);
    best_search_result.set_eval_bound(eval_bound);
    best_search_result
}



